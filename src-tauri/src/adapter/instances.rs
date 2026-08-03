//! 服务器实例管理的 Tauri 传输适配命令。
//!
//! 直接对接自托管的 [`AppServices`]（全局单例，非 Tauri State），
//! 把实例管理的查询/CRUD 暴露为前端可调的 Tauri 命令。
//! 后续可在保持命令签名的前提下切换为经 RPC 契约的 `dispatch` 调用。

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_server::rpc::traits::InstanceService;

use crate::services::AppServices;

/// 列出全部实例。
#[tauri::command]
pub async fn server_instance_list() -> Result<Vec<Instance>, String> {
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    instance.list().await.map_err(|e| e.to_string())
}

/// 按 ID 查找实例，不存在返回 `None`。
#[tauri::command]
pub async fn server_instance_get(id: String) -> Result<Option<Instance>, String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    instance.find(&id).await.map_err(|e| e.to_string())
}

/// 创建新实例并持久化。
#[tauri::command]
pub async fn server_instance_create(spec: InstanceSpec) -> Result<Instance, String> {
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    instance.create(spec).await.map_err(|e| e.to_string())
}

/// 删除实例，返回是否确实删除了某个实例。
#[tauri::command]
pub async fn server_instance_delete(id: String) -> Result<bool, String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    instance.delete(&id).await.map_err(|e| e.to_string())
}

/// 重命名实例。
#[tauri::command]
pub async fn server_instance_rename(id: String, name: String) -> Result<(), String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    instance.rename(&id, &name).await.map_err(|e| e.to_string())
}

/// 更新实例目录路径。
#[tauri::command]
pub async fn server_instance_update_path(id: String, path: String) -> Result<(), String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    instance
        .update_path(&id, &path)
        .await
        .map_err(|e| e.to_string())
}
