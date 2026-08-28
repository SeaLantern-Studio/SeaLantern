//! 在线隧道（联机）Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`OnlineTunnelService`] 以主机或加入模式建立隧道；隧道运行事件
//! 由 [`OnlineTunnelEventForwarder`] 转发为前端 `online_tunnel_event` 事件。
//!
//! 错误统一为接口契约错误 [`OnlineTunnelServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use std::sync::Mutex;

use sealantern_application::port::OnlineTunnelService;
use sealantern_application::services::AppServices;
use sealantern_contract::OnlineTunnelServiceError;
use sealantern_contract::online::{
    OnlineTunnelHostRequest, OnlineTunnelJoinRequest, OnlineTunnelStatus,
};
use tauri::{AppHandle, Emitter, State};

/// 持有在线隧道事件转发任务的后台句柄。
///
/// 由 Tauri 全局托管（见 `lib.rs` 的 `manage` 调用），保证同一时刻
/// 只有一个任务在监听事件流并向前端推送 `online_tunnel_event`。
#[derive(Default)]
pub struct OnlineTunnelEventForwarder {
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

impl OnlineTunnelEventForwarder {
    /// 停止旧转发任务，重新订阅服务事件流并启动新的转发任务。
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

    /// 停止当前转发任务并等待其退出。
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

/// 获取全局应用服务句柄（惰性初始化容器）。
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

/// 以主机模式开启在线隧道，把本地 Minecraft 端口转发到公网。
#[tauri::command(rename_all = "snake_case")]
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

/// 以加入模式通过票据连接主机，把隧道流量导向本地端口。
#[tauri::command(rename_all = "snake_case")]
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

/// 停止当前在线隧道。
#[tauri::command(rename_all = "snake_case")]
pub async fn online_tunnel_stop(
    forwarder: State<'_, OnlineTunnelEventForwarder>,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let status = services().await?.online_tunnel().stop().await?;
    forwarder.clear().await;
    Ok(status)
}

/// 查询当前在线隧道的运行状态。
#[tauri::command(rename_all = "snake_case")]
pub async fn online_tunnel_status() -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    services().await?.online_tunnel().status().await
}
