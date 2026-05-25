//! 隧道配置、运行时事件与连接快照类型。
//!
//! 调用方通过 [`HostConfig`] / [`JoinConfig`] 描述隧道行为策略，
//! 通过 `mpsc::Receiver<TunnelEvent>` 接收运行时状态推送，
//! 并可调用 [`IrohTunnel::connections`](crate::tunnel::IrohTunnel::connections)
//! 获取 [`ConnectionSnapshot`] 用于指标展示。

use std::fmt;
use std::time::Duration;

/// 对端节点标识，由 `EndpointId::fmt_short()` 生成的短格式。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerId(String);

impl PeerId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for PeerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Host 端隧道配置。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HostConfig {
    /// `PathChanged` 发送策略：`ZERO` 仅变化时发送，非零按间隔发送。
    pub event_delay: Duration,
    /// 连接密码，`None` 表示不校验。
    pub password: Option<String>,
    /// 最大玩家数（按唯一 `EndpointId` 计）。
    pub max_players: Option<u32>,
}

impl HostConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event_delay(mut self, delay: Duration) -> Self {
        self.event_delay = delay;
        self
    }

    pub fn password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    pub fn max_players(mut self, max_players: Option<u32>) -> Self {
        self.max_players = max_players;
        self
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            event_delay: Duration::ZERO,
            password: None,
            max_players: None,
        }
    }
}

/// Join 端隧道配置。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct JoinConfig {
    /// `PathChanged` 发送策略：`ZERO` 仅变化时发送，非零按间隔发送。
    pub event_delay: Duration,
    /// 连接密码，`None` 表示不校验。
    pub password: Option<String>,
    /// 最大重连次数：`None` 无限，`Some(0)` 关闭重连。
    pub max_retries: Option<u32>,
    /// 首次连接的重试上限，默认 3 次。
    pub initial_retries: u32,
    /// 重连初始退避。
    pub base_backoff: Duration,
    /// 重连最大退避。
    pub max_backoff: Duration,
}

impl JoinConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event_delay(mut self, delay: Duration) -> Self {
        self.event_delay = delay;
        self
    }

    pub fn password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    pub fn max_retries(mut self, max_retries: Option<u32>) -> Self {
        self.max_retries = max_retries;
        self
    }
}

impl Default for JoinConfig {
    fn default() -> Self {
        Self {
            event_delay: Duration::ZERO,
            password: None,
            max_retries: None,
            initial_retries: 3,
            base_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
        }
    }
}

/// 隧道运行时事件，由后台任务通过 `mpsc::Sender` 推送给调用方。
///
/// host 侧接收玩家连接/断开/拒绝事件，join 侧接收连接/重连/断开事件，
/// 双端均可收到 `PathChanged` 和 `Error`。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TunnelEvent {
    /// host 侧：新玩家建立连接。
    PlayerJoined { id: PeerId },
    /// host 侧：玩家断开连接，`reason` 为 QUIC 层关闭原因。
    PlayerLeft { id: PeerId, reason: String },
    /// join 侧：与 host 的 QUIC 连接已建立。
    Connected,
    /// join 侧：与 host 的连接断开，`reason` 为关闭原因。若将重连，随后会发送 [`Self::Reconnecting`]。
    Disconnected { reason: String },
    /// 选中路径切换或 RTT 变化时触发，`event_delay` 控制发送节流。
    PathChanged {
        remote_id: PeerId,
        is_relay: bool,
        rtt_ms: u64,
    },
    /// join 侧：即将发起第 `attempt` 次重连。
    Reconnecting { attempt: u32 },
    /// join 侧：重连成功。
    Reconnected,
    /// host 侧：密码验证失败，连接已被关闭。
    AuthFailed { id: PeerId },
    /// host 侧：连接被主动拒绝，如服务器满员时 `reason` 为 `"server full"`。
    PlayerRejected { id: PeerId, reason: String },
    /// 非致命的内部或 I/O 错误，隧道仍在运行。
    Error { message: String },
}

/// 单条连接的瞬时状态快照，由 [`IrohTunnel::connections`](crate::tunnel::IrohTunnel::connections) 返回。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConnectionSnapshot {
    /// 对端节点 ID。
    pub remote_id: PeerId,
    /// 当前路径是否经由 relay 转发，`false` 为直连。
    pub is_relay: bool,
    /// 选中路径的往返延迟，单位毫秒。
    pub rtt_ms: u64,
    /// 累计已发送 UDP 字节数。
    pub tx_bytes: u64,
    /// 累计已接收 UDP 字节数。
    pub rx_bytes: u64,
    /// 连接是否仍活跃。
    pub alive: bool,
    /// 快照采样时刻，距隧道创建的经过时间。
    pub elapsed: Duration,
}
