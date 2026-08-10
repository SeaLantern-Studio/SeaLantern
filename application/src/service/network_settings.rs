//! 持久化代理设置与进程级网络运行时之间的同步边界。

use sealantern_infra::net::proxy::{ProxyMode, ProxySettings};
use sealantern_infra::net::{
    commit_prepared_proxy_update, prepare_proxy_settings, NetworkCommitError,
    PreparedNetworkUpdate, SystemProxySnapshot,
};
use sealantern_infra::platform::current_system_proxy;

use crate::error::SettingsError;

const MAX_COMMIT_ATTEMPTS: usize = 3;

pub(super) trait PreparedProxyUpdate: Send {
    fn commit(self: Box<Self>) -> Result<(), NetworkCommitError>;
}

pub(super) trait NetworkSettingsRuntime: Send + Sync {
    fn prepare(
        &self,
        settings: ProxySettings,
    ) -> Result<Box<dyn PreparedProxyUpdate>, SettingsError>;
}

#[derive(Default)]
pub(super) struct GlobalNetworkSettingsRuntime;

struct GlobalPreparedProxyUpdate(PreparedNetworkUpdate);

impl PreparedProxyUpdate for GlobalPreparedProxyUpdate {
    fn commit(self: Box<Self>) -> Result<(), NetworkCommitError> {
        commit_prepared_proxy_update(self.0).map(|_| ())
    }
}

impl NetworkSettingsRuntime for GlobalNetworkSettingsRuntime {
    fn prepare(
        &self,
        settings: ProxySettings,
    ) -> Result<Box<dyn PreparedProxyUpdate>, SettingsError> {
        let system_proxy = match &settings.mode {
            ProxyMode::Adaptive | ProxyMode::Preserve => current_system_proxy()?,
            ProxyMode::Manual { .. } | ProxyMode::Disabled => SystemProxySnapshot::direct(),
        };
        let prepared = prepare_proxy_settings(settings, system_proxy)?;
        Ok(Box::new(GlobalPreparedProxyUpdate(prepared)))
    }
}

/// 提交持久化前准备的候选；revision 冲突时基于已持久化设置有界重试。
pub(super) fn commit_persisted_proxy(
    runtime: &dyn NetworkSettingsRuntime,
    mut prepared: Box<dyn PreparedProxyUpdate>,
    persisted: ProxySettings,
) -> Result<(), SettingsError> {
    for attempt in 0..MAX_COMMIT_ATTEMPTS {
        match prepared.commit() {
            Ok(()) => return Ok(()),
            Err(error @ NetworkCommitError::RuntimeUnavailable) => return Err(error.into()),
            Err(error @ NetworkCommitError::Conflict { .. }) => {
                if attempt + 1 == MAX_COMMIT_ATTEMPTS {
                    return Err(error.into());
                }
                prepared = runtime.prepare(persisted.clone())?;
            }
        }
    }
    unreachable!("提交尝试次数至少为一")
}

#[cfg(test)]
pub(super) struct NoopNetworkSettingsRuntime;

#[cfg(test)]
struct NoopPreparedProxyUpdate;

#[cfg(test)]
impl PreparedProxyUpdate for NoopPreparedProxyUpdate {
    fn commit(self: Box<Self>) -> Result<(), NetworkCommitError> {
        Ok(())
    }
}

#[cfg(test)]
impl NetworkSettingsRuntime for NoopNetworkSettingsRuntime {
    fn prepare(
        &self,
        _settings: ProxySettings,
    ) -> Result<Box<dyn PreparedProxyUpdate>, SettingsError> {
        Ok(Box::new(NoopPreparedProxyUpdate))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use sealantern_infra::net::proxy::ProxySettings;

    use super::*;

    struct FakePrepared {
        outcomes: Arc<Mutex<VecDeque<Result<(), NetworkCommitError>>>>,
    }

    impl PreparedProxyUpdate for FakePrepared {
        fn commit(self: Box<Self>) -> Result<(), NetworkCommitError> {
            self.outcomes
                .lock()
                .expect("测试提交结果锁不应污染")
                .pop_front()
                .expect("测试应提供提交结果")
        }
    }

    struct FakeRuntime {
        prepare_count: Arc<Mutex<usize>>,
        outcomes: Arc<Mutex<VecDeque<Result<(), NetworkCommitError>>>>,
    }

    impl NetworkSettingsRuntime for FakeRuntime {
        fn prepare(
            &self,
            _settings: ProxySettings,
        ) -> Result<Box<dyn PreparedProxyUpdate>, SettingsError> {
            *self.prepare_count.lock().expect("测试准备计数锁不应污染") += 1;
            Ok(Box::new(FakePrepared { outcomes: self.outcomes.clone() }))
        }
    }

    #[test]
    fn commit_reprepares_after_revision_conflict() {
        let prepare_count = Arc::new(Mutex::new(0));
        let outcomes = Arc::new(Mutex::new(VecDeque::from([
            Err(NetworkCommitError::Conflict { expected_revision: 1, actual_revision: 2 }),
            Ok(()),
        ])));
        let runtime = FakeRuntime {
            prepare_count: prepare_count.clone(),
            outcomes: outcomes.clone(),
        };
        let initial = runtime
            .prepare(ProxySettings::default())
            .expect("初次准备应成功");

        commit_persisted_proxy(&runtime, initial, ProxySettings::default())
            .expect("冲突后应重新准备并提交");

        assert_eq!(*prepare_count.lock().expect("测试准备计数锁不应污染"), 2);
        assert!(outcomes.lock().expect("测试提交结果锁不应污染").is_empty());
    }

    #[test]
    fn commit_stops_after_bounded_conflicts() {
        let prepare_count = Arc::new(Mutex::new(0));
        let outcomes = Arc::new(Mutex::new(VecDeque::from([
            Err(NetworkCommitError::Conflict { expected_revision: 1, actual_revision: 2 }),
            Err(NetworkCommitError::Conflict { expected_revision: 2, actual_revision: 3 }),
            Err(NetworkCommitError::Conflict { expected_revision: 3, actual_revision: 4 }),
        ])));
        let runtime = FakeRuntime {
            prepare_count: prepare_count.clone(),
            outcomes,
        };
        let initial = runtime
            .prepare(ProxySettings::default())
            .expect("初次准备应成功");

        assert!(matches!(
            commit_persisted_proxy(&runtime, initial, ProxySettings::default()),
            Err(SettingsError::NetworkSyncFailed { .. })
        ));
        assert_eq!(*prepare_count.lock().expect("测试准备计数锁不应污染"), 3);
    }
}
