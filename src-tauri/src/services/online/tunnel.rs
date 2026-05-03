mod config;
mod events;
mod i18n;
mod runtime;
mod state;

pub use state::TunnelStatus;

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

pub async fn status() -> TunnelStatus {
    runtime::status().await
}

#[cfg(test)]
#[path = "../../../tests/unit/services_online_tunnel_tests.rs"]
mod tests;
