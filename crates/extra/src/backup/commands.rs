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