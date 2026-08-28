//! 备份管理 Tauri 命令。

use sealantern_application::port::{InstanceService, ServerService};
use sealantern_application::services::AppServices;
use sealantern_contract::server::ServerState;
use sealantern_core::instance::InstanceId;
use sealantern_feature::backup::{BackupItem, BackupSettings, CreateBackupRequest};
use tauri::State;

/// 获取备份列表
#[tauri::command]
pub async fn get_backup_list(server_id: String) -> Result<Vec<BackupItem>, String> {
    sealantern_feature::backup::get_backup_list(server_id)
        .await
        .map_err(|e| e.to_string())
}

/// 创建备份
#[tauri::command]
pub async fn create_backup(
    services: State<'_, AppServices>,
    request: CreateBackupRequest,
) -> Result<BackupItem, String> {
    // 解析实例 ID
    let instance_id = InstanceId::new(request.server_id.clone()).map_err(|e| e.to_string())?;

    // 查找实例获取服务器目录
    let instance = services
        .instance()
        .find(&instance_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("服务器实例不存在: {}", request.server_id))?;

    // 检查服务器是否正在运行
    let server_status = services
        .server()
        .status(&instance_id)
        .await
        .map_err(|e| e.to_string())?;

    if server_status.state != ServerState::Stopped {
        return Err(format!(
            "服务器正在运行，无法创建备份。请先停止服务器。当前状态: {:?}",
            server_status.state
        ));
    }

    // 执行备份
    sealantern_feature::backup::create_backup(
        request,
        instance.directory.clone(),
        |_server_id| true, // 已验证服务器已停止
    )
    .await
    .map_err(|e| e.to_string())
}

/// 删除备份
#[tauri::command]
pub async fn delete_backup(backup_id: String) -> Result<(), String> {
    sealantern_feature::backup::delete_backup(backup_id)
        .await
        .map_err(|e| e.to_string())
}

/// 恢复备份
#[tauri::command]
pub async fn restore_backup(
    services: State<'_, AppServices>,
    backup_id: String,
    server_id: String,
) -> Result<(), String> {
    // 解析实例 ID
    let instance_id = InstanceId::new(server_id.clone()).map_err(|e| e.to_string())?;

    // 查找实例获取服务器目录
    let instance = services
        .instance()
        .find(&instance_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("服务器实例不存在: {}", server_id))?;

    // 检查服务器是否正在运行
    let server_status = services
        .server()
        .status(&instance_id)
        .await
        .map_err(|e| e.to_string())?;

    if server_status.state != ServerState::Stopped {
        return Err(format!(
            "服务器正在运行，无法恢复备份。请先停止服务器。当前状态: {:?}",
            server_status.state
        ));
    }

    // 执行恢复
    sealantern_feature::backup::restore_backup(
        backup_id,
        instance.directory.clone(),
        |_server_id| true, // 已验证服务器已停止
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取备份设置
#[tauri::command]
pub async fn get_backup_settings(server_id: String) -> Result<BackupSettings, String> {
    sealantern_feature::backup::get_backup_settings(server_id)
        .await
        .map_err(|e| e.to_string())
}

/// 更新备份设置
#[tauri::command]
pub async fn update_backup_settings(
    server_id: String,
    settings: BackupSettings,
) -> Result<(), String> {
    sealantern_feature::backup::update_backup_settings(server_id, settings)
        .await
        .map_err(|e| e.to_string())
}
