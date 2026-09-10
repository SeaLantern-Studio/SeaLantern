//! 系统代理快照的低频轮询与全局网络运行时刷新。

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use sealantern_infra::net::{NetworkUpdate, SystemProxySnapshot, apply_system_proxy_snapshot};
use sealantern_infra::platform::current_system_proxy;

const SYSTEM_PROXY_POLL_INTERVAL: Duration = Duration::from_secs(3);
const SYSTEM_PROXY_STOP_TIMEOUT: Duration = Duration::from_secs(5);

type BoxError = Box<dyn std::error::Error + Send + Sync>;

trait ProxyMonitoringRuntime: Send + Sync {
    fn read_system_proxy(&self) -> Result<SystemProxySnapshot, BoxError>;
    fn apply_system_proxy(&self, snapshot: SystemProxySnapshot) -> Result<NetworkUpdate, BoxError>;
}

#[derive(Default)]
struct GlobalProxyMonitoringRuntime;

impl ProxyMonitoringRuntime for GlobalProxyMonitoringRuntime {
    fn read_system_proxy(&self) -> Result<SystemProxySnapshot, BoxError> {
        current_system_proxy().map_err(|error| Box::new(error) as BoxError)
    }

    fn apply_system_proxy(&self, snapshot: SystemProxySnapshot) -> Result<NetworkUpdate, BoxError> {
        apply_system_proxy_snapshot(snapshot).map_err(|error| Box::new(error) as BoxError)
    }
}

struct ProxyMonitoringHandle {
    shutdown: tokio::sync::watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
}

/// 进程级系统代理轮询服务。
pub struct ProxyMonitoringService {
    runtime: Arc<dyn ProxyMonitoringRuntime>,
    handle: tokio::sync::Mutex<Option<ProxyMonitoringHandle>>,
}

impl ProxyMonitoringService {
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(GlobalProxyMonitoringRuntime),
            handle: tokio::sync::Mutex::new(None),
        }
    }

    /// 启动唯一轮询任务；已运行时返回 `false`。
    pub async fn start(self: &Arc<Self>) -> bool {
        self.start_with_interval(SYSTEM_PROXY_POLL_INTERVAL).await
    }

    /// 停止轮询任务并等待退出；未运行时返回 `false`。
    pub async fn stop(&self) -> bool {
        let handle = self.handle.lock().await.take();
        let Some(handle) = handle else {
            return false;
        };
        let _ = handle.shutdown.send(true);
        let mut task = handle.task;
        match tokio::time::timeout(SYSTEM_PROXY_STOP_TIMEOUT, &mut task).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                tracing::warn!(
                    target: "sealantern.application.proxy_monitoring",
                    error = %error,
                    "system proxy monitoring task did not stop cleanly"
                );
            }
            Err(_) => {
                // spawn_blocking cannot cancel an already-running OS call. Cancel the
                // monitor task so it no longer retains service state, and let the
                // blocking worker finish independently.
                task.abort();
                let _ = task.await;
                tracing::warn!(
                    target: "sealantern.application.proxy_monitoring",
                    timeout_seconds = SYSTEM_PROXY_STOP_TIMEOUT.as_secs(),
                    "system proxy monitoring task stop timed out"
                );
            }
        }
        true
    }

    async fn start_with_interval(self: &Arc<Self>, interval: Duration) -> bool {
        let mut handle = self.handle.lock().await;
        if handle.is_some() {
            return false;
        }

        let (shutdown, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let service = Arc::downgrade(self);
        let task = tokio::spawn(async move {
            let mut previous = None;
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        let Some(service) = service.upgrade() else {
                            break;
                        };
                        let runtime = service.runtime.clone();
                        let refresh = tokio::task::spawn_blocking(move || {
                            let mut previous = previous;
                            let result = refresh_proxy(runtime.as_ref(), &mut previous);
                            (previous, result)
                        })
                        .await;
                        match refresh {
                            Ok((next_previous, result)) => {
                                previous = next_previous;
                                report_refresh_result(result);
                            }
                            Err(error) => {
                                // 阻塞任务异常退出时无法取回被移动的快照；下一轮从
                                // 未知状态重新应用当前快照，避免错误地跳过刷新。
                                previous = None;
                                tracing::warn!(
                                    target: "sealantern.application.proxy_monitoring",
                                    stage = "task",
                                    error = %error,
                                    "system proxy refresh task failed"
                                );
                            }
                        }
                    }
                    changed = shutdown_rx.changed() => {
                        if changed.is_err() || *shutdown_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });
        handle.replace(ProxyMonitoringHandle { shutdown, task });
        true
    }

    #[cfg(test)]
    fn with_runtime(runtime: Arc<dyn ProxyMonitoringRuntime>) -> Self {
        Self {
            runtime,
            handle: tokio::sync::Mutex::new(None),
        }
    }
}

fn report_refresh_result(result: Result<Option<NetworkUpdate>, ProxyRefreshError>) {
    match result {
        Ok(Some(update)) => tracing::info!(
            target: "sealantern.application.proxy_monitoring",
            client_rebuilt = update.client_rebuilt,
            state_revision = update.state_revision,
            client_revision = update.client_revision,
            "system proxy snapshot applied"
        ),
        Ok(None) => {}
        Err(error) => tracing::warn!(
            target: "sealantern.application.proxy_monitoring",
            stage = error.stage(),
            error = %error,
            "system proxy refresh failed; keeping previous runtime"
        ),
    }
}

impl Default for ProxyMonitoringService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum ProxyRefreshError {
    Read(BoxError),
    Apply(BoxError),
}

impl ProxyRefreshError {
    const fn stage(&self) -> &'static str {
        match self {
            Self::Read(_) => "read",
            Self::Apply(_) => "apply",
        }
    }
}

impl fmt::Display for ProxyRefreshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) | Self::Apply(error) => error.fmt(formatter),
        }
    }
}

fn refresh_proxy(
    runtime: &dyn ProxyMonitoringRuntime,
    previous: &mut Option<SystemProxySnapshot>,
) -> Result<Option<NetworkUpdate>, ProxyRefreshError> {
    let snapshot = runtime
        .read_system_proxy()
        .map_err(ProxyRefreshError::Read)?;
    if previous.as_ref() == Some(&snapshot) {
        return Ok(None);
    }
    let update = runtime
        .apply_system_proxy(snapshot.clone())
        .map_err(ProxyRefreshError::Apply)?;
    *previous = Some(snapshot);
    Ok(Some(update))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct FakeRuntime {
        reads: Mutex<VecDeque<Result<SystemProxySnapshot, &'static str>>>,
        apply_results: Mutex<VecDeque<Result<NetworkUpdate, &'static str>>>,
        applied: Mutex<Vec<SystemProxySnapshot>>,
        read_count: AtomicUsize,
    }

    impl FakeRuntime {
        fn with_reads(reads: impl IntoIterator<Item = SystemProxySnapshot>) -> Self {
            Self {
                reads: Mutex::new(reads.into_iter().map(Ok).collect()),
                apply_results: Mutex::new(VecDeque::new()),
                applied: Mutex::new(Vec::new()),
                read_count: AtomicUsize::new(0),
            }
        }
    }

    impl ProxyMonitoringRuntime for FakeRuntime {
        fn read_system_proxy(&self) -> Result<SystemProxySnapshot, BoxError> {
            self.read_count.fetch_add(1, Ordering::Relaxed);
            self.reads
                .lock()
                .expect("测试读取队列锁不应污染")
                .pop_front()
                .unwrap_or_else(|| Ok(SystemProxySnapshot::direct()))
                .map_err(|message| Box::new(std::io::Error::other(message)) as BoxError)
        }

        fn apply_system_proxy(
            &self,
            snapshot: SystemProxySnapshot,
        ) -> Result<NetworkUpdate, BoxError> {
            self.applied
                .lock()
                .expect("测试应用记录锁不应污染")
                .push(snapshot);
            self.apply_results
                .lock()
                .expect("测试应用结果锁不应污染")
                .pop_front()
                .unwrap_or(Ok(NetworkUpdate {
                    client_rebuilt: true,
                    state_revision: 1,
                    client_revision: 1,
                }))
                .map_err(|message| Box::new(std::io::Error::other(message)) as BoxError)
        }
    }

    #[test]
    fn refresh_applies_only_changed_snapshots() {
        let proxy = SystemProxySnapshot::proxy("http://127.0.0.1:7897");
        let runtime = FakeRuntime::with_reads([proxy.clone(), proxy.clone()]);
        let mut previous = None;

        assert!(refresh_proxy(&runtime, &mut previous).unwrap().is_some());
        assert!(refresh_proxy(&runtime, &mut previous).unwrap().is_none());
        assert_eq!(runtime.applied.lock().unwrap().as_slice(), &[proxy]);
    }

    #[test]
    fn failed_apply_keeps_previous_snapshot_for_retry() {
        let proxy = SystemProxySnapshot::proxy("http://127.0.0.1:7897");
        let runtime = FakeRuntime::with_reads([proxy.clone(), proxy.clone()]);
        runtime
            .apply_results
            .lock()
            .unwrap()
            .push_back(Err("synthetic apply failure"));
        let mut previous = None;

        assert!(matches!(
            refresh_proxy(&runtime, &mut previous),
            Err(ProxyRefreshError::Apply(_))
        ));
        assert!(previous.is_none());
        assert!(refresh_proxy(&runtime, &mut previous).unwrap().is_some());
        assert_eq!(runtime.applied.lock().unwrap().len(), 2);
    }

    #[test]
    fn failed_read_keeps_previous_snapshot_and_retries() {
        let proxy = SystemProxySnapshot::proxy("http://127.0.0.1:7897");
        let runtime = FakeRuntime {
            reads: Mutex::new(VecDeque::from([
                Ok(proxy.clone()),
                Err("synthetic read failure"),
                Ok(SystemProxySnapshot::direct()),
            ])),
            apply_results: Mutex::new(VecDeque::new()),
            applied: Mutex::new(Vec::new()),
            read_count: AtomicUsize::new(0),
        };
        let mut previous = None;

        assert!(refresh_proxy(&runtime, &mut previous).unwrap().is_some());
        assert!(matches!(
            refresh_proxy(&runtime, &mut previous),
            Err(ProxyRefreshError::Read(_))
        ));
        assert_eq!(previous, Some(proxy));
        assert!(refresh_proxy(&runtime, &mut previous).unwrap().is_some());
        assert_eq!(previous, Some(SystemProxySnapshot::direct()));
        assert_eq!(runtime.applied.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn monitor_starts_once_and_stops_cleanly() {
        let runtime = Arc::new(FakeRuntime::with_reads([
            SystemProxySnapshot::direct(),
            SystemProxySnapshot::direct(),
        ]));
        let service = Arc::new(ProxyMonitoringService::with_runtime(runtime.clone()));

        assert!(service.start_with_interval(Duration::from_millis(5)).await);
        assert!(!service.start_with_interval(Duration::from_millis(5)).await);
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(service.stop().await);
        assert!(!service.stop().await);
        assert!(runtime.read_count.load(Ordering::Relaxed) > 0);
    }

    #[tokio::test]
    async fn monitor_task_does_not_hold_service_alive() {
        let runtime = Arc::new(FakeRuntime::with_reads([]));
        let service = Arc::new(ProxyMonitoringService::with_runtime(runtime));

        assert!(service.start_with_interval(Duration::from_secs(60)).await);
        assert_eq!(Arc::strong_count(&service), 1);
        assert!(service.stop().await);
    }
}
