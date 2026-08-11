//! 在线隧道服务实现。
//!
//! 实现 [`sealantern_interface::OnlineTunnelService`] 能力端口，包装
//! `extra` 的在线隧道实现（[`ExtraOnlineTunnelService`]），向宿主提供
//! 开服（host）/ 联机（join）/ 停止 / 状态查询 / 事件订阅 / 关闭能力。
//!
//! 本层只做两件事：把接口请求转换为 `extra` 的请求模型，并把 `extra`
//! 返回的状态、事件与错误映射回接口契约类型。
//!
//! 错误分层：底层 [`OnlineTunnelError`] 按语义映射为
//! [`OnlineTunnelServiceError`]：非法请求 → `InvalidInput`，隧道忙碌 →
//! `Busy`，未运行 → `NotRunning`，其余 provider 失败 → `OperationFailed`。

use async_trait::async_trait;
use sealantern_extra::online::{
    HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError,
    OnlineTunnelService as ExtraOnlineTunnelService, TunnelConnection, TunnelEvent, TunnelIdentity,
    TunnelMode, TunnelStatus, TunnelTicket,
};
use sealantern_interface::{
    OnlineTunnelConnection, OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest,
    OnlineTunnelMode, OnlineTunnelService, OnlineTunnelServiceError, OnlineTunnelStatus,
};
use tokio::sync::broadcast;

/// 基于 `extra` 在线隧道实现的在线隧道服务。
///
/// 内部持有 `extra` 层隧道服务句柄，对外提供接口契约视图；
/// 隧道会话由底层维护，本服务只做请求转换与结果映射。
#[derive(Clone, Default)]
pub struct CoreOnlineTunnelService {
    /// `extra` 层的隧道服务（实际连接与事件源）。
    inner: ExtraOnlineTunnelService,
}

#[async_trait]
impl OnlineTunnelService for CoreOnlineTunnelService {
    /// 以开服者身份建立在线隧道。
    ///
    /// 可选身份密钥先转为底层需要的字节形式；转换失败视为非法输入。
    async fn host(
        &self,
        request: OnlineTunnelHostRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        let identity = request
            .identity
            .map(|identity| {
                identity
                    .try_into()
                    .map(TunnelIdentity::from_bytes)
                    .map_err(|_| OnlineTunnelServiceError::InvalidInput)
            })
            .transpose()?;
        self.inner
            .host(HostTunnelRequest {
                minecraft_port: request.minecraft_port,
                password: request.password,
                max_players: request.max_players,
                relay_url: request.relay_url,
                identity,
            })
            .await
            .map(map_status)
            .map_err(map_error)
    }

    /// 以票据加入他人隧道。
    ///
    /// 票据解析失败直接以非法输入返回，不进入底层。
    async fn join(
        &self,
        request: OnlineTunnelJoinRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        let ticket = TunnelTicket::parse(request.ticket)
            .map_err(|_| OnlineTunnelServiceError::InvalidInput)?;
        self.inner
            .join(JoinTunnelRequest {
                ticket,
                local_port: request.local_port,
                password: request.password,
                max_retries: request.max_retries,
            })
            .await
            .map(map_status)
            .map_err(map_error)
    }

    /// 停止当前隧道。
    async fn stop(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        self.inner.stop().await.map(map_status).map_err(map_error)
    }

    /// 查询当前隧道状态快照。
    async fn status(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        self.inner.status().await.map(map_status).map_err(map_error)
    }

    /// 订阅隧道事件流。
    ///
    /// 底层事件模型与接口不同，这里起一个转发任务，把底层广播流的
    /// 每个事件映射为接口事件后重新广播；转发任务随源流关闭而结束。
    async fn subscribe(
        &self,
    ) -> Result<broadcast::Receiver<OnlineTunnelEvent>, OnlineTunnelServiceError> {
        let mut source = self.inner.subscribe().await.map_err(map_error)?;
        let (sender, receiver) = broadcast::channel(128);
        tokio::spawn(async move {
            while let Ok(event) = source.recv().await {
                let _ = sender.send(map_event(event));
            }
        });
        Ok(receiver)
    }

    /// 关闭隧道并释放底层资源。
    async fn shutdown(&self) -> Result<(), OnlineTunnelServiceError> {
        self.inner.shutdown().await.map_err(map_error)
    }
}

/// 将底层隧道错误按语义映射为接口契约错误。
fn map_error(error: OnlineTunnelError) -> OnlineTunnelServiceError {
    match error {
        OnlineTunnelError::InvalidRequest { .. } => OnlineTunnelServiceError::InvalidInput,
        OnlineTunnelError::Busy => OnlineTunnelServiceError::Busy,
        OnlineTunnelError::NotRunning => OnlineTunnelServiceError::NotRunning,
        OnlineTunnelError::Provider { .. } => OnlineTunnelServiceError::OperationFailed,
    }
}

/// 将底层隧道角色映射为接口角色。
fn map_mode(value: TunnelMode) -> OnlineTunnelMode {
    match value {
        TunnelMode::Host => OnlineTunnelMode::Host,
        TunnelMode::Join => OnlineTunnelMode::Join,
    }
}

/// 将底层对端连接快照映射为接口连接快照。
fn map_connection(value: TunnelConnection) -> OnlineTunnelConnection {
    OnlineTunnelConnection {
        remote_id: value.remote_id,
        is_relay: value.is_relay,
        rtt_ms: value.rtt_ms,
        tx_bytes: value.tx_bytes,
        rx_bytes: value.rx_bytes,
        alive: value.alive,
        elapsed_ms: value.elapsed_ms,
    }
}

/// 将底层隧道状态快照映射为接口状态快照。
fn map_status(value: TunnelStatus) -> OnlineTunnelStatus {
    OnlineTunnelStatus {
        active: value.active,
        mode: value.mode.map(map_mode),
        ticket: value.ticket.map(|ticket| ticket.as_str().to_owned()),
        connections: value.connections.into_iter().map(map_connection).collect(),
    }
}

/// 将底层隧道事件映射为接口事件。
fn map_event(value: TunnelEvent) -> OnlineTunnelEvent {
    match value {
        TunnelEvent::PlayerJoined { remote_id } => OnlineTunnelEvent::PlayerJoined { remote_id },
        TunnelEvent::PlayerLeft { remote_id, reason } => {
            OnlineTunnelEvent::PlayerLeft { remote_id, reason }
        }
        TunnelEvent::Connected => OnlineTunnelEvent::Connected,
        TunnelEvent::Disconnected { reason } => OnlineTunnelEvent::Disconnected { reason },
        TunnelEvent::PathChanged { remote_id, is_relay, rtt_ms } => {
            OnlineTunnelEvent::PathChanged { remote_id, is_relay, rtt_ms }
        }
        TunnelEvent::Reconnecting { attempt } => OnlineTunnelEvent::Reconnecting { attempt },
        TunnelEvent::Reconnected => OnlineTunnelEvent::Reconnected,
        TunnelEvent::AuthenticationFailed { remote_id } => {
            OnlineTunnelEvent::AuthenticationFailed { remote_id }
        }
        TunnelEvent::PlayerRejected { remote_id, reason } => {
            OnlineTunnelEvent::PlayerRejected { remote_id, reason }
        }
        TunnelEvent::Error { message } => OnlineTunnelEvent::Error { message },
        TunnelEvent::ProviderMessage { message } => OnlineTunnelEvent::ProviderMessage { message },
    }
}
