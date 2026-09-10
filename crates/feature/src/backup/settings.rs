use std::fs;
use std::path::{Path, PathBuf};

use sealantern_infra::persistence::ConfigFile;
use sealantern_infra::platform::get_app_data_dir;
use tracing::{debug, error, info};

use super::error::{BackupError, BackupResult};
use super::models::{BackupSettings, is_safe_path_component};

const MAX_BACKUPS: u32 = 50;
const MAX_AUTO_BACKUP_INTERVAL: u32 = 720;

/// 备份设置管理器
pub struct BackupSettingsManager {
    settings_dir: PathBuf,
}

impl BackupSettingsManager {
    /// 创建新的备份设置管理器
    pub fn new() -> BackupResult<Self> {
        let app_data_dir = get_app_data_dir();
        let settings_dir = app_data_dir.join("backup_settings");

        Self::from_settings_dir(settings_dir)
    }

    fn from_settings_dir(settings_dir: PathBuf) -> BackupResult<Self> {
        fs::create_dir_all(&settings_dir)
            .map_err(|_| BackupError::CannotCreateBackupDir(settings_dir.clone()))?;

        let metadata = fs::symlink_metadata(&settings_dir)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(BackupError::Validation(format!(
                "设置路径不是普通目录: {:?}",
                settings_dir
            )));
        }

        debug!("备份设置管理器初始化完成，设置目录: {:?}", settings_dir);

        Ok(Self { settings_dir })
    }

    #[cfg(test)]
    pub(crate) fn new_at(settings_dir: PathBuf) -> BackupResult<Self> {
        Self::from_settings_dir(settings_dir)
    }

    /// 获取服务器设置文件路径
    fn get_settings_file_path(&self, server_id: &str) -> BackupResult<PathBuf> {
        validate_server_id(server_id)?;
        Ok(self.settings_dir.join(format!("{}.json", server_id)))
    }

    /// 获取备份设置
    pub async fn get_backup_settings(&self, server_id: &str) -> BackupResult<BackupSettings> {
        let path = self.get_settings_file_path(server_id)?;

        if !settings_file_exists(&path)? {
            debug!("服务器 {} 没有备份设置，使用默认设置", server_id);
            return Ok(BackupSettings::default());
        }

        let config = ConfigFile::<BackupSettings>::load(path.clone())
            .await
            .map_err(|e| {
                error!("无法读取备份设置 {:?}: {}", path, e);
                e
            })?;
        let settings = config.get().clone();
        Self::validate_settings(&settings)?;

        info!("获取服务器 {} 的备份设置", server_id);
        Ok(settings)
    }

    /// 更新备份设置
    pub async fn update_backup_settings(
        &self,
        server_id: &str,
        settings: BackupSettings,
    ) -> BackupResult<()> {
        Self::validate_settings(&settings)?;
        let path = self.get_settings_file_path(server_id)?;
        settings_file_exists(&path)?;
        ConfigFile::update_persisted(
            path.clone(),
            BackupSettings::default(),
            false,
            move |stored| {
                *stored = settings;
            },
        )
        .await
        .map_err(|e| {
            error!("无法写入备份设置 {:?}: {}", path, e);
            e
        })?;

        info!("更新服务器 {} 的备份设置", server_id);
        Ok(())
    }

    /// 验证备份设置
    fn validate_settings(settings: &BackupSettings) -> BackupResult<()> {
        // 验证最大备份数量
        if settings.max_backups < 1 {
            return Err(BackupError::Validation("最大备份数量必须至少为1".to_string()));
        }
        if settings.max_backups > MAX_BACKUPS {
            return Err(BackupError::Validation(format!("最大备份数量不能超过{}", MAX_BACKUPS)));
        }

        // 验证自动备份间隔
        if settings.auto_backup_interval < 1 {
            return Err(BackupError::Validation("自动备份间隔必须至少为1小时".to_string()));
        }
        if settings.auto_backup_interval > MAX_AUTO_BACKUP_INTERVAL {
            return Err(BackupError::Validation(format!(
                "自动备份间隔不能超过{}小时",
                MAX_AUTO_BACKUP_INTERVAL
            )));
        }

        // 验证自动备份内容不为空（如果启用了自动备份）
        if settings.auto_backup_enabled && settings.auto_backup_contents.is_empty() {
            return Err(BackupError::Validation("启用自动备份时必须指定备份内容".to_string()));
        }

        Ok(())
    }
}

fn validate_server_id(server_id: &str) -> BackupResult<()> {
    if is_safe_path_component(server_id) {
        Ok(())
    } else {
        Err(BackupError::Validation(format!("无效的服务器ID: {}", server_id)))
    }
}

fn settings_file_exists(path: &Path) -> BackupResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(BackupError::Validation(format!("备份设置文件不是普通文件: {:?}", path)))
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(BackupError::Io(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = BackupSettings::default();
        assert_eq!(settings.max_backups, 10);
        assert!(!settings.auto_backup_enabled);
        assert_eq!(settings.auto_backup_interval, 24);
    }

    #[test]
    fn test_settings_validation() {
        // 测试有效的设置
        let valid_settings = BackupSettings::default();
        assert!(BackupSettingsManager::validate_settings(&valid_settings).is_ok());

        // 测试无效的最大备份数量（小于1）
        let invalid_settings = BackupSettings {
            max_backups: 0,
            ..BackupSettings::default()
        };
        assert!(BackupSettingsManager::validate_settings(&invalid_settings).is_err());

        // 测试空内容
        let empty_content_settings = BackupSettings {
            auto_backup_enabled: true,
            auto_backup_contents: vec![],
            ..BackupSettings::default()
        };
        assert!(BackupSettingsManager::validate_settings(&empty_content_settings).is_err());

        let too_many_backups = BackupSettings {
            max_backups: 51,
            ..BackupSettings::default()
        };
        assert!(BackupSettingsManager::validate_settings(&too_many_backups).is_err());

        let too_frequent = BackupSettings {
            auto_backup_interval: 721,
            ..BackupSettings::default()
        };
        assert!(BackupSettingsManager::validate_settings(&too_frequent).is_err());
    }

    #[tokio::test]
    async fn test_settings_persisted_atomically() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = BackupSettingsManager::new_at(temp_dir.path().join("settings")).unwrap();
        let settings = BackupSettings {
            max_backups: 20,
            auto_backup_enabled: true,
            auto_backup_interval: 48,
            ..BackupSettings::default()
        };

        manager
            .update_backup_settings("server-1", settings.clone())
            .await
            .unwrap();
        assert_eq!(
            manager
                .get_backup_settings("server-1")
                .await
                .unwrap()
                .max_backups,
            20
        );
        assert_eq!(
            manager
                .get_backup_settings("server-1")
                .await
                .unwrap()
                .auto_backup_interval,
            settings.auto_backup_interval
        );
    }

    #[tokio::test]
    async fn test_settings_reject_unsafe_server_id() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = BackupSettingsManager::new_at(temp_dir.path().join("settings")).unwrap();
        assert!(matches!(
            manager.get_backup_settings("../outside").await,
            Err(BackupError::Validation(_))
        ));
        assert!(matches!(
            manager
                .update_backup_settings("server/../outside", BackupSettings::default())
                .await,
            Err(BackupError::Validation(_))
        ));
    }
}
