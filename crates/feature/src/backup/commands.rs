use std::path::PathBuf;

use sealantern_infra::platform::AppLayout;

use super::error::BackupResult;
use super::manager::BackupManager;
use super::models::{BackupDirectory, BackupItem, BackupSettings, CreateBackupRequest};
use super::settings::BackupSettingsManager;

/// 获取备份列表
pub async fn get_backup_list(
    server_id: String,
    layout: AppLayout,
) -> BackupResult<Vec<BackupItem>> {
    tokio::task::spawn_blocking(move || {
        let manager = BackupManager::new(&layout)?;
        manager.get_backup_list(&server_id)
    })
    .await?
}

/// 获取备份存储目录（根目录，以及传入服务器 ID 时的服务器备份子目录）
pub async fn get_backup_dir(
    server_id: Option<String>,
    layout: AppLayout,
) -> BackupResult<BackupDirectory> {
    tokio::task::spawn_blocking(move || {
        let manager = BackupManager::new(&layout)?;
        manager.get_backup_directory(server_id.as_deref())
    })
    .await?
}

/// 创建备份
pub async fn create_backup(
    request: CreateBackupRequest,
    server_dir: PathBuf,
    layout: AppLayout,
    check_server_stopped: impl Fn(&str) -> bool + Send + 'static,
) -> BackupResult<BackupItem> {
    // 在阻塞任务中执行备份
    tokio::task::spawn_blocking(move || {
        let manager = BackupManager::new(&layout)?;
        manager.create_backup(request, &server_dir, check_server_stopped)
    })
    .await?
}

/// 删除备份
pub async fn delete_backup(backup_id: String, layout: AppLayout) -> BackupResult<()> {
    tokio::task::spawn_blocking(move || {
        let manager = BackupManager::new(&layout)?;
        manager.delete_backup(&backup_id)
    })
    .await?
}

/// 恢复备份
pub async fn restore_backup(
    backup_id: String,
    server_id: String,
    server_dir: PathBuf,
    layout: AppLayout,
    check_server_stopped: impl Fn(&str) -> bool + Send + 'static,
) -> BackupResult<()> {
    // 在阻塞任务中执行恢复
    tokio::task::spawn_blocking(move || {
        let manager = BackupManager::new(&layout)?;
        manager.restore_backup(&backup_id, &server_id, &server_dir, check_server_stopped)
    })
    .await?
}

/// 获取备份设置
pub async fn get_backup_settings(
    server_id: String,
    layout: AppLayout,
) -> BackupResult<BackupSettings> {
    let manager = BackupSettingsManager::new(&layout)?;
    manager.get_backup_settings(&server_id).await
}

/// 更新备份设置
pub async fn update_backup_settings(
    server_id: String,
    settings: BackupSettings,
    layout: AppLayout,
) -> BackupResult<()> {
    let manager = BackupSettingsManager::new(&layout)?;
    manager.update_backup_settings(&server_id, settings).await
}
