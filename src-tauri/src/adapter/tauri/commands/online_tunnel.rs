//! Snake-case Tauri commands for the process-owned online tunnel service.

use sealantern_application::services::AppServices;
use sealantern_interface::{
    OnlineTunnelHostRequest, OnlineTunnelJoinRequest, OnlineTunnelService,
    OnlineTunnelServiceError, OnlineTunnelStatus,
};
use tauri::{AppHandle, Emitter};

async fn services() -> Result<AppServices, OnlineTunnelServiceError> {
    AppServices::get()
        .await
        .map_err(|_| OnlineTunnelServiceError::OperationFailed)
}

fn forward_events(app: AppHandle, services: AppServices) {
    tauri::async_runtime::spawn(async move {
        let Ok(mut events) = services.online_tunnel().subscribe().await else {
            return;
        };
        while let Ok(event) = events.recv().await {
            let _ = app.emit("online_tunnel_event", event);
        }
    });
}

#[tauri::command]
pub async fn online_tunnel_host(
    app: AppHandle,
    request: OnlineTunnelHostRequest,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let services = services().await?;
    let status = services.online_tunnel().host(request).await?;
    forward_events(app, services);
    Ok(status)
}

#[tauri::command]
pub async fn online_tunnel_join(
    app: AppHandle,
    request: OnlineTunnelJoinRequest,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let services = services().await?;
    let status = services.online_tunnel().join(request).await?;
    forward_events(app, services);
    Ok(status)
}

#[tauri::command]
pub async fn online_tunnel_stop() -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    services().await?.online_tunnel().stop().await
}

#[tauri::command]
pub async fn online_tunnel_status() -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    services().await?.online_tunnel().status().await
}
