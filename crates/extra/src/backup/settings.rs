use std::fs;
use std::path::PathBuf;

use sealantern_infra::fs::get_app_data_dir;
use tracing::{debug, error, info};

use super::error::{BackupError, BackupResult};
use super::models::BackupSettings;

/// 备份设置管理器
pub struct BackupSettingsManager {
    settings_dir: PathBuf,
}

impl BackupSettingsManager {
    /// 创建新的备份设置管理器
    pub fn new() -> BackupResult<Self> {
        let app_data_dir = get_app_data_dir();
        let settings_dir = app_data_dir.join("backup_settings");
        
        // 确保设置目录存在
        fs::create_dir_all(&settings_dir)
            .map_err(|_| BackupError::CannotCreateBackupDir(settings_dir.clone()))?;
        
        debug!("备份设置管理器初始化完成，设置目录: {:?}", settings_dir);
        
        Ok(Self { settings_dir })
    }
    
    /// 获取服务器设置文件路径
    fn get_settings_file_path(&self, server_id: &str) -> PathBuf {
        self.settings_dir.join(format!("{}.json", server_id))
    }
    
    /// 获取备份设置
    pub fn get_backup_settings(&self, server_id: &str) -> BackupResult<BackupSettings> {
        let path = self.get_settings_file_path(server_id);
        
        if !path.exists() {
            debug!("服务器 {} 没有备份设置，使用默认设置", server_id);
            return Ok(BackupSettings::default());
        }
        
        let content = fs::read_to_string(&path)
            .map_err(|e| {
                error!("无法读取备份设置 {:?}: {}", path, e);
                e
            })?;
        
        let settings: BackupSettings = serde_json::from_str(&content)
            .map_err(|e| {
                error!("无法解析备份设置 {:?}: {}", path, e);
                e
            })?;
        
        info!("获取服务器 {} 的备份设置", server_id);
        Ok(settings)
    }
    
    /// 更新备份设置
    pub fn update_backup_settings(
        &self,
        server_id: &str,
        settings: super::models::BackupSettings,
    ) -> BackupResult<()> {
        // 验证设置
        self.validate_settings(&settings)?;
        
        let path = self.get_settings_file_path(server_id);
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| {
                error!("无法序列化备份设置: {}", e);
                e
            })?;
        
        fs::write(&path, content)
            .map_err(|e| {
                error!("无法写入备份设置 {:?}: {}", path, e);
                e
            })?;
        
        info!("更新服务器 {} 的备份设置", server_id);
        Ok(())
    }
    
    /// 验证备份设置
    fn validate_settings(&self, settings: &BackupSettings) -> BackupResult<()> {
        // 验证最大备份数量
        if settings.max_backups < 1 || settings.max_backups > 50 {
            return Err(BackupError::Validation(
                "最大备份数量必须在1-50之间".to_string(),
            ));
        }
        
        // 验证自动备份间隔
        if settings.auto_backup_interval < 1 {
            return Err(BackupError::Validation(
                "自动备份间隔必须至少为1小时".to_string(),
            ));
        }
        
        // 验证自动备份内容不为空（如果启用了自动备份）
        if settings.auto_backup_enabled && settings.auto_backup_contents.is_empty() {
            return Err(BackupError::Validation(
                "启用自动备份时必须指定备份内容".to_string(),
            ));
        }
        
        Ok(())
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
        let manager = BackupSettingsManager::new().unwrap();
        
        // 测试有效的设置
        let valid_settings = BackupSettings::default();
        assert!(manager.validate_settings(&valid_settings).is_ok());
        
        // 测试无效的最大备份数量
        let mut invalid_settings = BackupSettings::default();
        invalid_settings.max_backups = 100;
        assert!(manager.validate_settings(&invalid_settings).is_err());
        
        // 测试空内容
        let mut empty_content_settings = BackupSettings::default();
        empty_content_settings.auto_backup_enabled = true;
        empty_content_settings.auto_backup_contents = vec![];
        assert!(manager.validate_settings(&empty_content_settings).is_err());
    }
}