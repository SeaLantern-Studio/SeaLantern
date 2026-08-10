//! 进程级全局网络运行时。
//!
//! 集中维护当前代理策略和可复用的 HTTP 客户端。配置文件的读取与持久化由
//! 上层负责；本模块只接收已解析的代理设置或系统代理快照。

use std::fmt;
use std::sync::RwLock;

use super::proxy::{ProxyConfigError, ProxyController, ProxySettings, SystemProxySnapshot};
use super::{ClientConfig, NetClient, NetError};

/// 网络客户端更新结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkUpdate {
    /// 是否创建并替换了当前 HTTP 客户端。
    pub client_rebuilt: bool,
    /// 当前运行时状态世代；任意实际状态变化都会递增。
    pub state_revision: u64,
    /// 当前客户端世代；仅替换 HTTP 客户端时递增。
    pub client_revision: u64,
}

/// 已完成代理验证和候选客户端构建、等待提交的运行时更新。
pub struct PreparedNetworkUpdate {
    expected_state_revision: u64,
    candidate: NetworkState,
    client_rebuilt: bool,
}

/// 提交已准备网络更新时可能发生的错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCommitError {
    /// 准备完成后运行时已被其他更新修改，候选状态已过期。
    Conflict {
        expected_revision: u64,
        actual_revision: u64,
    },
    /// 全局运行时状态锁已污染。
    RuntimeUnavailable,
}

impl fmt::Display for NetworkCommitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict { expected_revision, actual_revision } => write!(
                formatter,
                "网络运行时状态冲突：预期世代 {expected_revision}，实际世代 {actual_revision}"
            ),
            Self::RuntimeUnavailable => formatter.write_str("全局网络运行时状态锁已污染"),
        }
    }
}

impl std::error::Error for NetworkCommitError {}

/// 网络运行时状态容器，仅由本模块的进程级全局实例对外提供能力。
struct NetworkRuntime {
    state: RwLock<Option<NetworkState>>,
}

#[derive(Clone)]
struct NetworkState {
    controller: ProxyController,
    client: NetClient,
    state_revision: u64,
    client_revision: u64,
}

impl NetworkRuntime {
    /// 创建尚未初始化的网络运行时。
    const fn new() -> Self {
        Self { state: RwLock::new(None) }
    }

    /// 获取当前网络客户端的廉价克隆。
    ///
    /// 首次获取时使用默认自适应策略和直连系统快照初始化第一代客户端。
    fn client(&self) -> Result<NetClient, NetError> {
        if let Some(client) = self
            .read_state()?
            .as_ref()
            .map(|state| state.client.clone())
        {
            return Ok(client);
        }

        let mut state = self.write_state()?;
        if let Some(existing) = state.as_ref() {
            return Ok(existing.client.clone());
        }

        let initialized = build_state(ProxySettings::default(), SystemProxySnapshot::direct())?;
        let client = initialized.client.clone();
        *state = Some(initialized);
        Ok(client)
    }

    /// 应用由上层下发的代理设置。
    ///
    /// 新客户端构建成功后才会替换当前状态；失败时保留旧策略和旧客户端。
    fn apply_proxy_settings(
        &self,
        settings: ProxySettings,
        system_proxy: SystemProxySnapshot,
    ) -> Result<NetworkUpdate, NetError> {
        let mut state = self.write_state()?;
        let Some(current) = state.as_mut() else {
            let initialized = build_state(settings, system_proxy)?;
            *state = Some(initialized);
            return Ok(NetworkUpdate {
                client_rebuilt: true,
                state_revision: 1,
                client_revision: 1,
            });
        };

        let mut next_controller = current.controller.clone();
        let update = next_controller
            .update_settings(settings, system_proxy)
            .map_err(proxy_config_error)?;

        commit_proxy_update(current, next_controller, update)
    }

    /// 应用操作系统代理变化。
    ///
    /// 只有自适应策略会根据新快照改变有效代理并重建客户端。
    fn apply_system_proxy_snapshot(
        &self,
        system_proxy: SystemProxySnapshot,
    ) -> Result<NetworkUpdate, NetError> {
        let mut state = self.write_state()?;
        let Some(current) = state.as_mut() else {
            let initialized = build_state(ProxySettings::default(), system_proxy)?;
            *state = Some(initialized);
            return Ok(NetworkUpdate {
                client_rebuilt: true,
                state_revision: 1,
                client_revision: 1,
            });
        };

        let mut next_controller = current.controller.clone();
        let update = next_controller.handle_system_proxy_change(system_proxy);
        commit_proxy_update(current, next_controller, update)
    }

    fn prepare_proxy_settings(
        &self,
        settings: ProxySettings,
        system_proxy: SystemProxySnapshot,
    ) -> Result<PreparedNetworkUpdate, NetError> {
        let state = self.read_state()?;
        let current = state.as_ref().cloned();
        drop(state);
        let Some(current) = current else {
            return Ok(PreparedNetworkUpdate {
                expected_state_revision: 0,
                candidate: build_state(settings, system_proxy)?,
                client_rebuilt: true,
            });
        };

        let mut next_controller = current.controller.clone();
        let update = next_controller
            .update_settings(settings, system_proxy)
            .map_err(proxy_config_error)?;
        let controller_changed = controller_changed(&current.controller, &next_controller);
        let state_revision = if controller_changed {
            next_revision(current.state_revision, "网络运行时状态世代已耗尽")?
        } else {
            current.state_revision
        };
        let (client, client_revision, client_rebuilt) = if update.changed() {
            let client = NetClient::from_config_with_effective_proxy(
                &ClientConfig::default(),
                &update.current,
            )?;
            let revision = next_revision(current.client_revision, "网络客户端世代已耗尽")?;
            (client, revision, true)
        } else {
            (current.client.clone(), current.client_revision, false)
        };

        Ok(PreparedNetworkUpdate {
            expected_state_revision: current.state_revision,
            candidate: NetworkState {
                controller: next_controller,
                client,
                state_revision,
                client_revision,
            },
            client_rebuilt,
        })
    }

    fn commit_prepared_proxy_update(
        &self,
        prepared: PreparedNetworkUpdate,
    ) -> Result<NetworkUpdate, NetworkCommitError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| NetworkCommitError::RuntimeUnavailable)?;
        let actual_revision = state.as_ref().map_or(0, |current| current.state_revision);
        if actual_revision != prepared.expected_state_revision {
            return Err(NetworkCommitError::Conflict {
                expected_revision: prepared.expected_state_revision,
                actual_revision,
            });
        }

        let update = NetworkUpdate {
            client_rebuilt: prepared.client_rebuilt,
            state_revision: prepared.candidate.state_revision,
            client_revision: prepared.candidate.client_revision,
        };
        *state = Some(prepared.candidate);
        Ok(update)
    }

    fn read_state(&self) -> Result<std::sync::RwLockReadGuard<'_, Option<NetworkState>>, NetError> {
        self.state.read().map_err(runtime_lock_poisoned)
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, Option<NetworkState>>, NetError> {
        self.state.write().map_err(runtime_lock_poisoned)
    }
}

fn build_state(
    settings: ProxySettings,
    system_proxy: SystemProxySnapshot,
) -> Result<NetworkState, NetError> {
    let controller = ProxyController::new(settings, system_proxy).map_err(proxy_config_error)?;
    let client = NetClient::from_config_with_effective_proxy(
        &ClientConfig::default(),
        controller.effective_proxy(),
    )?;
    Ok(NetworkState {
        controller,
        client,
        state_revision: 1,
        client_revision: 1,
    })
}

fn commit_proxy_update(
    current: &mut NetworkState,
    next_controller: ProxyController,
    update: super::ProxyUpdate,
) -> Result<NetworkUpdate, NetError> {
    if !update.changed() {
        if !controller_changed(&current.controller, &next_controller) {
            return Ok(NetworkUpdate {
                client_rebuilt: false,
                state_revision: current.state_revision,
                client_revision: current.client_revision,
            });
        }
        let state_revision = next_revision(current.state_revision, "网络运行时状态世代已耗尽")?;
        current.controller = next_controller;
        current.state_revision = state_revision;
        return Ok(NetworkUpdate {
            client_rebuilt: false,
            state_revision: current.state_revision,
            client_revision: current.client_revision,
        });
    }

    let next_client =
        NetClient::from_config_with_effective_proxy(&ClientConfig::default(), &update.current)?;
    let state_revision = next_revision(current.state_revision, "网络运行时状态世代已耗尽")?;
    let client_revision = next_revision(current.client_revision, "网络客户端世代已耗尽")?;
    current.controller = next_controller;
    current.client = next_client;
    current.state_revision = state_revision;
    current.client_revision = client_revision;
    Ok(NetworkUpdate {
        client_rebuilt: true,
        state_revision,
        client_revision,
    })
}

fn controller_changed(current: &ProxyController, next: &ProxyController) -> bool {
    current.settings() != next.settings() || current.effective_proxy() != next.effective_proxy()
}

fn proxy_config_error(error: ProxyConfigError) -> NetError {
    let message = match error {
        ProxyConfigError::EmptyManualProxy => "手动代理地址不能为空",
    };
    NetError::Config(message.into())
}

fn runtime_lock_poisoned<T>(_error: std::sync::PoisonError<T>) -> NetError {
    NetError::Config("全局网络运行时状态锁已污染".into())
}

fn next_revision(revision: u64, exhausted_message: &'static str) -> Result<u64, NetError> {
    revision
        .checked_add(1)
        .ok_or_else(|| NetError::Config(exhausted_message.into()))
}

static GLOBAL_NETWORK_RUNTIME: NetworkRuntime = NetworkRuntime::new();

/// 获取进程级全局网络客户端。
pub fn global_client() -> Result<NetClient, NetError> {
    GLOBAL_NETWORK_RUNTIME.client()
}

/// 向进程级全局网络运行时下发代理设置。
pub fn apply_proxy_settings(
    settings: ProxySettings,
    system_proxy: SystemProxySnapshot,
) -> Result<NetworkUpdate, NetError> {
    GLOBAL_NETWORK_RUNTIME.apply_proxy_settings(settings, system_proxy)
}

/// 验证代理设置并预构建候选客户端，但不修改当前全局运行时。
pub fn prepare_proxy_settings(
    settings: ProxySettings,
    system_proxy: SystemProxySnapshot,
) -> Result<PreparedNetworkUpdate, NetError> {
    GLOBAL_NETWORK_RUNTIME.prepare_proxy_settings(settings, system_proxy)
}

/// 在运行时状态未变化时提交预构建的代理更新。
pub fn commit_prepared_proxy_update(
    prepared: PreparedNetworkUpdate,
) -> Result<NetworkUpdate, NetworkCommitError> {
    GLOBAL_NETWORK_RUNTIME.commit_prepared_proxy_update(prepared)
}

/// 向进程级全局网络运行时下发系统代理快照。
pub fn apply_system_proxy_snapshot(
    system_proxy: SystemProxySnapshot,
) -> Result<NetworkUpdate, NetError> {
    GLOBAL_NETWORK_RUNTIME.apply_system_proxy_snapshot(system_proxy)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::net::proxy::{EffectiveProxy, ProxyMode};

    const FIRST_PROXY: &str = "http://127.0.0.1:7890";
    const SECOND_PROXY: &str = "http://127.0.0.1:7891";

    fn settings(mode: ProxyMode) -> ProxySettings {
        ProxySettings { mode }
    }

    fn runtime_state(runtime: &NetworkRuntime) -> NetworkState {
        runtime
            .read_state()
            .expect("network state lock should be available")
            .clone()
            .expect("network runtime should be initialized")
    }

    fn assert_send_static<T: Send + 'static>() {}

    #[test]
    fn prepared_network_update_can_cross_async_boundaries() {
        assert_send_static::<PreparedNetworkUpdate>();
    }

    #[test]
    fn client_initializes_once_with_direct_adaptive_state() {
        let runtime = NetworkRuntime::new();

        runtime.client().expect("first client should initialize");
        runtime.client().expect("second client should reuse state");

        let state = runtime_state(&runtime);
        assert_eq!(state.state_revision, 1);
        assert_eq!(state.client_revision, 1);
        assert_eq!(state.controller.settings(), &ProxySettings::default());
        assert_eq!(state.controller.effective_proxy(), &EffectiveProxy::Direct);
    }

    #[test]
    fn manual_proxy_rebuilds_only_when_effective_proxy_changes() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");

        let first = runtime
            .apply_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("manual proxy should apply");
        let repeated = runtime
            .apply_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("same manual proxy should remain valid");

        assert_eq!(
            first,
            NetworkUpdate {
                client_rebuilt: true,
                state_revision: 2,
                client_revision: 2,
            }
        );
        assert_eq!(
            repeated,
            NetworkUpdate {
                client_rebuilt: false,
                state_revision: 2,
                client_revision: 2,
            }
        );
    }

    #[test]
    fn policy_change_without_effective_proxy_change_updates_only_controller() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");

        let update = runtime
            .apply_proxy_settings(settings(ProxyMode::Disabled), SystemProxySnapshot::direct())
            .expect("disabled mode should apply");

        assert_eq!(
            update,
            NetworkUpdate {
                client_rebuilt: false,
                state_revision: 2,
                client_revision: 1,
            }
        );
        let state = runtime_state(&runtime);
        assert_eq!(state.controller.settings().mode, ProxyMode::Disabled);
        assert_eq!(state.controller.effective_proxy(), &EffectiveProxy::Direct);
    }

    #[test]
    fn invalid_proxy_settings_return_a_specific_reason_and_keep_state() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");
        let before = runtime_state(&runtime);

        let error = runtime
            .apply_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: "  ".into() }),
                SystemProxySnapshot::direct(),
            )
            .unwrap_err();

        assert_eq!(error.to_string(), "配置错误: 手动代理地址不能为空");
        let after = runtime_state(&runtime);
        assert_eq!(after.state_revision, before.state_revision);
        assert_eq!(after.client_revision, before.client_revision);
        assert_eq!(after.controller.settings(), before.controller.settings());
        assert_eq!(after.controller.effective_proxy(), before.controller.effective_proxy());
    }

    #[test]
    fn failed_candidate_keeps_previous_runtime_state() {
        let runtime = NetworkRuntime::new();
        runtime
            .apply_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("initial manual proxy should apply");
        let before = runtime_state(&runtime);

        let result = runtime.apply_proxy_settings(
            settings(ProxyMode::Manual {
                proxy_url: "http://user:secret@[::1".into(),
            }),
            SystemProxySnapshot::direct(),
        );

        assert!(matches!(result, Err(NetError::Config(_))));
        let after = runtime_state(&runtime);
        assert_eq!(after.state_revision, before.state_revision);
        assert_eq!(after.client_revision, before.client_revision);
        assert_eq!(after.controller.settings(), before.controller.settings());
        assert_eq!(after.controller.effective_proxy(), before.controller.effective_proxy());
    }

    #[test]
    fn adaptive_mode_rebuilds_for_system_proxy_changes() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");

        let first = runtime
            .apply_system_proxy_snapshot(SystemProxySnapshot::proxy(FIRST_PROXY))
            .expect("first system proxy should apply");
        let second = runtime
            .apply_system_proxy_snapshot(SystemProxySnapshot::proxy(SECOND_PROXY))
            .expect("second system proxy should apply");

        assert!(first.client_rebuilt);
        assert!(second.client_rebuilt);
        assert_eq!(second.state_revision, 3);
        assert_eq!(second.client_revision, 3);
    }

    #[test]
    fn preserve_and_disabled_modes_ignore_system_proxy_changes() {
        let preserve = NetworkRuntime::new();
        let initialized = preserve
            .apply_proxy_settings(
                settings(ProxyMode::Preserve),
                SystemProxySnapshot::proxy(FIRST_PROXY),
            )
            .expect("preserve mode should initialize");
        let unchanged = preserve
            .apply_system_proxy_snapshot(SystemProxySnapshot::proxy(SECOND_PROXY))
            .expect("preserve mode should accept system events");
        assert_eq!(initialized.state_revision, 1);
        assert_eq!(initialized.client_revision, 1);
        assert_eq!(
            unchanged,
            NetworkUpdate {
                client_rebuilt: false,
                state_revision: 1,
                client_revision: 1,
            }
        );

        let disabled = NetworkRuntime::new();
        disabled
            .apply_proxy_settings(
                settings(ProxyMode::Disabled),
                SystemProxySnapshot::proxy(FIRST_PROXY),
            )
            .expect("disabled mode should initialize");
        assert_eq!(runtime_state(&disabled).controller.effective_proxy(), &EffectiveProxy::Direct);
    }

    #[test]
    fn prepared_update_does_not_apply_until_committed() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");
        let prepared = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("manual proxy should prepare");

        assert_eq!(runtime_state(&runtime).controller.effective_proxy(), &EffectiveProxy::Direct);

        let committed = runtime
            .commit_prepared_proxy_update(prepared)
            .expect("prepared proxy should commit");
        assert_eq!(
            committed,
            NetworkUpdate {
                client_rebuilt: true,
                state_revision: 2,
                client_revision: 2,
            }
        );
        assert_eq!(
            runtime_state(&runtime)
                .controller
                .effective_proxy()
                .routes_ref()
                .and_then(|routes| routes.http_proxy()),
            Some(FIRST_PROXY)
        );
    }

    #[test]
    fn failed_prepare_keeps_current_runtime_state() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");
        let before = runtime_state(&runtime);

        let result = runtime.prepare_proxy_settings(
            settings(ProxyMode::Manual {
                proxy_url: "http://user:secret@[::1".into(),
            }),
            SystemProxySnapshot::direct(),
        );

        assert!(matches!(result, Err(NetError::Config(_))));
        let after = runtime_state(&runtime);
        assert_eq!(after.state_revision, before.state_revision);
        assert_eq!(after.client_revision, before.client_revision);
        assert_eq!(after.controller.settings(), before.controller.settings());
        assert_eq!(after.controller.effective_proxy(), before.controller.effective_proxy());
    }

    #[test]
    fn stale_prepared_update_conflicts_without_overwriting_current_state() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");
        let prepared = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("manual proxy should prepare");
        runtime
            .apply_system_proxy_snapshot(SystemProxySnapshot::proxy(SECOND_PROXY))
            .expect("system proxy should update current runtime");

        let error = runtime.commit_prepared_proxy_update(prepared).unwrap_err();

        assert_eq!(
            error,
            NetworkCommitError::Conflict { expected_revision: 1, actual_revision: 2 }
        );
        let state = runtime_state(&runtime);
        assert_eq!(state.state_revision, 2);
        assert_eq!(state.client_revision, 2);
        assert_eq!(
            state
                .controller
                .effective_proxy()
                .routes_ref()
                .and_then(|routes| routes.http_proxy()),
            Some(SECOND_PROXY)
        );
    }

    #[test]
    fn state_only_change_invalidates_a_prepared_update() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");
        let prepared = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("manual proxy should prepare");
        let state_only = runtime
            .apply_proxy_settings(settings(ProxyMode::Disabled), SystemProxySnapshot::direct())
            .expect("disabled policy should apply without rebuilding the client");

        assert_eq!(state_only.state_revision, 2);
        assert_eq!(state_only.client_revision, 1);
        assert!(!state_only.client_rebuilt);
        assert_eq!(
            runtime.commit_prepared_proxy_update(prepared),
            Err(NetworkCommitError::Conflict { expected_revision: 1, actual_revision: 2 })
        );
    }

    #[test]
    fn only_one_prepared_update_can_commit_for_the_same_revision() {
        let runtime = NetworkRuntime::new();
        runtime.client().expect("default client should initialize");
        let first = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("first proxy should prepare");
        let second = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: SECOND_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("second proxy should prepare");

        runtime
            .commit_prepared_proxy_update(first)
            .expect("first candidate should commit");
        assert!(matches!(
            runtime.commit_prepared_proxy_update(second),
            Err(NetworkCommitError::Conflict { .. })
        ));
    }

    #[test]
    fn uninitialized_runtime_accepts_one_prepared_initial_state() {
        let runtime = NetworkRuntime::new();
        let prepared = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("initial proxy should prepare");

        let committed = runtime
            .commit_prepared_proxy_update(prepared)
            .expect("initial proxy should commit");

        assert_eq!(committed.state_revision, 1);
        assert_eq!(committed.client_revision, 1);
        assert!(committed.client_rebuilt);
    }

    #[test]
    fn lazy_initialization_invalidates_an_uninitialized_candidate() {
        let runtime = NetworkRuntime::new();
        let prepared = runtime
            .prepare_proxy_settings(
                settings(ProxyMode::Manual { proxy_url: FIRST_PROXY.into() }),
                SystemProxySnapshot::direct(),
            )
            .expect("initial proxy should prepare");
        runtime
            .client()
            .expect("default client should initialize first");

        assert_eq!(
            runtime.commit_prepared_proxy_update(prepared),
            Err(NetworkCommitError::Conflict { expected_revision: 0, actual_revision: 1 })
        );
    }

    #[test]
    fn concurrent_client_reads_share_one_initialized_generation() {
        let runtime = Arc::new(NetworkRuntime::new());
        let threads = (0..8)
            .map(|_| {
                let runtime = runtime.clone();
                std::thread::spawn(move || runtime.client().expect("client should initialize"))
            })
            .collect::<Vec<_>>();

        for thread in threads {
            thread.join().expect("client thread should finish");
        }

        let state = runtime_state(&runtime);
        assert_eq!(state.state_revision, 1);
        assert_eq!(state.client_revision, 1);
    }
}
