use std::fmt;
use std::time::Duration;

/// 当前运行的隧道角色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelMode {
    Host,
    Join,
}

impl TunnelMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Join => "join",
        }
    }
}

/// 隧道生命周期阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelPhase {
    Idle,
    Starting,
    Active,
    Stopping,
}

/// 隧道错误的产品级分类。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelErrorCategory {
    InvalidJoinUri,
    InvalidEndpoint,
    AuthorizationDenied,
    HostUnreachable,
    TargetUnavailable,
    LocalPortUnavailable,
    IdentityUnavailable,
    OperationConflict,
    ResourceLimit,
    InvalidConfiguration,
    Internal,
}

/// 分享链接（访问令牌）的有效期策略；令牌轮换后旧链接立即失效。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TunnelLinkLifetime {
    /// 关闭隧道前有效；每次发布都生成新链接。
    #[default]
    UntilStopped,
    /// 复用同一条链接，直到手动轮换。
    Permanent,
    /// 每过指定时长自动轮换。
    After(Duration),
}

impl TunnelLinkLifetime {
    /// 支持的有效期选项，与前端下拉框一一对应。
    pub const OPTIONS: [&'static str; 7] = ["always", "never", "1h", "3h", "6h", "12h", "24h"];

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "always" => Some(Self::UntilStopped),
            "never" => Some(Self::Permanent),
            "1h" => Some(Self::After(Duration::from_secs(60 * 60))),
            "3h" => Some(Self::After(Duration::from_secs(3 * 60 * 60))),
            "6h" => Some(Self::After(Duration::from_secs(6 * 60 * 60))),
            "12h" => Some(Self::After(Duration::from_secs(12 * 60 * 60))),
            "24h" => Some(Self::After(Duration::from_secs(24 * 60 * 60))),
            _ => None,
        }
    }

    /// 稳定标识，与 [`Self::parse`] 互为逆运算。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UntilStopped => "always",
            Self::Permanent => "never",
            Self::After(period) => match period.as_secs() {
                3600 => "1h",
                10800 => "3h",
                21600 => "6h",
                43200 => "12h",
                86400 => "24h",
                _ => "always",
            },
        }
    }
}

/// Host 隧道启动请求。
#[derive(Clone)]
pub struct HostTunnelRequest {
    pub minecraft_port: u16,
    pub max_players: Option<u32>,
    pub relay_url: Option<String>,
    pub link_lifetime: TunnelLinkLifetime,
    pub identity: Option<TunnelIdentity>,
}

impl fmt::Debug for HostTunnelRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostTunnelRequest")
            .field("minecraft_port", &self.minecraft_port)
            .field("max_players", &self.max_players)
            .field("relay_url", &self.relay_url)
            .field("link_lifetime", &self.link_lifetime)
            .field("identity", &self.identity.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Join 隧道启动请求。
#[derive(Clone)]
pub struct JoinTunnelRequest {
    pub ticket: TunnelTicket,
    pub local_port: u16,
    pub max_retries: Option<u32>,
}

impl fmt::Debug for JoinTunnelRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JoinTunnelRequest")
            .field("ticket", &"[REDACTED]")
            .field("local_port", &self.local_port)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

/// 由用户分享的隧道票据。
///
/// sculk 0.6 起票据形态为 `JoinUri`（`sculk://join/v1/...`），校验由
/// [`sculk::tunnel::JoinUri`] 完成。
#[derive(Clone, PartialEq, Eq)]
pub struct TunnelTicket(String);

impl TunnelTicket {
    pub fn parse(value: impl Into<String>) -> Result<Self, OnlineTunnelError> {
        let value = normalize_share_url(value.into());
        if value.trim().is_empty() {
            return Err(OnlineTunnelError::invalid_request("tunnel ticket must not be empty"));
        }
        value
            .parse::<sculk::tunnel::JoinUri>()
            .map_err(|error| OnlineTunnelError::provider("parse tunnel ticket", error))?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_provider(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// 用户侧使用网页分享链接；仅在进入 sculk 前转换为内部 Join URI。
fn normalize_share_url(value: String) -> String {
    let trimmed = value.trim().to_owned();
    trimmed
        .strip_prefix("https://ideaflash.cn/#v1/")
        .filter(|payload| !payload.is_empty() && !payload.contains(['/', '?', '#', ' ']))
        .map_or(trimmed.clone(), |payload| format!("sculk://join/v1/{payload}"))
}

impl fmt::Debug for TunnelTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TunnelTicket([REDACTED])")
    }
}

/// Host 节点使用的 32 字节稳定身份密钥。
#[derive(Clone)]
pub struct TunnelIdentity([u8; 32]);

impl TunnelIdentity {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for TunnelIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TunnelIdentity([REDACTED])")
    }
}

/// 已建立隧道中单个对端的运行时快照。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TunnelConnection {
    pub remote_id: String,
    pub is_relay: bool,
    pub rtt_ms: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub alive: bool,
    pub elapsed_ms: u64,
}

/// 应用层的在线隧道事件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TunnelEvent {
    PlayerJoined {
        remote_id: String,
    },
    PlayerLeft {
        remote_id: String,
        reason: String,
    },
    Connected,
    Disconnected {
        reason: String,
    },
    PathChanged {
        remote_id: String,
        is_relay: bool,
        rtt_ms: u64,
    },
    Reconnecting {
        attempt: u32,
    },
    Reconnected,
    AuthenticationFailed {
        remote_id: String,
    },
    PlayerRejected {
        remote_id: String,
        reason: String,
    },
    /// 访问令牌已轮换，Host 应重新获取票据。
    TokenRotated,
    Error {
        category: TunnelErrorCategory,
        message: String,
    },
    ProviderMessage {
        message: String,
    },
}

/// 当前在线隧道状态快照。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TunnelStatus {
    pub active: bool,
    pub phase: TunnelPhase,
    pub mode: Option<TunnelMode>,
    pub ticket: Option<TunnelTicket>,
    /// Join 隧道实际绑定的本地监听地址；Host 或空闲时为空。
    pub local_address: Option<String>,
    pub connections: Vec<TunnelConnection>,
    pub last_error: Option<TunnelErrorCategory>,
}

impl TunnelStatus {
    pub(crate) fn idle() -> Self {
        Self {
            active: false,
            phase: TunnelPhase::Idle,
            mode: None,
            ticket: None,
            local_address: None,
            connections: Vec::new(),
            last_error: None,
        }
    }
}

/// 在线隧道操作失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnlineTunnelError {
    InvalidRequest {
        message: String,
    },
    /// Minecraft 本地端口未开放，或没有已开放到 LAN 的世界。
    PortUnavailable {
        message: String,
    },
    Busy,
    NotRunning,
    Provider {
        operation: &'static str,
        message: String,
    },
}

impl OnlineTunnelError {
    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest { message: message.into() }
    }

    pub(crate) fn port_unavailable(message: impl Into<String>) -> Self {
        Self::PortUnavailable { message: message.into() }
    }

    pub(crate) fn provider(operation: &'static str, error: impl fmt::Display) -> Self {
        Self::Provider { operation, message: error.to_string() }
    }
}

impl fmt::Display for OnlineTunnelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { message } => {
                write!(formatter, "invalid tunnel request: {message}")
            }
            Self::PortUnavailable { message } => {
                write!(formatter, "minecraft port is unavailable: {message}")
            }
            Self::Busy => formatter.write_str("an online tunnel is already starting or running"),
            Self::NotRunning => formatter.write_str("no online tunnel is running"),
            Self::Provider { operation, message } => {
                write!(formatter, "failed to {operation}: {message}")
            }
        }
    }
}

impl std::error::Error for OnlineTunnelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_debug_output_is_redacted() {
        let ticket = TunnelTicket::from_provider("sculk://sensitive-ticket");

        assert!(!format!("{ticket:?}").contains(ticket.as_str()));
    }

    #[test]
    fn link_lifetime_roundtrips_every_option() {
        for option in TunnelLinkLifetime::OPTIONS {
            let lifetime = TunnelLinkLifetime::parse(option).expect("known option");
            assert_eq!(lifetime.as_str(), option);
        }
    }

    #[test]
    fn link_lifetime_rejects_unknown_option() {
        assert_eq!(TunnelLinkLifetime::parse("2h"), None);
        assert_eq!(TunnelLinkLifetime::default(), TunnelLinkLifetime::UntilStopped);
    }
}
