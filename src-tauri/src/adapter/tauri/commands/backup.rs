//! 备份管理 Tauri 命令。

use sealantern_application::port::BackupService;
use sealantern_application::services::AppServices;
use sealantern_contract::BackupServiceError;
use sealantern_contract::backup::{BackupItem, BackupSettings, CreateBackupRequest};
use tauri::State;

/// 获取备份列表
#[tauri::command]
pub async fn get_backup_list(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<Vec<BackupItem>, BackupServiceError> {
    services.backup().list(&server_id).await
}

/// 创建备份
#[tauri::command]
pub async fn create_backup(
    services: State<'_, AppServices>,
    request: CreateBackupRequest,
) -> Result<BackupItem, BackupServiceError> {
    services.backup().create(request).await
}

/// 删除备份
#[tauri::command]
pub async fn delete_backup(
    services: State<'_, AppServices>,
    backup_id: String,
) -> Result<(), BackupServiceError> {
    services.backup().delete(&backup_id).await
}

/// 恢复备份
#[tauri::command]
pub async fn restore_backup(
    services: State<'_, AppServices>,
    backup_id: String,
    server_id: String,
) -> Result<(), BackupServiceError> {
    services.backup().restore(&backup_id, &server_id).await
}

/// 获取备份设置
#[tauri::command]
pub async fn get_backup_settings(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<BackupSettings, BackupServiceError> {
    services.backup().settings(&server_id).await
}

/// 更新备份设置
#[tauri::command]
pub async fn update_backup_settings(
    services: State<'_, AppServices>,
    server_id: String,
    settings: BackupSettings,
) -> Result<(), BackupServiceError> {
    services
        .backup()
        .update_settings(&server_id, settings)
        .await
}
