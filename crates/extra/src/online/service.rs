use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};

use super::sculk::{self, ActiveTunnel};
use super::{HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError, TunnelEvent, TunnelStatus};

/// SeaLantern 在线隧道服务。
///
/// 一个服务实例最多持有一条活动隧道。调用方通过实例生命周期管理隧道，避免旧实现中
/// 进程级全局状态导致的测试干扰和宿主重复初始化问题。
#[derive(Clone, Default)]
pub struct OnlineTunnelService {
    state: Arc<Mutex<ServiceState>>,
}

enum ServiceState {
    Idle,
    Starting,
    Active(ActiveTunnel),
}

impl Default for ServiceState {
    fn default() -> Self {
        Self::Idle
    }
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
        let active = {
            let mut state = self.state.lock().await;
            match std::mem::replace(&mut *state, ServiceState::Idle) {
                ServiceState::Active(active) => active,
                ServiceState::Idle => return Err(OnlineTunnelError::NotRunning),
                ServiceState::Starting => {
                    *state = ServiceState::Starting;
                    return Err(OnlineTunnelError::Busy);
                }
            }
        };

        let mode = active.mode;
        active.close().await;
        crate::observability::online_tunnel_stopped(mode.as_str());
        Ok(TunnelStatus::idle())
    }

    pub async fn status(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        let state = self.state.lock().await;
        match &*state {
            ServiceState::Idle => Ok(TunnelStatus::idle()),
            ServiceState::Starting => Err(OnlineTunnelError::Busy),
            ServiceState::Active(active) => active.status(),
        }
    }

    pub async fn subscribe(&self) -> Result<broadcast::Receiver<TunnelEvent>, OnlineTunnelError> {
        let state = self.state.lock().await;
        match &*state {
            ServiceState::Active(active) => Ok(active.subscribe()),
            ServiceState::Idle => Err(OnlineTunnelError::NotRunning),
            ServiceState::Starting => Err(OnlineTunnelError::Busy),
        }
    }

    async fn begin_start(&self) -> Result<(), OnlineTunnelError> {
        let mut state = self.state.lock().await;
        match &*state {
            ServiceState::Idle => {
                *state = ServiceState::Starting;
                Ok(())
            }
            ServiceState::Starting | ServiceState::Active(_) => Err(OnlineTunnelError::Busy),
        }
    }

    async fn reset_starting(&self) {
        let mut state = self.state.lock().await;
        if matches!(*state, ServiceState::Starting) {
            *state = ServiceState::Idle;
        }
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
        if !matches!(*state, ServiceState::Starting) {
            drop(state);
            active.close().await;
            return Err(OnlineTunnelError::Busy);
        }
        *state = ServiceState::Active(active);
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
}
