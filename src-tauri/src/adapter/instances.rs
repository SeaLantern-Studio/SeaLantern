//! 服务器实例管理的 Tauri 传输适配命令。
//!
//! 命令层只做三件事：经 [`AppServices::instance_service`] 拿到服务句柄、
//! 用 [`super::super::services::rpc`] 提供的方法对象与 request 装配、把
//! dispatch 结果映射为 Tauri 命令返回值。权限与请求上下文集中由
//! `services::rpc` 统一构造，不在本层分散。

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};

use crate::services::rpc::instances::{
    InstanceCreate, InstanceDelete, InstanceGet, InstanceList, InstanceRename, InstanceUpdatePath,
    RenameRequest, UpdatePathRequest,
};
use crate::services::rpc::{rpc_error_message, tauri_request};
use crate::services::AppServices;

/// 列出全部实例。
#[tauri::command]
pub async fn server_instance_list() -> Result<Vec<Instance>, String> {
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    let response =
        sealantern_server::rpc::dispatch(&InstanceList::new(instance), tauri_request(()))
            .await
            .map_err(rpc_error_message)?;
    Ok(response.into_data())
}

/// 按 ID 查找实例，不存在返回 `None`。
#[tauri::command]
pub async fn server_instance_get(id: String) -> Result<Option<Instance>, String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    let response = sealantern_server::rpc::dispatch(&InstanceGet::new(instance), tauri_request(id))
        .await
        .map_err(rpc_error_message)?;
    Ok(response.into_data())
}

/// 创建新实例并持久化。
#[tauri::command]
pub async fn server_instance_create(spec: InstanceSpec) -> Result<Instance, String> {
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    let response =
        sealantern_server::rpc::dispatch(&InstanceCreate::new(instance), tauri_request(spec))
            .await
            .map_err(rpc_error_message)?;
    Ok(response.into_data())
}

/// 删除实例，返回是否确实删除了某个实例。
#[tauri::command]
pub async fn server_instance_delete(id: String) -> Result<bool, String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    let response =
        sealantern_server::rpc::dispatch(&InstanceDelete::new(instance), tauri_request(id))
            .await
            .map_err(rpc_error_message)?;
    Ok(response.into_data())
}

/// 重命名实例。
#[tauri::command]
pub async fn server_instance_rename(id: String, name: String) -> Result<(), String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    let params = RenameRequest { id, name };
    let response =
        sealantern_server::rpc::dispatch(&InstanceRename::new(instance), tauri_request(params))
            .await
            .map_err(rpc_error_message)?;
    response.into_data();
    Ok(())
}

/// 更新实例目录路径。
#[tauri::command]
pub async fn server_instance_update_path(id: String, path: String) -> Result<(), String> {
    let id = InstanceId::new(&id).map_err(|_| "invalid instance id".to_string())?;
    let instance = AppServices::instance_service()
        .await
        .map_err(|e| e.to_string())?;
    let params = UpdatePathRequest { id, path };
    let response =
        sealantern_server::rpc::dispatch(&InstanceUpdatePath::new(instance), tauri_request(params))
            .await
            .map_err(rpc_error_message)?;
    response.into_data();
    Ok(())
}
