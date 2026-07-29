use crate::services::online::tunnel;

#[tauri::command]
pub async fn tunnel_host(
    port: u16,
    password: Option<String>,
    max_players: Option<u32>,
    relay_url: Option<String>,
) -> Result<tunnel::TunnelStatus, String> {
    tunnel::host(port, password, max_players, relay_url).await
}

#[tauri::command]
pub async fn tunnel_join(
    ticket: String,
    local_port: u16,
    password: Option<String>,
) -> Result<tunnel::TunnelStatus, String> {
    tunnel::join(ticket, local_port, password).await
}

#[tauri::command]
pub async fn tunnel_stop() -> Result<tunnel::TunnelStatus, String> {
    tunnel::stop().await
}

#[tauri::command]
pub async fn tunnel_status() -> Result<tunnel::TunnelStatus, String> {
    Ok(tunnel::status().await)
}

#[tauri::command]
pub async fn tunnel_copy_ticket() -> Result<bool, String> {
    tunnel::copy_ticket().await
}

#[tauri::command]
pub async fn tunnel_regenerate_ticket() -> Result<tunnel::TunnelStatus, String> {
    tunnel::regenerate_ticket().await
}

#[tauri::command]
pub async fn tunnel_generate_ticket() -> Result<tunnel::TunnelStatus, String> {
    tunnel::generate_ticket().await
}
