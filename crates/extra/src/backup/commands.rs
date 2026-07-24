use std::path::PathBuf;

use super::error::BackupResult;
use super::manager::BackupManager;
use super::models::{BackupItem, BackupSettings, CreateBackupRequest};
use super::settings::BackupSettingsManager;

/// 获取备份列表
pub async fn get_backup_list(server_id: String) -> BackupResult<Vec<BackupItem>> {
    let manager = BackupManager::new()?;
    manager.get_backup_list(&server_id)
}

/// 创建备份
pub async fn create_backup(
    request: CreateBackupRequest,
    server_dir: PathBuf,
    check_server_stopped: impl Fn(&str) -> bool + Send + 'static,
) -> BackupResult<BackupItem> {
    let manager = BackupManager::new()?;
    
    // 在阻塞任务中执行备份
    tokio::task::spawn_blocking(move || {
        manager.create_backup(request, &server_dir, check_server_stopped)
    })
    .await
    .unwrap() // unwrap 是安全的，因为我们的闭包不会 panic
}

/// 删除备份
pub async fn delete_backup(backup_id: String) -> BackupResult<()> {
    let manager = BackupManager::new()?;
    manager.delete_backup(&backup_id)
}

/// 恢复备份
pub async fn restore_backup(
    backup_id: String,
    server_dir: PathBuf,
    check_server_stopped: impl Fn(&str) -> bool + Send + 'static,
) -> BackupResult<()> {
    let manager = BackupManager::new()?;
    
    // 在阻塞任务中执行恢复
    tokio::task::spawn_blocking(move || {
        manager.restore_backup(&backup_id, &server_dir, check_server_stopped)
    })
    .await
    .unwrap()
}

/// 获取备份设置
pub async fn get_backup_settings(server_id: String) -> BackupResult<BackupSettings> {
    let manager = BackupSettingsManager::new()?;
    manager.get_backup_settings(&server_id)
}

/// 更新备份设置
pub async fn update_backup_settings(
    server_id: String,
    settings: BackupSettings,
) -> BackupResult<()> {
    let manager = BackupSettingsManager::new()?;
    manager.update_backup_settings(&server_id, settings)
}

// ============= Tauri Command 定义（需要配合 tauri::command 宏使用）=============

/// Tauri Command: 获取备份列表
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn get_backup_list_cmd(server_id: String) -> Result<Vec<BackupItem>, String> {
    get_backup_list(server_id).await.map_err(|e| e.to_string())
}

/// Tauri Command: 创建备份
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn create_backup_cmd(
    request: CreateBackupRequest,
    server_dir: String,
) -> Result<BackupItem, String> {
    // 注意：需要从应用状态获取服务器停止检查函数
    // 这里需要修改为接受一个状态检查回调
    create_backup(
        request,
        PathBuf::from(server_dir),
        |_server_id| true, // 默认实现，需要从应用状态获取真实实现
    )
    .await
    .map_err(|e| e.to_string())
}

/// Tauri Command: 删除备份
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn delete_backup_cmd(backup_id: String) -> Result<(), String> {
    delete_backup(backup_id).await.map_err(|e| e.to_string())
}

/// Tauri Command: 恢复备份
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn restore_backup_cmd(
    backup_id: String,
    server_dir: String,
) -> Result<(), String> {
    restore_backup(
        backup_id,
        PathBuf::from(server_dir),
        |_server_id| true, // 默认实现，需要从应用状态获取真实实现
    )
    .await
    .map_err(|e| e.to_string())
}

/// Tauri Command: 获取备份设置
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn get_backup_settings_cmd(server_id: String) -> Result<BackupSettings, String> {
    get_backup_settings(server_id).await.map_err(|e| e.to_string())
}

/// Tauri Command: 更新备份设置
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn update_backup_settings_cmd(
    server_id: String,
    settings: BackupSettings,
) -> Result<(), String> {
    update_backup_settings(server_id, settings)
        .await
        .map_err(|e| e.to_string())
}