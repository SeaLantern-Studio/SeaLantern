//! 服务器进程管理 Tauri 命令。

use std::sync::Arc;

use sealantern_application::port::ServerService;
use sealantern_application::service::CoreServerService;
use sealantern_application::services::AppServices;
use sealantern_contract::ServerServiceError;
use sealantern_contract::server::ServerSnapshot;
use sealantern_core::instance::InstanceId;
use tauri::State;

/// 获取宿主注入的服务器进程管理服务句柄。
fn server_service(services: &AppServices) -> Arc<CoreServerService> {
    services.server().clone()
}

/// 解析 Tauri 命令传入的实例 ID 字符串。
///
/// 统一映射解析错误为 [`ServerServiceError::InvalidInput`]。
fn parse_id_for_tauri(id: String) -> Result<InstanceId, ServerServiceError> {
    InstanceId::new(id).map_err(|_| ServerServiceError::InvalidInput)
}

/// 查询服务器进程状态。
#[tauri::command(rename_all = "snake_case")]
pub async fn server_status(
    services: State<'_, AppServices>,
    id: String,
) -> Result<ServerSnapshot, ServerServiceError> {
    let service = server_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.status(&id).await
}

/// 启动服务器进程。
#[tauri::command(rename_all = "snake_case")]
pub async fn start_server(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), ServerServiceError> {
    let service = server_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.start(&id).await
}

/// 重启服务器进程。
#[tauri::command(rename_all = "snake_case")]
pub async fn restart_server(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), ServerServiceError> {
    let service = server_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.restart(&id).await
}

/// 优雅停止服务器进程。
#[tauri::command(rename_all = "snake_case")]
pub async fn stop_server(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), ServerServiceError> {
    let service = server_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.stop(&id).await
}

/// 强制停止服务器进程（终止进程树）。
#[tauri::command(rename_all = "snake_case")]
pub async fn force_stop_server(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), ServerServiceError> {
    let service = server_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.force_stop(&id).await
}

/// 向服务器控制台发送单行命令。
#[tauri::command(rename_all = "snake_case")]
pub async fn send_server_command(
    services: State<'_, AppServices>,
    id: String,
    command: String,
) -> Result<(), ServerServiceError> {
    let service = server_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.send_command(&id, &command).await
}
