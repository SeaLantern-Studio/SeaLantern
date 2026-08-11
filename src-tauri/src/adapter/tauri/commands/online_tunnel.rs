//! Snake-case Tauri commands for the process-owned online tunnel service.

use std::sync::Mutex;

use sealantern_application::services::AppServices;
use sealantern_interface::{
    OnlineTunnelHostRequest, OnlineTunnelJoinRequest, OnlineTunnelService,
    OnlineTunnelServiceError, OnlineTunnelStatus,
};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
pub struct OnlineTunnelEventForwarder {
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

impl OnlineTunnelEventForwarder {
    async fn replace(
        &self,
        app: AppHandle,
        services: AppServices,
    ) -> Result<(), OnlineTunnelServiceError> {
        self.clear().await;
        let mut events = services.online_tunnel().subscribe().await?;
        let task = tauri::async_runtime::spawn(async move {
            while let Ok(event) = events.recv().await {
                let _ = app.emit("online_tunnel_event", event);
            }
        });

        match self.task.lock() {
            Ok(mut slot) => {
                *slot = Some(task);
                Ok(())
            }
            Err(error) => {
                task.abort();
                tracing::error!(
                    target: "sealantern.tauri.online_tunnel",
                    error = %error,
                    "failed to retain online tunnel event forwarder"
                );
                Err(OnlineTunnelServiceError::OperationFailed)
            }
        }
    }

    pub async fn clear(&self) {
        let task = match self.task.lock() {
            Ok(mut slot) => slot.take(),
            Err(error) => {
                tracing::error!(
                    target: "sealantern.tauri.online_tunnel",
                    error = %error,
                    "failed to access online tunnel event forwarder"
                );
                None
            }
        };
        if let Some(task) = task {
            task.abort();
            let _ = task.await;
        }
    }
}

async fn services() -> Result<AppServices, OnlineTunnelServiceError> {
    AppServices::get().await.map_err(|error| {
        tracing::error!(
            target: "sealantern.tauri.online_tunnel",
            error = %error,
            "failed to initialize application services for online tunnel"
        );
        OnlineTunnelServiceError::OperationFailed
    })
}

#[tauri::command]
pub async fn online_tunnel_host(
    app: AppHandle,
    forwarder: State<'_, OnlineTunnelEventForwarder>,
    request: OnlineTunnelHostRequest,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let services = services().await?;
    let status = services.online_tunnel().host(request).await?;
    forwarder.replace(app, services).await?;
    Ok(status)
}

#[tauri::command]
pub async fn online_tunnel_join(
    app: AppHandle,
    forwarder: State<'_, OnlineTunnelEventForwarder>,
    request: OnlineTunnelJoinRequest,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let services = services().await?;
    let status = services.online_tunnel().join(request).await?;
    forwarder.replace(app, services).await?;
    Ok(status)
}

#[tauri::command]
pub async fn online_tunnel_stop(
    forwarder: State<'_, OnlineTunnelEventForwarder>,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let status = services().await?.online_tunnel().stop().await?;
    forwarder.clear().await;
    Ok(status)
}

#[tauri::command]
pub async fn online_tunnel_status() -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    services().await?.online_tunnel().status().await
}
