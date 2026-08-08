//! 服务器进程管理（后端语义 API，保留参考）。
//!
//! 前端命令名已由 `compat` 兼容层接管（`start_server`/`stop_server`/...），
//! 本文件保留新命名的服务调用函数作为后端语义 API，不注册为 Tauri 命令，
//! 供未来前端适配新版本命令时使用。
#![allow(dead_code)]

use std::sync::Arc;

use sealantern_application::service::CoreServerService;
use sealantern_application::services::AppServices;
use sealantern_core::instance::InstanceId;
use sealantern_interface::server::ServerSnapshot;
use sealantern_interface::{ServerService, ServerServiceError};

/// 获取全局服务器进程管理服务句柄（惰性初始化容器）。
async fn server_service() -> Result<Arc<CoreServerService>, ServerServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| ServerServiceError::OperationFailed)?;
    Ok(services.server().clone())
}

/// 解析 Tauri 命令传入的实例 ID 字符串。
///
/// 统一映射解析错误为 [`ServerServiceError::InvalidInput`]。
fn parse_id_for_tauri(id: String) -> Result<InstanceId, ServerServiceError> {
    InstanceId::new(id).map_err(|_| ServerServiceError::InvalidInput)
}

/// 查询服务器进程状态。
pub async fn server_status(id: String) -> Result<ServerSnapshot, ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.status(&id).await
}

/// 启动服务器进程。
pub async fn start_server(id: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.start(&id).await
}

/// 优雅停止服务器进程。
pub async fn stop_server(id: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.stop(&id).await
}

/// 强制停止服务器进程（终止进程树）。
pub async fn force_stop_server(id: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.force_stop(&id).await
}

/// 向服务器控制台发送单行命令。
pub async fn send_server_command(id: String, command: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.send_command(&id, &command).await
}
