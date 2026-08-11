use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelHostRequest {
    pub minecraft_port: u16,
    pub password: Option<String>,
    pub max_players: Option<u32>,
    pub relay_url: Option<String>,
    /// Optional stable 32-byte host identity. It is never included in responses.
    pub identity: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelJoinRequest {
    pub ticket: String,
    pub local_port: u16,
    pub password: Option<String>,
    pub max_retries: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineTunnelMode {
    Host,
    Join,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelConnection {
    pub remote_id: String,
    pub is_relay: bool,
    pub rtt_ms: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub alive: bool,
    pub elapsed_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OnlineTunnelStatus {
    pub active: bool,
    pub mode: Option<OnlineTunnelMode>,
    /// Present only for a hosted tunnel so the caller can share it with peers.
    pub ticket: Option<String>,
    pub connections: Vec<OnlineTunnelConnection>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OnlineTunnelEvent {
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
