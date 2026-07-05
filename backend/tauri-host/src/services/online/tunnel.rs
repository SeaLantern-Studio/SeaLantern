#[cfg(feature = "online-tunnel")]
mod config;
#[cfg(feature = "online-tunnel")]
mod events;
#[cfg(feature = "online-tunnel")]
mod i18n;
#[cfg(feature = "online-tunnel")]
mod runtime;
#[cfg(feature = "online-tunnel")]
mod state;

use serde::Serialize;

#[cfg(not(feature = "online-tunnel"))]
const ONLINE_TUNNEL_UNAVAILABLE: &str =
    "online tunnel support is unavailable without the 'online-tunnel' feature";

#[derive(Debug, Serialize, Clone)]
pub struct TunnelConnection {
    pub remote_id: String,
    pub is_relay: bool,
    pub rtt_ms: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub alive: bool,
    pub elapsed_secs: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TunnelStatus {
    pub running: bool,
    pub mode: Option<String>,
    pub ticket: Option<String>,
    pub connections: Vec<TunnelConnection>,
    pub logs: Vec<String>,
    pub host_port: u16,
    pub join_port: u16,
    pub last_ticket: Option<String>,
    pub relay_url: Option<String>,
}

#[cfg(not(feature = "online-tunnel"))]
fn unavailable_error() -> String {
    ONLINE_TUNNEL_UNAVAILABLE.to_string()
}

#[cfg(not(feature = "online-tunnel"))]
fn unavailable_status() -> TunnelStatus {
    TunnelStatus {
        running: false,
        mode: None,
        ticket: None,
        connections: Vec::new(),
        logs: vec![ONLINE_TUNNEL_UNAVAILABLE.to_string()],
        host_port: 0,
        join_port: 0,
        last_ticket: None,
        relay_url: None,
    }
}

pub async fn host(
    port: u16,
    password: Option<String>,
    max_players: Option<u32>,
    relay_url: Option<String>,
) -> Result<TunnelStatus, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::host(port, password, max_players, relay_url).await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        let _ = (port, password, max_players, relay_url);
        Err(unavailable_error())
    }
}

pub async fn join(
    ticket: String,
    local_port: u16,
    password: Option<String>,
) -> Result<TunnelStatus, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::join(ticket, local_port, password).await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        let _ = (ticket, local_port, password);
        Err(unavailable_error())
    }
}

pub async fn regenerate_ticket() -> Result<TunnelStatus, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::regenerate_ticket().await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        Err(unavailable_error())
    }
}

pub async fn generate_ticket() -> Result<TunnelStatus, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::generate_ticket().await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        Err(unavailable_error())
    }
}

pub async fn stop() -> Result<TunnelStatus, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::stop().await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        Ok(unavailable_status())
    }
}

pub async fn copy_ticket() -> Result<bool, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::copy_ticket().await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        Err(unavailable_error())
    }
}

pub async fn status() -> Result<TunnelStatus, String> {
    #[cfg(feature = "online-tunnel")]
    {
        runtime::status().await
    }

    #[cfg(not(feature = "online-tunnel"))]
    {
        Ok(unavailable_status())
    }
}

#[cfg(all(test, feature = "online-tunnel"))]
mod tests;
