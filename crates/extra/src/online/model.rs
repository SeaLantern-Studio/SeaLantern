use std::fmt;

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

/// Host 隧道启动请求。
#[derive(Clone)]
pub struct HostTunnelRequest {
    pub minecraft_port: u16,
    pub password: Option<String>,
    pub max_players: Option<u32>,
    pub relay_url: Option<String>,
    pub identity: Option<TunnelIdentity>,
}

impl fmt::Debug for HostTunnelRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostTunnelRequest")
            .field("minecraft_port", &self.minecraft_port)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("max_players", &self.max_players)
            .field("relay_url", &self.relay_url)
            .field("identity", &self.identity.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Join 隧道启动请求。
#[derive(Clone)]
pub struct JoinTunnelRequest {
    pub ticket: TunnelTicket,
    pub local_port: u16,
    pub password: Option<String>,
    pub max_retries: Option<u32>,
}

impl fmt::Debug for JoinTunnelRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JoinTunnelRequest")
            .field("ticket", &"[REDACTED]")
            .field("local_port", &self.local_port)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

/// 由用户分享的隧道票据。
#[derive(Clone, PartialEq, Eq)]
pub struct TunnelTicket(String);

impl TunnelTicket {
    pub fn parse(value: impl Into<String>) -> Result<Self, OnlineTunnelError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(OnlineTunnelError::invalid_request("tunnel ticket must not be empty"));
        }
        value
            .parse::<sculk::tunnel::Ticket>()
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

impl fmt::Debug for TunnelTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TunnelTicket([REDACTED])")
    }
}

impl fmt::Display for TunnelTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
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
    Error {
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
    pub mode: Option<TunnelMode>,
    pub ticket: Option<TunnelTicket>,
    pub connections: Vec<TunnelConnection>,
}

impl TunnelStatus {
    pub(crate) fn idle() -> Self {
        Self {
            active: false,
            mode: None,
            ticket: None,
            connections: Vec::new(),
        }
    }
}

/// 在线隧道操作失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnlineTunnelError {
    InvalidRequest {
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
            Self::Busy => formatter.write_str("an online tunnel is already starting or running"),
            Self::NotRunning => formatter.write_str("no online tunnel is running"),
            Self::Provider { operation, message } => {
                write!(formatter, "failed to {operation}: {message}")
            }
        }
    }
}

impl std::error::Error for OnlineTunnelError {}
