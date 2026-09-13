//! 在线隧道服务实现。
//!
//! 按角色分工使用 sculk 的两套托管 API：Host 走 `SculkNode`（只有 Node API 支持
//! `TokenRefreshPolicy` 与令牌状态持久化），Join 走 `TunnelService`。本模块只做
//! 请求翻译、状态/事件映射，以及把 sculk 的订阅模型扇出为单一事件广播。
//!
//! 一个服务实例同一时刻至多持有一条活动隧道。

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use sculk::ErrorCategory as SculkErrorCategory;
use sculk::minecraft::lan::LanBroadcaster;
use sculk::persist::{self, HostState as PersistedHostState};
use sculk::tunnel::{
    ConnectionSnapshot, HostConfig, HostedServiceHandle, HostedServiceOptions, HostedServicePhase,
    HostedServiceStatus, JoinConfig, JoinOptions, JoinUri, LocalPort, NodeOptions, RelayUrl,
    SculkNode, SecretKey, ServiceId, TokenRefreshPolicy, TunnelEvent as SculkEvent,
    TunnelPhase as SculkPhase, TunnelService as SculkService,
    TunnelServiceError as SculkServiceError, TunnelStatus as SculkStatus,
    TunnelUpdate as SculkUpdate,
};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::model::{
    HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError, TunnelConnection, TunnelErrorCategory,
    TunnelEvent, TunnelIdentity, TunnelLinkLifetime, TunnelMode, TunnelPhase, TunnelStatus,
    TunnelTicket,
};

/// 应用事件广播的缓冲区容量。
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Host 节点身份密钥文件名；与 `host.state` 同目录。
const HOST_KEY_FILE: &str = "secret.key";

/// `PathChanged` 事件的发送节流；与 SeaLantern Connect 保持一致。
const TUNNEL_EVENT_DELAY: Duration = Duration::from_secs(1);

/// 局域网公告里的世界名称；本机 Minecraft 多人列表直接显示它。
const LAN_BROADCAST_NAME: &str = "Sea Lantern";

/// 用户侧分享链接前缀；内部 `JoinUri` 只在进入 sculk 前使用。
const SHARE_URL_PREFIX: &str = "https://ideaflash.cn/#v1/";
const JOIN_URI_PREFIX: &str = "sculk://join/v1/";

/// SeaLantern 在线隧道服务。
#[derive(Clone)]
pub struct OnlineTunnelService {
    join: SculkService,
    host: Arc<AsyncMutex<Option<HostSession>>>,
    events: Arc<EventFanout>,
    /// Host 启动的取消标记：`stop` 置位后，正在 bind 的 `host` 会在检查点放弃启动。
    host_cancel: Arc<AtomicBool>,
    /// Host 启动互斥：并发的 host 请求只有一个能走到发布。
    host_start: Arc<AsyncMutex<()>>,
    /// 加入方就绪后的局域网公告；本机 Minecraft 无需手动输入地址即可发现房间。
    lan_broadcast: Arc<StdMutex<Option<LanBroadcaster>>>,
    /// Host 令牌状态与稳定 `ServiceId` 的持久化路径。
    state_path: PathBuf,
}

/// 活动中的 Host 端会话。
struct HostSession {
    node: SculkNode,
    handle: HostedServiceHandle,
    /// Node API 不暴露逐连接指标，对端快照由事件流维护。
    peers: Arc<StdMutex<BTreeMap<String, PeerState>>>,
}

#[derive(Clone)]
struct PeerState {
    is_relay: bool,
    rtt_ms: u64,
    joined_at: Instant,
}

impl PeerState {
    fn new(joined_at: Instant) -> Self {
        Self { is_relay: false, rtt_ms: 0, joined_at }
    }
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
        Self::with_state_path(default_host_state_path())
    }

    /// 创建空闲的隧道服务，并指定 Host 状态文件位置。
    fn with_state_path(state_path: PathBuf) -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            join: SculkService::new(),
            host: Arc::new(AsyncMutex::new(None)),
            events: Arc::new(EventFanout { sender, task: StdMutex::new(None) }),
            host_cancel: Arc::new(AtomicBool::new(false)),
            host_start: Arc::new(AsyncMutex::new(())),
            lan_broadcast: Arc::new(StdMutex::new(None)),
            state_path,
        }
    }

    /// 以 Host 角色开启隧道，返回隧道就绪后的状态快照。
    ///
    /// 不预检本机是否已有 Minecraft 世界监听到该端口：建房与开世界是两件事，
    /// 世界稍后再开也应该允许先发布隧道。
    ///
    /// 启动期间**不持有 `host` 锁**：`stop` 需要能立刻置位取消标记并返回，
    /// 否则「取消连接」只能等整个建房流程跑完才生效。
    pub async fn host(
        &self,
        request: HostTunnelRequest,
    ) -> Result<TunnelStatus, OnlineTunnelError> {
        self.ensure_idle().await?;
        if request.minecraft_port == 0 {
            return Err(OnlineTunnelError::port_unavailable(
                "Minecraft port must be between 1 and 65535",
            ));
        }

        // 同一时刻只允许一次 Host 启动；guard 在函数结束时释放。
        let _starting = self
            .host_start
            .try_lock()
            .map_err(|_| OnlineTunnelError::Busy)?;
        self.host_cancel.store(false, Ordering::SeqCst);

        let node = SculkNode::bind(NodeOptions {
            secret_key: Some(self.host_secret_key(request.identity.as_ref())?),
            relay_url: parse_relay_url(request.relay_url.as_deref())?,
            ..NodeOptions::default()
        })
        .await
        .map_err(|error| OnlineTunnelError::provider("bind tunnel node", error))?;

        if self.host_cancel.load(Ordering::SeqCst) {
            node.close().await;
            return Err(cancelled_host());
        }

        // 复用上次的 ServiceId 与令牌，使 `never` / 定时轮换策略能跨重启生效。
        let saved = match persist::load_host_state(&self.state_path) {
            Ok(saved) => saved,
            Err(error) => {
                node.close().await;
                return Err(OnlineTunnelError::provider("load host state", error));
            }
        };
        let service_id = saved
            .as_ref()
            .map_or_else(ServiceId::generate, |state| state.service_id);
        // `UntilStopped` 每次发布都换新链接，因此忽略历史令牌。
        let token_state = match request.link_lifetime {
            TunnelLinkLifetime::UntilStopped => None,
            _ => saved.map(|state| state.token_state),
        };

        let options = HostedServiceOptions {
            service_id,
            target_addr: SocketAddr::from(([127, 0, 0, 1], request.minecraft_port)),
            token_state,
            token_refresh: token_refresh_policy(request.link_lifetime),
            config: HostConfig::new()
                .event_delay(TUNNEL_EVENT_DELAY)
                .max_players(request.max_players),
        };

        let handle = match node.start_service(options).await {
            Ok(handle) => handle,
            Err(error) => {
                node.close().await;
                return Err(OnlineTunnelError::provider("publish host service", error));
            }
        };

        if self.host_cancel.load(Ordering::SeqCst) {
            let _ = handle.stop().await;
            node.close().await;
            return Err(cancelled_host());
        }

        if let Err(error) = save_host_state(&self.state_path, &handle).await {
            let _ = handle.stop().await;
            node.close().await;
            return Err(error);
        }

        let peers = Arc::new(StdMutex::new(BTreeMap::new()));
        // 先订阅再读取状态：服务发布后立即建立扇出，避免开局事件丢失。
        if let Err(error) = self.restart_host_fanout(&handle, Arc::clone(&peers)).await {
            let _ = handle.stop().await;
            node.close().await;
            return Err(error);
        }
        let session = HostSession { node, handle, peers };
        let status = host_status(&session).await?;

        // 抢到锁后再确认一次取消：避免「取消」与「发布」擦身而过，
        // 让用户以为取消了、隧道却挂在那里。
        let mut slot = self.host.lock().await;
        if self.host_cancel.load(Ordering::SeqCst) {
            drop(slot);
            let _ = session.handle.stop().await;
            session.node.close().await;
            return Err(cancelled_host());
        }
        *slot = Some(session);
        Ok(status)
    }

    /// Host 节点身份密钥：调用方显式提供时优先，否则使用持久化密钥。
    ///
    /// 密钥决定端点 ID，而端点 ID 参与分享链接；不持久化的话每次重启都会换链接，
    /// `never` / 定时轮换策略复用的 service_id 与令牌就失去意义。
    fn host_secret_key(
        &self,
        identity: Option<&TunnelIdentity>,
    ) -> Result<SecretKey, OnlineTunnelError> {
        if let Some(identity) = identity {
            return Ok(identity_to_secret_key(identity));
        }
        persist::load_or_generate_key(&identity_key_path(&self.state_path))
            .map_err(|error| OnlineTunnelError::provider("load tunnel identity", error))
    }

    /// 以 Join 角色加入票据指定的隧道。
    ///
    /// sculk 的 `start_join` 只负责「接受启动任务」并立即返回，隧道就绪是异步的；
    /// 这里同步返回启动后的状态（通常为 `starting`），前端轮询观察就绪。
    pub async fn join(
        &self,
        request: JoinTunnelRequest,
    ) -> Result<TunnelStatus, OnlineTunnelError> {
        self.ensure_idle().await?;
        let options = build_join_options(&request)?;
        self.join
            .start_join(options)
            .await
            .map_err(map_service_error)?;
        // 先建立事件扇出，避免就绪过程中产生的事件丢失。
        self.restart_join_fanout();
        Ok(map_join_status(&self.join.status()))
    }

    /// 同一时刻只允许一条隧道；Host 与 Join 底层实现不同，需显式互斥。
    async fn ensure_idle(&self) -> Result<(), OnlineTunnelError> {
        let host_active = self.host.lock().await.is_some();
        // Host 启动期间还没写入会话槽，靠启动锁判断，避免 Join 插进来。
        let host_starting = self.host_start.try_lock().is_err();
        if host_active || host_starting || self.join.status().state.phase != SculkPhase::Idle {
            return Err(OnlineTunnelError::Busy);
        }
        Ok(())
    }

    /// 停止当前活动隧道，空闲时幂等返回空闲快照。
    ///
    /// sculk 的 `stop` 会打断正在启动的任务（`Starting` → abort 后台任务），
    /// 因此「取消连接」就是一次普通的 `stop`。
    pub async fn stop(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        // 先置位：Host 可能仍在 bind，它会在下一个检查点放弃启动。
        self.host_cancel.store(true, Ordering::SeqCst);

        let session = self.host.lock().await.take();
        if let Some(session) = session {
            self.stop_fanout();
            self.stop_lan_broadcast();
            let stop_result = session.handle.stop().await;
            session.node.close().await;
            stop_result.map_err(|error| OnlineTunnelError::provider("stop host service", error))?;
            return Ok(TunnelStatus::idle());
        }

        if self.join.status().state.phase == SculkPhase::Idle {
            return Ok(TunnelStatus::idle());
        }

        self.join.stop().await.map_err(map_service_error)?;
        self.stop_fanout();
        // 扇出任务可能没来得及处理最后一条 Idle 状态就被中止，这里兜底。
        self.stop_lan_broadcast();
        Ok(TunnelStatus::idle())
    }

    /// 停止局域网公告；没有公告时是空操作。
    fn stop_lan_broadcast(&self) {
        let Ok(mut current) = self.lan_broadcast.lock() else {
            return;
        };
        if let Some(broadcaster) = current.take() {
            let _ = broadcaster.stop();
        }
    }

    /// 查询当前隧道状态。
    pub async fn status(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        let slot = self.host.lock().await;
        if let Some(session) = slot.as_ref() {
            return host_status(session).await;
        }
        drop(slot);
        Ok(map_join_status(&self.join.status()))
    }

    /// 订阅隧道事件。
    ///
    /// 仅在隧道 `Active` 时可用：`Idle` 返回 [`OnlineTunnelError::NotRunning`]，
    /// 启动/停止等过渡阶段返回 [`OnlineTunnelError::Busy`]，避免调用方拿到
    /// 尚未就绪或已不再活动的事件流。
    pub async fn subscribe(&self) -> Result<broadcast::Receiver<TunnelEvent>, OnlineTunnelError> {
        if let Some(session) = self.host.lock().await.as_ref() {
            let phase = session
                .handle
                .status()
                .await
                .map_err(|error| OnlineTunnelError::provider("read tunnel status", error))?
                .phase;
            return match phase {
                HostedServicePhase::Active => Ok(self.events.sender.subscribe()),
                HostedServicePhase::Stopping => Err(OnlineTunnelError::Busy),
                HostedServicePhase::Stopped => Err(OnlineTunnelError::NotRunning),
            };
        }

        match self.join.status().state.phase {
            // join 命令现在立即返回 `starting`，就绪过程的事件（如 path_changed）也要能订阅。
            SculkPhase::Active | SculkPhase::Starting => Ok(self.events.sender.subscribe()),
            SculkPhase::Idle => Err(OnlineTunnelError::NotRunning),
            SculkPhase::Stopping => Err(OnlineTunnelError::Busy),
        }
    }

    /// 幂等关闭服务持有的隧道与事件转发任务。
    pub async fn shutdown(&self) -> Result<(), OnlineTunnelError> {
        // 正在 bind 的 Host 启动也要一并取消。
        self.host_cancel.store(true, Ordering::SeqCst);
        self.stop_fanout();
        self.stop_lan_broadcast();
        if let Some(session) = self.host.lock().await.take() {
            let stop_result = session.handle.stop().await;
            session.node.close().await;
            stop_result.map_err(|error| OnlineTunnelError::provider("stop host service", error))?;
        }
        self.join.shutdown().await.map_err(map_service_error)
    }

    /// 重新订阅 Join 事件流并启动扇出任务。
    fn restart_join_fanout(&self) {
        self.stop_fanout();
        let mut updates = self.join.subscribe();
        let sender = self.events.sender.clone();
        let broadcast = Arc::clone(&self.lan_broadcast);
        let task = tokio::spawn(async move {
            while let Some(update) = updates.recv().await {
                match update {
                    // 就绪时把房间公告到本机局域网，Minecraft 多人列表会直接列出它。
                    SculkUpdate::Status(status) => sync_lan_broadcast(&broadcast, &status),
                    SculkUpdate::Event(event) => {
                        let _ = sender.send(map_event(event));
                    }
                    _ => {}
                }
            }
        });
        self.retain_fanout(task);
    }

    /// 重新订阅 Host 事件流并启动扇出任务，同时维护对端快照。
    async fn restart_host_fanout(
        &self,
        handle: &HostedServiceHandle,
        peers: Arc<StdMutex<BTreeMap<String, PeerState>>>,
    ) -> Result<(), OnlineTunnelError> {
        self.stop_fanout();
        let mut events = handle
            .subscribe()
            .await
            .map_err(|error| OnlineTunnelError::provider("subscribe host events", error))?;
        let sender = self.events.sender.clone();
        let task = tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(event) => {
                        apply_peer_event(&peers, &event);
                        let _ = sender.send(map_event(event));
                    }
                    // 广播落后只丢失过程事件；对端快照会在后续事件中收敛。
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(
                            target: "sealantern.feature.online",
                            skipped,
                            "host event fanout lagged"
                        );
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        self.retain_fanout(task);
        Ok(())
    }

    /// 保存扇出任务句柄；句柄不可用时终止任务，避免监听泄漏。
    fn retain_fanout(&self, task: JoinHandle<()>) {
        match self.events.task.lock() {
            Ok(mut slot) => *slot = Some(task),
            Err(_) => task.abort(),
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

/// Host 状态文件位置：`<应用数据目录>/online/host.state`。
fn default_host_state_path() -> PathBuf {
    sealantern_infra::platform::get_app_data_dir()
        .join("online")
        .join("host.state")
}

/// Host 节点身份密钥位置：与 `host.state` 同目录。
fn identity_key_path(state_path: &Path) -> PathBuf {
    state_path.with_file_name(HOST_KEY_FILE)
}

/// Host 启动被 [`OnlineTunnelService::stop`] 取消时的错误。
///
/// 复用 `Provider` 分类：对调用方而言这与「启动失败」同样是操作未完成。
fn cancelled_host() -> OnlineTunnelError {
    OnlineTunnelError::provider("start host service", "host start was cancelled")
}

/// 把应用请求翻译为 sculk 的 Join 启动参数。
fn build_join_options(request: &JoinTunnelRequest) -> Result<JoinOptions, OnlineTunnelError> {
    let join_uri: JoinUri = request
        .ticket
        .as_str()
        .parse()
        .map_err(|error| OnlineTunnelError::provider("parse tunnel ticket", error))?;
    let local_port = NonZeroU16::new(request.local_port).map_or(LocalPort::Auto, LocalPort::Fixed);

    Ok(JoinOptions::new(join_uri).local_port(local_port).config(
        // `reconnect_timeout` 保持 `None`：断线后无限重连，只有用户主动停止才结束。
        JoinConfig::default().reconnect_timeout(None),
    ))
}

/// 把应用层的有效期策略翻译为 sculk 的令牌刷新策略。
fn token_refresh_policy(lifetime: TunnelLinkLifetime) -> TokenRefreshPolicy {
    match lifetime {
        TunnelLinkLifetime::UntilStopped => TokenRefreshPolicy::Always,
        TunnelLinkLifetime::Permanent => TokenRefreshPolicy::Never,
        TunnelLinkLifetime::After(period) => TokenRefreshPolicy::After(period),
    }
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

/// `JoinUri` → 用户侧网页分享链接。
fn to_share_url(uri: &str) -> String {
    uri.strip_prefix(JOIN_URI_PREFIX)
        .filter(|payload| !payload.is_empty())
        .map_or_else(|| uri.to_owned(), |payload| format!("{SHARE_URL_PREFIX}{payload}"))
}

async fn read_ticket(handle: &HostedServiceHandle) -> Result<TunnelTicket, OnlineTunnelError> {
    let uri = handle
        .join_uri()
        .await
        .map_err(|error| OnlineTunnelError::provider("read tunnel ticket", error))?;
    let uri = uri
        .expose_secret_uri()
        .map_err(|error| OnlineTunnelError::provider("read tunnel ticket", error))?;
    Ok(TunnelTicket::from_provider(to_share_url(&uri)))
}

async fn save_host_state(
    path: &Path,
    handle: &HostedServiceHandle,
) -> Result<(), OnlineTunnelError> {
    let token_state = handle
        .token_state()
        .await
        .map_err(|error| OnlineTunnelError::provider("read token state", error))?;
    persist::save_host_state(
        path,
        &PersistedHostState {
            service_id: handle.service_id(),
            token_state,
        },
    )
    .map_err(|error| OnlineTunnelError::provider("save host state", error))
}

/// 按 Join 状态同步局域网公告：就绪时公告本地端口，其他阶段一律停止。
fn sync_lan_broadcast(broadcast: &StdMutex<Option<LanBroadcaster>>, status: &SculkStatus) {
    let desired_port = if status.state.phase == SculkPhase::Active
        && status.state.mode == Some(sculk::tunnel::TunnelMode::Join)
    {
        status
            .state
            .local_addr
            .and_then(|addr| NonZeroU16::new(addr.port()))
    } else {
        None
    };

    let Ok(mut current) = broadcast.lock() else {
        return;
    };

    let Some(port) = desired_port else {
        if let Some(broadcaster) = current.take() {
            let _ = broadcaster.stop();
        }
        return;
    };

    // 后台线程会自行探测端口可用性；仍在运行就不重建。
    if current.as_ref().is_some_and(|item| !item.is_finished()) {
        return;
    }
    if let Some(broadcaster) = current.take() {
        let _ = broadcaster.stop();
    }
    match LanBroadcaster::start(LAN_BROADCAST_NAME, port) {
        Ok(broadcaster) => *current = Some(broadcaster),
        Err(error) => tracing::warn!(
            target: "sealantern.feature.online",
            %error,
            "failed to announce the joined world on the local network"
        ),
    }
}

fn apply_peer_event(peers: &StdMutex<BTreeMap<String, PeerState>>, event: &SculkEvent) {
    match event {
        SculkEvent::PlayerJoined { id } => track_peer_joined(peers, id.as_ref()),
        SculkEvent::PlayerLeft { id, .. } => track_peer_left(peers, id.as_ref()),
        SculkEvent::PathChanged { remote_id, is_relay, rtt_ms } => {
            track_peer_path(peers, remote_id.as_ref(), *is_relay, *rtt_ms);
        }
        _ => {}
    }
}

fn track_peer_joined(peers: &StdMutex<BTreeMap<String, PeerState>>, remote_id: &str) {
    if let Ok(mut peers) = peers.lock() {
        peers.insert(remote_id.to_owned(), PeerState::new(Instant::now()));
    }
}

fn track_peer_left(peers: &StdMutex<BTreeMap<String, PeerState>>, remote_id: &str) {
    if let Ok(mut peers) = peers.lock() {
        peers.remove(remote_id);
    }
}

fn track_peer_path(
    peers: &StdMutex<BTreeMap<String, PeerState>>,
    remote_id: &str,
    is_relay: bool,
    rtt_ms: u64,
) {
    let Ok(mut peers) = peers.lock() else {
        return;
    };
    let peer = peers
        .entry(remote_id.to_owned())
        .or_insert_with(|| PeerState::new(Instant::now()));
    peer.is_relay = is_relay;
    peer.rtt_ms = rtt_ms;
}

async fn host_status(session: &HostSession) -> Result<TunnelStatus, OnlineTunnelError> {
    let status: HostedServiceStatus = session
        .handle
        .status()
        .await
        .map_err(|error| OnlineTunnelError::provider("read tunnel status", error))?;
    let connections = session
        .peers
        .lock()
        .map(|peers| {
            peers
                .iter()
                .map(|(remote_id, peer)| TunnelConnection {
                    remote_id: remote_id.clone(),
                    is_relay: peer.is_relay,
                    rtt_ms: peer.rtt_ms,
                    // Node API 的事件流不携带累计字节。
                    tx_bytes: 0,
                    rx_bytes: 0,
                    alive: true,
                    elapsed_ms: peer.joined_at.elapsed().as_millis() as u64,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(TunnelStatus {
        active: status.phase == HostedServicePhase::Active,
        phase: map_host_phase(status.phase),
        mode: Some(TunnelMode::Host),
        ticket: Some(read_ticket(&session.handle).await?),
        local_address: None,
        connections,
        last_error: status.last_error.map(map_error_category),
    })
}

fn map_host_phase(phase: HostedServicePhase) -> TunnelPhase {
    match phase {
        HostedServicePhase::Active => TunnelPhase::Active,
        HostedServicePhase::Stopping => TunnelPhase::Stopping,
        HostedServicePhase::Stopped => TunnelPhase::Idle,
    }
}

/// sculk Join 状态快照 → 应用状态快照。
fn map_join_status(status: &SculkStatus) -> TunnelStatus {
    let active = status.state.phase == SculkPhase::Active;
    let mode = status.state.mode.map(map_mode);
    let ticket = status
        .state
        .join_uri
        .as_ref()
        .and_then(|uri| uri.expose_secret_uri().ok())
        .map(|ticket| TunnelTicket::from_provider(to_share_url(&ticket)));
    let connections = status.connections.iter().map(map_connection).collect();
    TunnelStatus {
        active,
        phase: map_phase(status.state.phase),
        mode,
        ticket,
        local_address: status.state.local_addr.map(|addr| addr.to_string()),
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
        SculkServiceError::InvalidPort => OnlineTunnelError::port_unavailable("invalid port"),
        SculkServiceError::InvalidMaxPlayers => {
            OnlineTunnelError::invalid_request("max players must be greater than zero")
        }
        other => OnlineTunnelError::provider("operate tunnel", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sculk::tunnel::{AccessToken, JoinUri, ServiceId};

    #[tokio::test]
    async fn a_new_service_is_idle() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("idle"));

        assert_eq!(service.status().await.expect("idle status"), TunnelStatus::idle());
    }

    #[tokio::test]
    async fn idle_service_stops_idempotently_and_rejects_subscription() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("idle-reject"));

        // 空闲时停止等价于「没有隧道可停」，不再报错，方便前端取消建链。
        assert_eq!(service.stop().await.expect("idle stop"), TunnelStatus::idle());
        assert!(matches!(service.subscribe().await, Err(OnlineTunnelError::NotRunning)));
    }

    #[tokio::test]
    async fn shutdown_is_idempotent_while_idle() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("shutdown"));

        service
            .shutdown()
            .await
            .expect("idle shutdown must succeed");
    }

    #[tokio::test]
    async fn host_rejects_an_out_of_range_port() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("port"));

        let result = service
            .host(HostTunnelRequest {
                minecraft_port: 0,
                max_players: None,
                relay_url: None,
                link_lifetime: TunnelLinkLifetime::default(),
                identity: None,
            })
            .await;

        assert!(matches!(result, Err(OnlineTunnelError::PortUnavailable { .. })));
        assert_eq!(service.status().await.expect("idle status"), TunnelStatus::idle());
    }

    #[tokio::test]
    async fn stop_marks_a_pending_host_start_as_cancelled() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("host-cancel"));

        // 空闲时 stop 本身是空操作，但必须置位取消标记：
        // 正在 bind 的 host 靠它在下个检查点放弃，否则「取消连接」是假的。
        service.stop().await.expect("idle stop");

        assert!(service.host_cancel.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn a_pending_host_start_blocks_other_tunnels() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("host-starting"));
        // 模拟 host 已进入启动流程（持启动锁、但还没写入会话槽）。
        let _guard = service.host_start.lock().await;

        assert!(matches!(service.ensure_idle().await, Err(OnlineTunnelError::Busy)));
    }

    #[test]
    fn stopping_lan_broadcast_without_one_is_a_no_op() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("lan-idle"));

        service.stop_lan_broadcast();
        service.stop_lan_broadcast();
    }

    #[test]
    fn host_secret_key_is_stable_across_calls() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("identity"));

        // 端点身份必须落盘：否则每次重启换 endpoint_id，复用的分享链接就失效。
        let first = service.host_secret_key(None).expect("generated identity");
        let second = service
            .host_secret_key(None)
            .expect("persisted identity must reload");

        assert_eq!(first.to_bytes(), second.to_bytes());
    }

    #[test]
    fn host_secret_key_prefers_an_explicit_identity() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("identity-explicit"));
        let identity = TunnelIdentity::from_bytes([0x11; 32]);

        let key = service
            .host_secret_key(Some(&identity))
            .expect("explicit identity");

        assert_eq!(key.to_bytes(), [0x11; 32]);
    }

    #[test]
    fn join_options_reject_a_malformed_ticket() {
        let request = JoinTunnelRequest {
            ticket: TunnelTicket::from_provider("not-a-ticket"),
            local_port: 30000,
            max_retries: None,
        };

        assert!(build_join_options(&request).is_err());
    }

    #[tokio::test]
    async fn join_returns_immediately_and_stop_cancels() {
        let service = OnlineTunnelService::with_state_path(temp_state_path("cancel-join"));
        let request = JoinTunnelRequest {
            ticket: unreachable_ticket("cancel-join"),
            local_port: 30000,
            max_retries: None,
        };

        // join 只接受启动任务并立即返回，不阻塞等待就绪。
        let started = service.join(request).await.expect("join must start");
        assert_eq!(started.phase, TunnelPhase::Starting);

        // 「取消连接」就是一次 stop：sculk 会 abort 正在启动的后台任务。
        let stopped = service.stop().await;
        assert!(stopped.is_ok(), "取消连接必须成功: {stopped:?}");

        // 取消后状态机回到空闲，后续仍可再次加入。
        assert_eq!(service.status().await.expect("idle status"), TunnelStatus::idle());
    }

    #[test]
    fn maps_link_lifetime_to_token_refresh() {
        assert_eq!(
            token_refresh_policy(TunnelLinkLifetime::UntilStopped),
            TokenRefreshPolicy::Always
        );
        assert_eq!(token_refresh_policy(TunnelLinkLifetime::Permanent), TokenRefreshPolicy::Never);
        assert_eq!(
            token_refresh_policy(TunnelLinkLifetime::After(Duration::from_secs(3 * 60 * 60))),
            TokenRefreshPolicy::After(Duration::from_secs(3 * 60 * 60))
        );
    }

    #[test]
    fn wraps_join_uri_into_share_url() {
        assert_eq!(
            to_share_url("sculk://join/v1/payload_123-abc"),
            "https://ideaflash.cn/#v1/payload_123-abc"
        );
        assert_eq!(to_share_url("sculk://join/v1/"), "sculk://join/v1/");
        assert_eq!(to_share_url("https://example.com/invite"), "https://example.com/invite");
    }

    #[test]
    fn tracks_peers_from_events() {
        let peers = StdMutex::new(BTreeMap::new());

        track_peer_joined(&peers, "peer-a");
        assert!(peers.lock().expect("peers").contains_key("peer-a"));

        track_peer_path(&peers, "peer-a", true, 42);
        let peer = peers
            .lock()
            .expect("peers")
            .get("peer-a")
            .cloned()
            .expect("peer");
        assert!(peer.is_relay);
        assert_eq!(peer.rtt_ms, 42);

        track_peer_left(&peers, "peer-a");
        assert!(peers.lock().expect("peers").is_empty());
    }

    fn temp_state_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("sealantern_online_{name}_{}", std::process::id()))
            .join("host.state")
    }

    /// 格式合法但指向不存在对端的票据：隧道会一直停在启动阶段。
    fn unreachable_ticket(name: &str) -> TunnelTicket {
        let endpoint = SecretKey::from_bytes(&[0x42; 32]).public();
        let uri = JoinUri::new(endpoint, ServiceId::generate(), AccessToken::generate(), None)
            .expose_secret_uri()
            .unwrap_or_else(|error| panic!("{name}: must build a valid join uri: {error}"));

        TunnelTicket::from_provider(uri)
    }
}
