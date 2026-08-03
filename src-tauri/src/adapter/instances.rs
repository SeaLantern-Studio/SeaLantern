//! 服务器实例管理的 Tauri 传输适配命令。
//!
//! 直接对接自托管的 [`AppServices`]（全局单例，非 Tauri State），
//! 把实例管理的查询/CRUD 暴露为前端可调的 Tauri 命令。
//! 后续可在保持命令签名的前提下切换为经 RPC 契约的 `dispatch` 调用。

use sealantern_core::instance::{Instance, InstanceId};
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
