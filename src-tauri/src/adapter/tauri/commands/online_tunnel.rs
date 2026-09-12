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
    OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest, OnlineTunnelMode,
    OnlineTunnelStatus,
};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::broadcast::error::RecvError;

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
            loop {
                match events.recv().await {
                    Ok(event) => {
                        if let Err(error) = app.emit("online_tunnel_event", &event) {
                            tracing::error!(
                                target: "sealantern.tauri.online_tunnel",
                                error = %error,
                                "failed to emit online tunnel event"
                            );
                        }
                    }
                    Err(RecvError::Lagged(skipped)) => {
                        tracing::warn!(
                            target: "sealantern.tauri.online_tunnel",
                            skipped,
                            "online tunnel event forwarder lagged"
                        );
                    }
                    Err(RecvError::Closed) => break,
                }
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

/// 向前端推送一个隧道生命周期事件。
///
/// 生命周期事件由命令层发出而非事件流内部：`Started` 要等事件转发订阅就绪
/// 后再发，`Stopped` 要在停止转发前发出，否则会因 `broadcast` 当前无接收者
/// 而丢失。
fn emit_lifecycle_event(app: &AppHandle, event: OnlineTunnelEvent) {
    tracing::debug!(
        target: "sealantern.tauri.online_tunnel",
        event = ?event,
        "emitting online tunnel lifecycle event"
    );
    if let Err(error) = app.emit("online_tunnel_event", &event) {
        tracing::error!(
            target: "sealantern.tauri.online_tunnel",
            error = %error,
            "failed to emit online tunnel lifecycle event"
        );
    }
}

/// 把票据写入系统剪贴板；失败只记录日志，不影响隧道建立。
///
/// 能力层已经把内部 Join URI 转成用户侧分享链接，这里直接写入。
fn copy_ticket_to_clipboard(ticket: &str) {
    if let Err(error) = sealantern_infra::platform::copy_text(ticket) {
        tracing::warn!(
            target: "sealantern.tauri.online_tunnel",
            error = %error,
            "failed to copy tunnel ticket to clipboard"
        );
    }
}

/// 以主机模式开启在线隧道，把本地 Minecraft 端口转发到公网。
#[tauri::command(rename_all = "snake_case")]
pub async fn online_tunnel_host(
    app: AppHandle,
    forwarder: State<'_, OnlineTunnelEventForwarder>,
    services: State<'_, AppServices>,
    request: OnlineTunnelHostRequest,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let services = services.inner().clone();
    let status = services.online_tunnel().host(request).await?;
    forwarder.replace(app.clone(), services).await?;
    // 对齐 v1.2.0：host 成功后立即把票据复制到剪贴板，方便直接分享。
    if let Some(ticket) = status.ticket.as_deref() {
        copy_ticket_to_clipboard(ticket);
    }
    // 转发订阅就绪后再发 Started，保证该事件能落达前端。
    emit_lifecycle_event(
        &app,
        OnlineTunnelEvent::Started {
            mode: OnlineTunnelMode::Host,
            ticket: status.ticket.clone(),
        },
    );
    Ok(status)
}

/// 以加入模式通过票据连接主机，把隧道流量导向本地端口。
#[tauri::command(rename_all = "snake_case")]
pub async fn online_tunnel_join(
    app: AppHandle,
    forwarder: State<'_, OnlineTunnelEventForwarder>,
    services: State<'_, AppServices>,
    request: OnlineTunnelJoinRequest,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    let services = services.inner().clone();
    let status = services.online_tunnel().join(request).await?;
    forwarder.replace(app.clone(), services).await?;
    // 转发订阅就绪后再发 Started，保证该事件能落达前端。
    emit_lifecycle_event(
        &app,
        OnlineTunnelEvent::Started {
            mode: OnlineTunnelMode::Join,
            ticket: None,
        },
    );
    Ok(status)
}

/// 停止当前在线隧道。
#[tauri::command(rename_all = "snake_case")]
pub async fn online_tunnel_stop(
    app: AppHandle,
    forwarder: State<'_, OnlineTunnelEventForwarder>,
    services: State<'_, AppServices>,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    // 停止后状态归 idle、mode 为空，需在停止前记录运行角色。
    let mode = services
        .online_tunnel()
        .status()
        .await
        .ok()
        .and_then(|status| status.mode);
    let status = services.online_tunnel().stop().await?;
    if let Some(mode) = mode {
        // 先发 Stopped 再停止转发，保证该事件能落达前端。
        emit_lifecycle_event(&app, OnlineTunnelEvent::Stopped { mode });
    }
    forwarder.clear().await;
    Ok(status)
}

/// 查询当前在线隧道的运行状态。
#[tauri::command(rename_all = "snake_case")]
pub async fn online_tunnel_status(
    services: State<'_, AppServices>,
) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
    services.online_tunnel().status().await
}
