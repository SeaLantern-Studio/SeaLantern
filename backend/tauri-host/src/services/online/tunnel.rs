mod config;
mod events;
mod i18n;
mod runtime;
mod state;

use serde::Serialize;

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

pub async fn host(
    port: u16,
    password: Option<String>,
    max_players: Option<u32>,
    relay_url: Option<String>,
) -> Result<TunnelStatus, String> {
    runtime::host(port, password, max_players, relay_url).await
}

pub async fn join(
    ticket: String,
    local_port: u16,
    password: Option<String>,
) -> Result<TunnelStatus, String> {
    runtime::join(ticket, local_port, password).await
}

pub async fn regenerate_ticket() -> Result<TunnelStatus, String> {
    runtime::regenerate_ticket().await
}

pub async fn generate_ticket() -> Result<TunnelStatus, String> {
    runtime::generate_ticket().await
}

pub async fn stop() -> Result<TunnelStatus, String> {
    runtime::stop().await
}

pub async fn copy_ticket() -> Result<bool, String> {
    runtime::copy_ticket().await
}

pub async fn status() -> Result<TunnelStatus, String> {
    runtime::status().await
}

#[cfg(test)]
mod tests;
