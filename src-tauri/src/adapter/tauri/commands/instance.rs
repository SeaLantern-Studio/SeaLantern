//! 实例管理 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`CoreInstanceService`](sealantern_application::service::CoreInstanceService)
//! 执行查询与 CRUD。
//!
//! 错误统一为接口契约错误 [`InstanceServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use std::sync::Arc;

use sealantern_application::error::InstanceError;
use sealantern_application::service::CoreInstanceService;
use sealantern_application::services::AppServices;
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_interface::{InstanceService, InstanceServiceError};

/// 获取全局实例管理服务句柄（惰性初始化容器）。
///
/// 应用层主错误 [`InstanceError`] 收敛为契约错误 [`InstanceServiceError`]。
async fn instance_service() -> Result<Arc<CoreInstanceService>, InstanceServiceError> {
    let services = AppServices::get().await?;
    Ok(services.instance().clone())
}

/// 解析 Tauri 命令传入的实例 ID 字符串。
///
/// 统一映射解析错误为 [`InstanceServiceError::InvalidInput`]，避免各命令重复
/// 内联解析与错误映射；后续若调整非法输入的错误变体，只需修改此处。
fn parse_id_for_tauri(id: String) -> Result<InstanceId, InstanceServiceError> {
    InstanceId::new(id)
        .map_err(InstanceError::from)
        .map_err(InstanceServiceError::from)
}

/// 列出全部实例。
#[tauri::command]
pub async fn list_instances() -> Result<Vec<Instance>, InstanceServiceError> {
    let service = instance_service().await?;
    service.list().await
}

/// 按 ID 查找实例，不存在时返回 `None`。
#[tauri::command]
pub async fn get_instance(id: String) -> Result<Option<Instance>, InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.find(&id).await
}

/// 创建新实例并持久化。
#[tauri::command]
pub async fn create_instance(spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
    let service = instance_service().await?;
    service.create(spec).await
}

/// 删除实例；实例不存在时返回 [`InstanceServiceError::InstanceNotFound`]。
#[tauri::command]
pub async fn delete_instance(id: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.delete(&id).await
}

/// 重命名实例。
#[tauri::command]
pub async fn rename_instance(id: String, name: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.rename(&id, &name).await
}

/// 更新实例目录路径。
#[tauri::command]
pub async fn update_instance_path(id: String, path: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.update_path(&id, &path).await
}
