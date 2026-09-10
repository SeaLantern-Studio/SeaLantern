use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};

use super::sculk::{self, ActiveTunnel};
use super::{HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError, TunnelEvent, TunnelStatus};

/// SeaLantern 在线隧道服务。
///
/// 一个服务实例最多持有一条活动隧道。调用方通过实例生命周期管理隧道，避免旧实现中
/// 进程级全局状态导致的测试干扰和宿主重复初始化问题。
#[derive(Clone)]
pub struct OnlineTunnelService {
    state: Arc<Mutex<ServiceState>>,
    state_changed: Arc<tokio::sync::Notify>,
}

impl Default for OnlineTunnelService {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(ServiceState::Idle)),
            state_changed: Arc::new(tokio::sync::Notify::new()),
        }
    }
}

#[derive(Default)]
enum ServiceState {
    #[default]
    Idle,
    Starting,
    Stopping,
    Active(ActiveTunnel),
}

impl OnlineTunnelService {
    pub async fn host(
        &self,
        request: HostTunnelRequest,
    ) -> Result<TunnelStatus, OnlineTunnelError> {
        self.begin_start().await?;
        let active = match sculk::host(request).await {
            Ok(active) => active,
            Err(error) => {
                crate::observability::online_tunnel_failed("start host tunnel", &error);
                self.reset_starting().await;
                return Err(error);
            }
        };
        self.finish_start(active).await
    }

    pub async fn join(
        &self,
        request: JoinTunnelRequest,
    ) -> Result<TunnelStatus, OnlineTunnelError> {
        self.begin_start().await?;
        let active = match sculk::join(request).await {
            Ok(active) => active,
            Err(error) => {
                crate::observability::online_tunnel_failed("join host tunnel", &error);
                self.reset_starting().await;
                return Err(error);
            }
        };
        self.finish_start(active).await
    }

    pub async fn stop(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        let active = self.begin_stop().await?;
        self.finish_stop(active).await;
        Ok(TunnelStatus::idle())
    }

    /// 幂等关闭服务持有的活动隧道，供宿主退出流程调用。
    pub async fn shutdown(&self) -> Result<(), OnlineTunnelError> {
        loop {
            // 在调用 begin_stop 前创建通知 future，避免状态刚从 Starting/
            // Stopping 转换时错过唤醒。
            let notified = self.state_changed.notified();
            match self.begin_stop().await {
                Ok(active) => {
                    self.finish_stop(active).await;
                    return Ok(());
                }
                Err(OnlineTunnelError::NotRunning) => return Ok(()),
                Err(OnlineTunnelError::Busy) => notified.await,
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn status(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        let state = self.state.lock().await;
        match &*state {
            ServiceState::Idle => Ok(TunnelStatus::idle()),
            ServiceState::Starting | ServiceState::Stopping => Err(OnlineTunnelError::Busy),
            ServiceState::Active(active) => active.status(),
        }
    }

    pub async fn subscribe(&self) -> Result<broadcast::Receiver<TunnelEvent>, OnlineTunnelError> {
        let state = self.state.lock().await;
        match &*state {
            ServiceState::Active(active) => Ok(active.subscribe()),
            ServiceState::Idle => Err(OnlineTunnelError::NotRunning),
            ServiceState::Starting | ServiceState::Stopping => Err(OnlineTunnelError::Busy),
        }
    }

    async fn begin_start(&self) -> Result<(), OnlineTunnelError> {
        let mut state = self.state.lock().await;
        match &*state {
            ServiceState::Idle => {
                *state = ServiceState::Starting;
                Ok(())
            }
            ServiceState::Starting | ServiceState::Stopping | ServiceState::Active(_) => {
                Err(OnlineTunnelError::Busy)
            }
        }
    }

    async fn reset_starting(&self) {
        let mut state = self.state.lock().await;
        if matches!(&*state, ServiceState::Starting) {
            *state = ServiceState::Idle;
            self.state_changed.notify_waiters();
        }
    }

    async fn begin_stop(&self) -> Result<ActiveTunnel, OnlineTunnelError> {
        let mut state = self.state.lock().await;
        match std::mem::replace(&mut *state, ServiceState::Stopping) {
            ServiceState::Active(active) => Ok(active),
            ServiceState::Idle => {
                *state = ServiceState::Idle;
                Err(OnlineTunnelError::NotRunning)
            }
            ServiceState::Starting => {
                *state = ServiceState::Starting;
                Err(OnlineTunnelError::Busy)
            }
            ServiceState::Stopping => {
                *state = ServiceState::Stopping;
                Err(OnlineTunnelError::Busy)
            }
        }
    }

    async fn finish_stop(&self, active: ActiveTunnel) {
        let mode = active.mode;
        active.close().await;

        let mut state = self.state.lock().await;
        if matches!(&*state, ServiceState::Stopping) {
            *state = ServiceState::Idle;
            self.state_changed.notify_waiters();
        }
        crate::observability::online_tunnel_stopped(mode.as_str());
    }

    async fn finish_start(&self, active: ActiveTunnel) -> Result<TunnelStatus, OnlineTunnelError> {
        let status = match active.status() {
            Ok(status) => status,
            Err(error) => {
                active.close().await;
                crate::observability::online_tunnel_failed("read initial tunnel status", &error);
                self.reset_starting().await;
                return Err(error);
            }
        };
        let mode = active.mode;
        let mut state = self.state.lock().await;
        if !matches!(&*state, ServiceState::Starting) {
            drop(state);
            active.close().await;
            return Err(OnlineTunnelError::Busy);
        }
        *state = ServiceState::Active(active);
        self.state_changed.notify_waiters();
        crate::observability::online_tunnel_started(mode.as_str());
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_new_service_is_idle() {
        let service = OnlineTunnelService::default();

        assert_eq!(service.status().await.expect("idle status"), TunnelStatus::idle());
    }

    #[tokio::test]
    async fn idle_service_rejects_stop_and_subscription() {
        let service = OnlineTunnelService::default();

        assert_eq!(service.stop().await, Err(OnlineTunnelError::NotRunning));
        assert!(matches!(service.subscribe().await, Err(OnlineTunnelError::NotRunning)));
    }

    #[tokio::test]
    async fn stopping_state_blocks_new_operations() {
        let service = OnlineTunnelService::default();
        *service.state.lock().await = ServiceState::Stopping;

        assert_eq!(service.begin_start().await, Err(OnlineTunnelError::Busy));
        assert!(matches!(service.begin_stop().await, Err(OnlineTunnelError::Busy)));
        assert_eq!(service.status().await, Err(OnlineTunnelError::Busy));
        assert!(matches!(service.subscribe().await, Err(OnlineTunnelError::Busy)));
    }

    #[tokio::test]
    async fn shutdown_is_idempotent_while_idle() {
        let service = OnlineTunnelService::default();

        service
            .shutdown()
            .await
            .expect("idle shutdown must succeed");
    }

    #[tokio::test]
    async fn shutdown_waits_for_transient_state_to_finish() {
        let service = OnlineTunnelService::default();
        *service.state.lock().await = ServiceState::Starting;

        let shutdown_service = service.clone();
        let mut shutdown = tokio::spawn(async move { shutdown_service.shutdown().await });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), &mut shutdown)
                .await
                .is_err()
        );

        *service.state.lock().await = ServiceState::Idle;
        service.state_changed.notify_waiters();
        shutdown
            .await
            .expect("shutdown task should finish")
            .expect("transient state shutdown should succeed");
    }
}
