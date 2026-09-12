//! 在线隧道服务实现。
//!
//! 底层使用 sculk 的托管 [`sculk::tunnel::TunnelService`]：隧道生命周期、状态
//! 快照与事件流由库统一维护，本模块只做三件事：
//!
//! 1. 把应用的启动请求翻译成 sculk 的 Host / Join 参数；
//! 2. 把 sculk 的状态与事件映射为应用自己的 [`TunnelStatus`] / [`TunnelEvent`]；
//! 3. 把 sculk 的订阅模型（`watch` 状态 + `broadcast` 事件）扇出为单一只事件
//!    广播，供应用层按需订阅。
//!
//! 一个服务实例同一时刻至多持有一条活动隧道。

use std::num::NonZeroU16;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use sculk::ErrorCategory as SculkErrorCategory;
use sculk::tunnel::{
    AccessToken, ConnectionSnapshot, HostConfig, HostOptions, JoinConfig, JoinOptions, JoinUri,
    LocalPort, RelayUrl, SecretKey, ServiceId, TunnelEvent as SculkEvent,
    TunnelPhase as SculkPhase, TunnelService as SculkService,
    TunnelServiceError as SculkServiceError, TunnelStatus as SculkStatus,
    TunnelUpdate as SculkUpdate,
};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::model::{
    HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError, TunnelConnection, TunnelErrorCategory,
    TunnelEvent, TunnelIdentity, TunnelMode, TunnelPhase, TunnelStatus, TunnelTicket,
};

/// 应用事件广播的缓冲区容量。
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// 等待隧道进入 `Active` 的上限；超过则判定启动失败。
const START_TIMEOUT: Duration = Duration::from_secs(30);

/// SeaLantern 在线隧道服务。
#[derive(Clone)]
pub struct OnlineTunnelService {
    inner: SculkService,
    events: Arc<EventFanout>,
}

/// 把 sculk 的事件流扇出为应用层的事件广播。
struct EventFanout {
    sender: broadcast::Sender<TunnelEvent>,
    task: StdMutex<Option<JoinHandle<()>>>,
}

impl Default for OnlineTunnelService {
    fn default() -> Self {
        Self::new()
    }
}

impl OnlineTunnelService {
    /// 创建空闲的隧道服务。
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            inner: SculkService::new(),
            events: Arc::new(EventFanout { sender, task: StdMutex::new(None) }),
        }
    }

    /// 以 Host 角色开启隧道，返回隧道就绪后的状态快照。
    ///
    /// sculk 的 `start_host` 只保证「启动任务已被接受」，隧道就绪（含 Join URI）
    /// 是异步完成的；这里等状态进入 `Active` 再返回，避免调用方拿到尚未就绪的
    /// 快照（例如票据仍为空）。
    pub async fn host(
        &self,
        request: HostTunnelRequest,
    ) -> Result<TunnelStatus, OnlineTunnelError> {
        let options = build_host_options(&request)?;
        self.inner
            .start_host(options)
            .await
            .map_err(map_service_error)?;
        // 先建立事件扇出，避免就绪过程中产生的事件丢失。
        self.restart_fanout();
        match self.wait_until_active().await {
            Ok(status) => Ok(status),
            Err(error) => {
                self.stop_fanout();
                Err(error)
            }
        }
    }

    /// 以 Join 角色加入票据指定的隧道，返回隧道就绪后的状态快照。
    pub async fn join(
        &self,
        request: JoinTunnelRequest,
    ) -> Result<TunnelStatus, OnlineTunnelError> {
        let options = build_join_options(&request)?;
        self.inner
            .start_join(options)
            .await
            .map_err(map_service_error)?;
        self.restart_fanout();
        match self.wait_until_active().await {
            Ok(status) => Ok(status),
            Err(error) => {
                self.stop_fanout();
                Err(error)
            }
        }
    }

    /// 等待隧道进入 `Active` 并返回就绪状态。
    ///
    /// 订阅 sculk 的状态流而非轮询；超时或提前回到 `Idle` 都视为启动失败。
    async fn wait_until_active(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        let mut updates = self.inner.subscribe();
        let deadline = tokio::time::Instant::now() + START_TIMEOUT;

        loop {
            let current = self.inner.status();
            match current.state.phase {
                SculkPhase::Active => return Ok(map_status(&current)),
                SculkPhase::Idle => {
                    return Err(OnlineTunnelError::provider(
                        "start tunnel",
                        "tunnel became idle before becoming active",
                    ));
                }
                SculkPhase::Starting | SculkPhase::Stopping => {}
            }

            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(OnlineTunnelError::provider(
                    "start tunnel",
                    "timed out waiting for the tunnel to become active",
                ));
            }

            match tokio::time::timeout(remaining, updates.recv()).await {
                Ok(Some(SculkUpdate::Status(status))) => {
                    if status.state.phase == SculkPhase::Active {
                        return Ok(map_status(&status));
                    }
                }
                // 过程事件（以及 `non_exhaustive` 后续新增的变体）：继续等待就绪。
                Ok(Some(_)) => {}
                Ok(None) => {
                    return Err(OnlineTunnelError::provider(
                        "start tunnel",
                        "tunnel status stream closed",
                    ));
                }
                Err(_) => {
                    return Err(OnlineTunnelError::provider(
                        "start tunnel",
                        "timed out waiting for the tunnel to become active",
                    ));
                }
            }
        }
    }

    /// 停止当前活动隧道；无活动隧道时返回 [`OnlineTunnelError::NotRunning`]。
    pub async fn stop(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        self.inner.stop().await.map_err(map_service_error)?;
        self.stop_fanout();
        Ok(TunnelStatus::idle())
    }

    /// 查询当前隧道状态。
    pub async fn status(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        Ok(map_status(&self.inner.status()))
    }

    /// 订阅隧道事件；无活动隧道时返回 [`OnlineTunnelError::NotRunning`]。
    pub async fn subscribe(&self) -> Result<broadcast::Receiver<TunnelEvent>, OnlineTunnelError> {
        if self.inner.status().state.phase == SculkPhase::Idle {
            return Err(OnlineTunnelError::NotRunning);
        }
        Ok(self.events.sender.subscribe())
    }

    /// 幂等关闭服务持有的隧道与事件转发任务。
    pub async fn shutdown(&self) -> Result<(), OnlineTunnelError> {
        self.stop_fanout();
        self.inner.shutdown().await.map_err(map_service_error)
    }

    /// 重新订阅 sculk 事件并启动扇出任务。
    fn restart_fanout(&self) {
        self.stop_fanout();
        let mut updates = self.inner.subscribe();
        let sender = self.events.sender.clone();
        let task = tokio::spawn(async move {
            while let Some(update) = updates.recv().await {
                if let SculkUpdate::Event(event) = update {
                    let _ = sender.send(map_event(event));
                }
            }
        });
        if let Ok(mut slot) = self.events.task.lock() {
            *slot = Some(task);
        }
    }

    /// 停止事件扇出任务。
    fn stop_fanout(&self) {
        let task = self
            .events
            .task
            .lock()
            .ok()
            .and_then(|mut slot| slot.take());
        if let Some(task) = task {
            task.abort();
        }
    }
}

/// 把应用请求翻译为 sculk 的 Host 启动参数。
///
/// sculk 0.6 起改用 `ServiceId` + `AccessToken` 认证（凭证内嵌在 Join URI 中），
/// 不再有独立密码；请求里的 `password` 字段暂时保留但被忽略。
fn build_host_options(request: &HostTunnelRequest) -> Result<HostOptions, OnlineTunnelError> {
    let _ = request.password.as_deref();

    Ok(HostOptions::new(request.minecraft_port)
        .service_id(ServiceId::generate())
        .token(AccessToken::generate())
        .secret_key(request.identity.as_ref().map(identity_to_secret_key))
        .relay_url(parse_relay_url(request.relay_url.as_deref())?)
        .config(HostConfig::default().max_players(request.max_players)))
}

/// 把应用请求翻译为 sculk 的 Join 启动参数。
fn build_join_options(request: &JoinTunnelRequest) -> Result<JoinOptions, OnlineTunnelError> {
    let join_uri: JoinUri = request
        .ticket
        .as_str()
        .parse()
        .map_err(|error| OnlineTunnelError::provider("parse tunnel ticket", error))?;
    let local_port = NonZeroU16::new(request.local_port).map_or(LocalPort::Auto, LocalPort::Fixed);

    Ok(JoinOptions::new(join_uri)
        .local_port(local_port)
        .config(JoinConfig::default()))
}

fn parse_relay_url(value: Option<&str>) -> Result<Option<RelayUrl>, OnlineTunnelError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    value
        .parse::<RelayUrl>()
        .map(Some)
        .map_err(|error| OnlineTunnelError::provider("parse relay URL", error))
}

fn identity_to_secret_key(identity: &TunnelIdentity) -> SecretKey {
    SecretKey::from_bytes(identity.as_bytes())
}

/// sculk 状态快照 → 应用状态快照。
fn map_status(status: &SculkStatus) -> TunnelStatus {
    let active = status.state.phase == SculkPhase::Active;
    let mode = status.state.mode.map(map_mode);
    let ticket = status
        .state
        .join_uri
        .as_ref()
        .and_then(|uri| uri.expose_secret_uri().ok())
        .map(TunnelTicket::from_provider);
    let connections = status.connections.iter().map(map_connection).collect();
    TunnelStatus {
        active,
        phase: map_phase(status.state.phase),
        mode,
        ticket,
        connections,
        last_error: status.last_error.map(map_error_category),
    }
}

fn map_phase(phase: SculkPhase) -> TunnelPhase {
    match phase {
        SculkPhase::Idle => TunnelPhase::Idle,
        SculkPhase::Starting => TunnelPhase::Starting,
        SculkPhase::Active => TunnelPhase::Active,
        SculkPhase::Stopping => TunnelPhase::Stopping,
    }
}

fn map_error_category(category: SculkErrorCategory) -> TunnelErrorCategory {
    match category {
        SculkErrorCategory::InvalidJoinUri => TunnelErrorCategory::InvalidJoinUri,
        SculkErrorCategory::InvalidEndpoint => TunnelErrorCategory::InvalidEndpoint,
        SculkErrorCategory::AuthorizationDenied => TunnelErrorCategory::AuthorizationDenied,
        SculkErrorCategory::HostUnreachable => TunnelErrorCategory::HostUnreachable,
        SculkErrorCategory::TargetUnavailable => TunnelErrorCategory::TargetUnavailable,
        SculkErrorCategory::LocalPortUnavailable => TunnelErrorCategory::LocalPortUnavailable,
        SculkErrorCategory::IdentityUnavailable => TunnelErrorCategory::IdentityUnavailable,
        SculkErrorCategory::OperationConflict => TunnelErrorCategory::OperationConflict,
        SculkErrorCategory::ResourceLimit => TunnelErrorCategory::ResourceLimit,
        SculkErrorCategory::InvalidConfiguration => TunnelErrorCategory::InvalidConfiguration,
        SculkErrorCategory::Internal => TunnelErrorCategory::Internal,
        // sculk 的分类标记为 non_exhaustive：新增分类在产品层按 Internal 兜底。
        _ => TunnelErrorCategory::Internal,
    }
}

fn map_mode(mode: sculk::tunnel::TunnelMode) -> TunnelMode {
    match mode {
        sculk::tunnel::TunnelMode::Host => TunnelMode::Host,
        sculk::tunnel::TunnelMode::Join => TunnelMode::Join,
    }
}

fn map_connection(connection: &ConnectionSnapshot) -> TunnelConnection {
    TunnelConnection {
        remote_id: connection.remote_id.to_string(),
        is_relay: connection.is_relay,
        rtt_ms: connection.rtt_ms,
        tx_bytes: connection.tx_bytes,
        rx_bytes: connection.rx_bytes,
        alive: connection.alive,
        elapsed_ms: connection.elapsed.as_millis() as u64,
    }
}

/// sculk 事件 → 应用事件。
fn map_event(event: SculkEvent) -> TunnelEvent {
    match event {
        SculkEvent::PlayerJoined { id } => TunnelEvent::PlayerJoined { remote_id: id.to_string() },
        SculkEvent::PlayerLeft { id, reason } => {
            TunnelEvent::PlayerLeft { remote_id: id.to_string(), reason }
        }
        SculkEvent::Connected => TunnelEvent::Connected,
        SculkEvent::Disconnected { reason } => TunnelEvent::Disconnected { reason },
        SculkEvent::PathChanged { remote_id, is_relay, rtt_ms } => TunnelEvent::PathChanged {
            remote_id: remote_id.to_string(),
            is_relay,
            rtt_ms,
        },
        SculkEvent::Reconnecting { attempt } => TunnelEvent::Reconnecting { attempt },
        SculkEvent::Reconnected => TunnelEvent::Reconnected,
        SculkEvent::AuthFailed { id } => {
            TunnelEvent::AuthenticationFailed { remote_id: id.to_string() }
        }
        SculkEvent::PlayerRejected { id, reason } => {
            TunnelEvent::PlayerRejected { remote_id: id.to_string(), reason }
        }
        SculkEvent::TokenRotated => TunnelEvent::TokenRotated,
        SculkEvent::Error { category, message } => TunnelEvent::Error {
            category: map_error_category(category),
            message,
        },
        other => TunnelEvent::ProviderMessage { message: format!("{other:?}") },
    }
}

/// sculk 服务错误 → 应用错误。
fn map_service_error(error: SculkServiceError) -> OnlineTunnelError {
    match error {
        SculkServiceError::Busy => OnlineTunnelError::Busy,
        SculkServiceError::NotRunning => OnlineTunnelError::NotRunning,
        SculkServiceError::InvalidPort => {
            OnlineTunnelError::invalid_request("minecraft port must be non-zero")
        }
        SculkServiceError::InvalidMaxPlayers => {
            OnlineTunnelError::invalid_request("max players must be greater than zero")
        }
        other => OnlineTunnelError::provider("operate tunnel", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_new_service_is_idle() {
        let service = OnlineTunnelService::new();

        assert_eq!(service.status().await.expect("idle status"), TunnelStatus::idle());
    }

    #[tokio::test]
    async fn idle_service_rejects_stop_and_subscription() {
        let service = OnlineTunnelService::new();

        assert_eq!(service.stop().await, Err(OnlineTunnelError::NotRunning));
        assert!(matches!(service.subscribe().await, Err(OnlineTunnelError::NotRunning)));
    }

    #[tokio::test]
    async fn shutdown_is_idempotent_while_idle() {
        let service = OnlineTunnelService::new();

        service
            .shutdown()
            .await
            .expect("idle shutdown must succeed");
    }

    #[test]
    fn join_options_reject_a_malformed_ticket() {
        let request = JoinTunnelRequest {
            ticket: TunnelTicket::from_provider("not-a-ticket"),
            local_port: 30000,
            password: None,
            max_retries: None,
        };

        assert!(build_join_options(&request).is_err());
    }
}
