//! `src/api/server.ts` 对应的兼容命令。
//!
//! 前端使用旧命令名（如 `create_server`、`get_server_list`），本模块把这些旧名
//! 注册为 Tauri 命令，内部经 [`super::adapter`] 做参数/响应适配后调用
//! [`CoreInstanceService`](sealantern_application::service::CoreInstanceService)。
//!
//! 未对接后端的命令一律返回 [`InstanceServiceError::Unsupported`]，绝不静默 no-op。

use std::sync::Arc;

use sealantern_application::error::InstanceError;
use sealantern_application::service::{CoreInstanceService, CoreServerService};
use sealantern_application::services::AppServices;
use sealantern_core::instance::InstanceId;
use sealantern_interface::{
    InstanceService, InstanceServiceError, ServerService, ServerServiceError,
};

use super::adapter::{
    add_existing_params_to_spec, create_params_to_spec, instance_to_frontend,
    server_status_to_frontend,
};
use super::models::{
    AddExistingServerParams, CreateServerParams, ForceStopServerParams, FrontendServerInstance,
    FrontendServerStatusInfo, UpdateServerPathParams,
};

// ── 服务句柄获取（复用 instance.rs 的模式）────────────────────────────

/// 获取全局实例记录管理服务句柄（惰性初始化容器）。
///
/// 应用层主错误 [`InstanceError`] 经 `From` 收敛为契约错误 [`InstanceServiceError`]。
async fn instance_service() -> Result<Arc<CoreInstanceService>, InstanceServiceError> {
    let services = AppServices::get().await?;
    Ok(services.instance().clone())
}

/// 获取全局服务器进程管理服务句柄（惰性初始化容器）。
async fn server_service() -> Result<Arc<CoreServerService>, ServerServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| ServerServiceError::OperationFailed)?;
    Ok(services.server().clone())
}

/// 解析实例 ID 字符串。
fn parse_id(id: String) -> Result<InstanceId, InstanceServiceError> {
    InstanceId::new(id)
        .map_err(InstanceError::from)
        .map_err(InstanceServiceError::from)
}

/// 解析实例 ID 字符串（服务器进程命令用，错误为 `ServerServiceError`）。
fn parse_id_server(id: String) -> Result<InstanceId, ServerServiceError> {
    InstanceId::new(id).map_err(|_| ServerServiceError::InvalidInput)
}

// ── 可用命令（后端已对接）──────────────────────────────────────────────

/// 创建新服务器实例（兼容 `create_server`）。
#[tauri::command]
pub async fn create_server(
    params: CreateServerParams,
) -> Result<FrontendServerInstance, InstanceServiceError> {
    let spec = create_params_to_spec(params)?;
    let service = instance_service().await?;
    let instance = service.create(spec).await?;
    Ok(instance_to_frontend(&instance))
}

/// 添加已存在的服务器（兼容 `add_existing_server`）。
#[tauri::command]
pub async fn add_existing_server(
    params: AddExistingServerParams,
) -> Result<FrontendServerInstance, InstanceServiceError> {
    let spec = add_existing_params_to_spec(params)?;
    let service = instance_service().await?;
    let instance = service.create(spec).await?;
    Ok(instance_to_frontend(&instance))
}

/// 获取服务器列表（兼容 `get_server_list`）。
#[tauri::command]
pub async fn get_server_list() -> Result<Vec<FrontendServerInstance>, InstanceServiceError> {
    let service = instance_service().await?;
    let instances = service.list().await?;
    Ok(instances.iter().map(instance_to_frontend).collect())
}

/// 删除服务器（兼容 `delete_server`）。
#[tauri::command]
pub async fn delete_server(id: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id(id)?;
    service.delete(&id).await
}

/// 重命名服务器（兼容 `update_server_name`）。
#[tauri::command]
pub async fn update_server_name(id: String, name: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id(id)?;
    service.rename(&id, &name).await
}

/// 更新服务器路径（兼容 `update_server_path`）。
///
/// 第一阶段只改目录，丢弃 `newJarPath`/`newStartupMode`（待 Phase 2 扩展）。
#[tauri::command]
pub async fn update_server_path(
    params: UpdateServerPathParams,
) -> Result<FrontendServerInstance, InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id(params.id)?;
    service.update_path(&id, &params.new_path).await?;
    let instance = service
        .find(&id)
        .await?
        .ok_or(InstanceServiceError::InstanceNotFound)?;
    Ok(instance_to_frontend(&instance))
}

// ── 服务器进程生命周期（经 ServerService 接入）────────────────────────

/// 启动服务器（兼容 `start_server`）。
#[tauri::command]
pub async fn start_server(id: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_server(id)?;
    service.start(&id).await
}

/// 停止服务器（兼容 `stop_server`）。
#[tauri::command]
pub async fn stop_server(id: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_server(id)?;
    service.stop(&id).await
}

/// 强制停止服务器（兼容 `force_stop_server`，丢弃 confirmation_token）。
///
/// Phase 1 后端无 Daemon，无法验证 token；`prepare_force_stop_server` 同样返回
/// Unsupported，故此处 token 一并忽略。Phase 2 接入 Daemon 后需恢复 token 校验链路。
#[tauri::command]
pub async fn force_stop_server(params: ForceStopServerParams) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let id = parse_id_server(params.id)?;
    service.force_stop(&id).await
}

/// 获取服务器状态（兼容 `get_server_status`）。
#[tauri::command]
pub async fn get_server_status(id: String) -> Result<FrontendServerStatusInfo, ServerServiceError> {
    let service = server_service().await?;
    let instance_id = parse_id_server(id.clone())?;
    let status = service.status(&instance_id).await?;
    Ok(server_status_to_frontend(id, status))
}

/// 发送控制台命令（兼容 `send_command`）。
#[tauri::command]
pub async fn send_command(id: String, command: String) -> Result<(), ServerServiceError> {
    let service = server_service().await?;
    let instance_id = parse_id_server(id)?;
    service.send_command(&instance_id, &command).await
}

// ── 显式 Unsupported（后端能力未装配）─────────────────────────────────

/// 导入服务器（Phase 2 由 provisioning 接入）。
#[tauri::command]
pub async fn import_server() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 导入整合包（Phase 2 由 provisioning 接入）。
#[tauri::command]
pub async fn import_modpack() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 解析服务端核心类型（Phase 2 由 server_inspection 接入）。
#[tauri::command]
pub async fn parse_server_core_type() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 扫描启动候选（Phase 2 由 startup_parsing 接入）。
#[tauri::command]
pub async fn scan_startup_candidates() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 收集拷贝冲突（Phase 2 由 provisioning::copy 接入）。
#[tauri::command]
pub async fn collect_copy_conflicts() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 拷贝目录内容（Phase 2 由 provisioning::copy 接入）。
#[tauri::command]
pub async fn copy_directory_contents() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 准备强制停止（与 force_stop 一致返回 Unsupported，避免 token 流半通）。
#[tauri::command]
pub async fn prepare_force_stop_server() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 获取服务器日志（Phase 2 接 console 服务）。
#[tauri::command]
pub async fn get_server_logs() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}

/// 校验服务器路径（Phase 2 由 server_inspection 接入）。
#[tauri::command]
pub async fn validate_server_path() -> Result<(), InstanceServiceError> {
    Err(InstanceServiceError::Unsupported)
}
