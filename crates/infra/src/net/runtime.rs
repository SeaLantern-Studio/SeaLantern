//! 进程级全局网络运行时。
//!
//! 集中维护当前代理策略和可复用的 HTTP 客户端。配置文件的读取与持久化由
//! 上层负责；本模块只接收已解析的代理设置或系统代理快照。

use std::sync::RwLock;

use super::proxy::{ProxyConfigError, ProxyController, ProxySettings, SystemProxySnapshot};
use super::{ClientConfig, NetClient, NetError};

/// 网络客户端更新结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkUpdate {
    /// 是否创建并替换了当前 HTTP 客户端。
    pub client_rebuilt: bool,
    /// 当前客户端世代；初始客户端为第一代。
    pub revision: u64,
}

/// 网络运行时状态容器，仅由本模块的进程级全局实例对外提供能力。
struct NetworkRuntime {
    state: RwLock<Option<NetworkState>>,
}

#[derive(Clone)]
struct NetworkState {
    controller: ProxyController,
    client: NetClient,
    revision: u64,
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

        let initialized = build_state(ProxySettings::default(), SystemProxySnapshot::direct(), 1)?;
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
        let Some(current) = state.as_ref() else {
            let initialized = build_state(settings, system_proxy, 1)?;
            *state = Some(initialized);
            return Ok(NetworkUpdate { client_rebuilt: true, revision: 1 });
        };

        let mut next_controller = current.controller.clone();
        let update = next_controller
            .update_settings(settings, system_proxy)
            .map_err(proxy_config_error)?;

        if update.changed() {
            let next_client = NetClient::from_config_with_effective_proxy(
                &ClientConfig::default(),
                &update.current,
            )?;
            let revision = next_revision(current.revision)?;
            *state = Some(NetworkState {
                controller: next_controller,
                client: next_client,
                revision,
            });
            Ok(NetworkUpdate { client_rebuilt: true, revision })
        } else {
            let revision = current.revision;
            let client = current.client.clone();
            *state = Some(NetworkState {
                controller: next_controller,
                client,
                revision,
            });
            Ok(NetworkUpdate { client_rebuilt: false, revision })
        }
    }

    /// 应用操作系统代理变化。
    ///
    /// 只有自适应策略会根据新快照改变有效代理并重建客户端。
    fn apply_system_proxy_snapshot(
        &self,
        system_proxy: SystemProxySnapshot,
    ) -> Result<NetworkUpdate, NetError> {
        let mut state = self.write_state()?;
        let Some(current) = state.as_ref() else {
            let initialized = build_state(ProxySettings::default(), system_proxy, 1)?;
            *state = Some(initialized);
            return Ok(NetworkUpdate { client_rebuilt: true, revision: 1 });
        };

        let mut next_controller = current.controller.clone();
        let update = next_controller.handle_system_proxy_change(system_proxy);
        if update.changed() {
            let next_client = NetClient::from_config_with_effective_proxy(
                &ClientConfig::default(),
                &update.current,
            )?;
            let revision = next_revision(current.revision)?;
            *state = Some(NetworkState {
                controller: next_controller,
                client: next_client,
                revision,
            });
            Ok(NetworkUpdate { client_rebuilt: true, revision })
        } else {
            let revision = current.revision;
            let client = current.client.clone();
            *state = Some(NetworkState {
                controller: next_controller,
                client,
                revision,
            });
            Ok(NetworkUpdate { client_rebuilt: false, revision })
        }
    }

    fn read_state(&self) -> Result<std::sync::RwLockReadGuard<'_, Option<NetworkState>>, NetError> {
        self.state
            .read()
            .map_err(|_| NetError::Config("全局网络运行时不可用".into()))
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, Option<NetworkState>>, NetError> {
        self.state
            .write()
            .map_err(|_| NetError::Config("全局网络运行时不可用".into()))
    }
}

fn build_state(
    settings: ProxySettings,
    system_proxy: SystemProxySnapshot,
    revision: u64,
) -> Result<NetworkState, NetError> {
    let controller = ProxyController::new(settings, system_proxy).map_err(proxy_config_error)?;
    let client = NetClient::from_config_with_effective_proxy(
        &ClientConfig::default(),
        controller.effective_proxy(),
    )?;
    Ok(NetworkState { controller, client, revision })
}

fn proxy_config_error(_error: ProxyConfigError) -> NetError {
    NetError::Config("代理设置无效".into())
}

fn next_revision(revision: u64) -> Result<u64, NetError> {
    revision
        .checked_add(1)
        .ok_or_else(|| NetError::Config("网络客户端世代已耗尽".into()))
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

    #[test]
    fn client_initializes_once_with_direct_adaptive_state() {
        let runtime = NetworkRuntime::new();

        runtime.client().expect("first client should initialize");
        runtime.client().expect("second client should reuse state");

        let state = runtime_state(&runtime);
        assert_eq!(state.revision, 1);
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

        assert_eq!(first, NetworkUpdate { client_rebuilt: true, revision: 2 });
        assert_eq!(repeated, NetworkUpdate { client_rebuilt: false, revision: 2 });
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
        assert_eq!(after.revision, before.revision);
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
        assert_eq!(second.revision, 3);
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
        assert_eq!(initialized.revision, 1);
        assert_eq!(unchanged, NetworkUpdate { client_rebuilt: false, revision: 1 });

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

        assert_eq!(runtime_state(&runtime).revision, 1);
    }
}
