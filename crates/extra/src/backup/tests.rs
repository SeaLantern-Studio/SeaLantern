#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    
    use tempfile::tempdir;
    use uuid::Uuid;
    
    use super::super::manager::BackupManager;
    use super::super::models::*;
    use super::super::error::BackupError;
    
    fn create_test_server_dir(temp_dir: &Path) -> PathBuf {
        let server_dir = temp_dir.join("test_server");
        fs::create_dir_all(&server_dir).unwrap();
        
        // 创建一些测试文件
        fs::write(server_dir.join("server.properties"), "motd=Test Server").unwrap();
        fs::write(server_dir.join("server.jar"), "fake jar content").unwrap();
        
        // 创建 config 目录
        let config_dir = server_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.yml"), "test: value").unwrap();
        
        // 创建 world 目录
        let world_dir = server_dir.join("world");
        fs::create_dir_all(&world_dir).unwrap();
        fs::write(world_dir.join("level.dat"), "level data").unwrap();
        
        // 创建 plugins 目录
        let plugins_dir = server_dir.join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(plugins_dir.join("test.jar"), "plugin content").unwrap();
        
        server_dir
    }
    
    #[test]
    fn test_backup_manager_creation() {
        let manager = BackupManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_create_and_list_backup() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path());
        
        let manager = BackupManager::new().unwrap();
        let server_id = format!("test-server-002-{}", Uuid::new_v4());
        
        // 创建备份
        let request = CreateBackupRequest {
            server_id: server_id.to_string(),
            contents: vec![BackupContentType::Core, BackupContentType::World],
            format: BackupFormat::Zip,
            compression_level: CompressionLevel::Medium,
            name: Some("test-backup".to_string()),
        };
        
        let backup = manager.create_backup(
            request,
            &server_dir,
            |_server_id| true, // 服务器已停止
        ).unwrap();
        
        assert_eq!(backup.server_id, server_id);
        assert_eq!(backup.format, BackupFormat::Zip);
        assert!(backup.size > 0);
        
        // 获取备份列表
        let backups = manager.get_backup_list(&server_id).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, backup.id);
    }
    
    #[test]
    fn test_delete_backup() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path());
        
        let manager = BackupManager::new().unwrap();
        let server_id = format!("test-server-002-{}", uuid::Uuid::new_v4());
        
        // 创建备份
        let request = CreateBackupRequest {
            server_id: server_id.to_string(),
            contents: vec![BackupContentType::Core],
            format: BackupFormat::Zip,
            compression_level: CompressionLevel::Low,
            name: None,
        };
        
        let backup = manager.create_backup(
            request,
            &server_dir,
            |_server_id| true,
        ).unwrap();
        
        // 确认备份存在
        let backups = manager.get_backup_list(&server_id).unwrap();
        assert_eq!(backups.len(), 1);
        
        // 删除备份
        manager.delete_backup(&backup.id).unwrap();
        
        // 确认备份已被删除
        let backups = manager.get_backup_list(&server_id).unwrap();
        assert_eq!(backups.len(), 0);
    }
    
    #[test]
    fn test_restore_backup() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path());
        
        let manager = BackupManager::new().unwrap();
        let server_id = format!("test-server-005-{}", Uuid::new_v4());
        
        // 创建备份
        let request = CreateBackupRequest {
            server_id: server_id.to_string(),
            contents: vec![BackupContentType::Core, BackupContentType::World],
            format: BackupFormat::Zip,
            compression_level: CompressionLevel::Medium,
            name: None,
        };
        
        let backup = manager.create_backup(
            request,
            &server_dir,
            |_server_id| true,
        ).unwrap();
        
        // 修改服务器文件
        fs::write(server_dir.join("server.properties"), "motd=Modified").unwrap();
        
        // 恢复备份
        manager.restore_backup(
            &backup.id,
            &server_dir,
            |_server_id| true,
        ).unwrap();
        
        // 验证文件已恢复
        let content = fs::read_to_string(server_dir.join("server.properties")).unwrap();
        assert_eq!(content, "motd=Test Server");
    }
    
    #[test]
    fn test_server_running_check() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path());
        
        let manager = BackupManager::new().unwrap();
        let server_id = format!("test-server-004-{}", uuid::Uuid::new_v4());
        
        let request = CreateBackupRequest {
            server_id: server_id.to_string(),
            contents: vec![BackupContentType::Core],
            format: BackupFormat::Zip,
            compression_level: CompressionLevel::Medium,
            name: None,
        };
        
        // 服务器正在运行
        let result = manager.create_backup(
            request,
            &server_dir,
            |_server_id| false, // 服务器正在运行
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            BackupError::ServerRunning(id) => assert_eq!(id, server_id),
            _ => panic!("期望 ServerRunning 错误"),
        }
    }
    
    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path());
        
        let manager = BackupManager::new().unwrap();
        let server_id = format!("test-server-005-{}", uuid::Uuid::new_v4());
        
        // 创建3个备份
        for i in 0..3 {
            let request = CreateBackupRequest {
                server_id: server_id.to_string(),
                contents: vec![BackupContentType::Core],
                format: BackupFormat::Zip,
                compression_level: CompressionLevel::Low,
                name: Some(format!("backup-{}", i)),
            };
            
            manager.create_backup(
                request,
                &server_dir,
                |_server_id| true,
            ).unwrap();
            
            // 稍微延迟以确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        // 确认有3个备份
        let backups = manager.get_backup_list(&server_id).unwrap();
        assert_eq!(backups.len(), 3);
        
        // 清理，只保留2个最新的
        let removed = manager.cleanup_old_backups(&server_id, 2).unwrap();
        assert_eq!(removed.len(), 1);
        
        // 确认只剩2个备份
        let backups = manager.get_backup_list(&server_id).unwrap();
        assert_eq!(backups.len(), 2);
    }
}