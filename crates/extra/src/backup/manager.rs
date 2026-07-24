use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use sealantern_infra::archive::{create_zip, extract_zip};
use sealantern_infra::fs::get_app_data_dir;
use tracing::{debug, error, info, warn};

use super::error::{BackupError, BackupResult};
use super::models::*;

/// 备份管理器
pub struct BackupManager {
    backups_dir: PathBuf,
}

impl BackupManager {
    /// 创建新的备份管理器
    pub fn new() -> BackupResult<Self> {
        let app_data_dir = get_app_data_dir();
        let backups_dir = app_data_dir.join("backups");
        
        // 确保备份目录存在
        fs::create_dir_all(&backups_dir)
            .map_err(|_| BackupError::CannotCreateBackupDir(backups_dir.clone()))?;
        
        debug!("备份管理器初始化完成，备份目录: {:?}", backups_dir);
        
        Ok(Self { backups_dir })
    }
    
    /// 获取服务器的备份目录
    fn get_server_backup_dir(&self, server_id: &str) -> PathBuf {
        self.backups_dir.join(server_id)
    }
    
    /// 获取备份元数据文件路径
    fn get_backup_metadata_path(&self, server_id: &str, backup_id: &str) -> PathBuf {
        self.get_server_backup_dir(server_id)
            .join(format!("{}.json", backup_id))
    }
    
    /// 获取备份文件路径
    fn get_backup_file_path(&self, server_id: &str, backup_id: &str, format: BackupFormat) -> PathBuf {
        self.get_server_backup_dir(server_id)
            .join(format!("{}.{}", backup_id, format.extension()))
    }
    
    /// 生成备份ID
    fn generate_backup_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("backup-{}", timestamp)
    }
    
    /// 生成备份文件名
    fn generate_backup_name(server_id: &str, created_at: &DateTime<Utc>) -> String {
        format!(
            "{}_{}.backup",
            server_id,
            created_at.format("%Y%m%d_%H%M%S")
        )
    }
    
    /// 获取备份列表
    pub fn get_backup_list(&self, server_id: &str) -> BackupResult<Vec<BackupItem>> {
        let server_backup_dir = self.get_server_backup_dir(server_id);
        
        if !server_backup_dir.exists() {
            debug!("服务器备份目录不存在: {:?}", server_backup_dir);
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&server_backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match self.load_backup_metadata(&path) {
                    Ok(backup) => backups.push(backup),
                    Err(e) => {
                        warn!("无法加载备份元数据 {:?}: {}", path, e);
                        // 删除损坏的元数据文件
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
        
        // 按创建时间倒序排序
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        info!("获取服务器 {} 的备份列表，共 {} 个备份", server_id, backups.len());
        Ok(backups)
    }
    
    /// 加载备份元数据
    fn load_backup_metadata(&self, path: &Path) -> BackupResult<BackupItem> {
        let content = fs::read_to_string(path)?;
        let backup: BackupItem = serde_json::from_str(&content)?;
        Ok(backup)
    }
    
    /// 保存备份元数据
    fn save_backup_metadata(&self, backup: &BackupItem) -> BackupResult<()> {
        let path = self.get_backup_metadata_path(&backup.server_id, &backup.id);
        let content = serde_json::to_string_pretty(backup)?;
        fs::write(&path, content)?;
        debug!("保存备份元数据: {:?}", path);
        Ok(())
    }
    
    /// 创建备份（冷备份）
    pub fn create_backup(
        &self,
        request: CreateBackupRequest,
        server_dir: &Path,
        check_server_stopped: impl Fn(&str) -> bool,
    ) -> BackupResult<BackupItem> {
        // 检查服务器是否已停止
        if !check_server_stopped(&request.server_id) {
            error!("服务器 {} 正在运行，无法执行冷备份", request.server_id);
            return Err(BackupError::ServerRunning(request.server_id.clone()));
        }
        
        // 检查服务器目录是否存在
        if !server_dir.exists() {
            error!("服务器目录不存在: {:?}", server_dir);
            return Err(BackupError::ServerNotFound(request.server_id.clone()));
        }
        
        // 确保服务器备份目录存在
        let server_backup_dir = self.get_server_backup_dir(&request.server_id);
        fs::create_dir_all(&server_backup_dir)?;
        
        // 生成备份信息
        let backup_id = Self::generate_backup_id();
        let created_at = Utc::now();
        let name = request.name.clone().unwrap_or_else(|| {
            Self::generate_backup_name(&request.server_id, &created_at)
        });
        
        info!(
            "开始创建备份: ID={}, 服务器={}, 内容={:?}",
            backup_id, request.server_id, request.contents
        );
        
        // 创建临时目录用于准备备份内容
        let temp_dir = server_backup_dir.join(".temp").join(&backup_id);
        fs::create_dir_all(&temp_dir)?;
        
        // 准备要备份的文件
        self.prepare_backup_content(server_dir, &temp_dir, &request.contents)?;
        
        // 创建备份文件
        let backup_file = self.get_backup_file_path(&request.server_id, &backup_id, request.format);
        
        let result: Result<(), BackupError> = match request.format {
            BackupFormat::Zip => {
                create_zip(&temp_dir, &backup_file)
                    .map_err(|e| {
                        error!("创建ZIP备份失败: {}", e);
                        e
                    })?;
                Ok(())
            }
            BackupFormat::TarGz => {
                // TODO: 实现 tar.gz 支持
                // 目前先使用 ZIP
                warn!("tar.gz 格式暂未实现，使用 ZIP 代替");
                create_zip(&temp_dir, &backup_file)
                    .map_err(|e| {
                        error!("创建ZIP备份失败: {}", e);
                        e
                    })?;
                Ok(())
            }
        };
        
        // 清理临时目录
        if let Err(e) = fs::remove_dir_all(&temp_dir) {
            warn!("清理临时目录失败: {:?} - {}", temp_dir, e);
        }
        
        result?;
        
        // 获取备份文件大小
        let size = fs::metadata(&backup_file)?.len();
        
        // 创建备份元数据
        let backup_item = BackupItem {
            id: backup_id.clone(),
            server_id: request.server_id.clone(),
            name,
            format: request.format,
            size,
            created_at: created_at.to_rfc3339(),
            contents: request.contents,
        };
        
        // 保存备份元数据
        self.save_backup_metadata(&backup_item)?;
        
        info!(
            "备份创建成功: ID={}, 大小={}字节",
            backup_item.id, backup_item.size
        );
        
        Ok(backup_item)
    }
    
    /// 准备备份内容
    fn prepare_backup_content(
        &self,
        server_dir: &Path,
        temp_dir: &Path,
        contents: &[BackupContentType],
    ) -> BackupResult<()> {
        for content_type in contents {
            let source_path = match content_type {
                BackupContentType::Core => server_dir.to_path_buf(),
                BackupContentType::Config => server_dir.join("config"),
                BackupContentType::Plugins => server_dir.join("plugins"),
                BackupContentType::World => server_dir.join("world"),
                BackupContentType::Logs => server_dir.join("logs"),
            };
            
            if !source_path.exists() {
                debug!("跳过不存在的内容: {:?}", source_path);
                continue;
            }
            
            let dest_path = match content_type {
                BackupContentType::Core => temp_dir.to_path_buf(),
                BackupContentType::Config => temp_dir.join("config"),
                BackupContentType::Plugins => temp_dir.join("plugins"),
                BackupContentType::World => temp_dir.join("world"),
                BackupContentType::Logs => temp_dir.join("logs"),
            };
            
            debug!("复制 {:?} -> {:?}", source_path, dest_path);
            
            if source_path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.copy_dir_all(&source_path, &dest_path)?;
            } else {
                fs::create_dir_all(dest_path.parent().unwrap())?;
                fs::copy(&source_path, &dest_path)?;
            }
        }
        
        Ok(())
    }
    
    /// 递归复制目录
    fn copy_dir_all(&self, src: &Path, dst: &Path) -> BackupResult<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if ty.is_dir() {
                self.copy_dir_all(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
    
    /// 删除备份
    pub fn delete_backup(&self, backup_id: &str) -> BackupResult<()> {
        // 从备份ID中提取服务器ID（格式：backup-timestamp）
        // 我们需要扫描所有服务器的备份目录来找到这个备份
        let mut found = false;
        
        for entry in fs::read_dir(&self.backups_dir)? {
            let entry = entry?;
            let server_backup_dir = entry.path();
            
            if server_backup_dir.is_dir() {
                let metadata_path = server_backup_dir.join(format!("{}.json", backup_id));
                let backup_file_zip = server_backup_dir.join(format!("{}.zip", backup_id));
                let backup_file_tar = server_backup_dir.join(format!("{}.tar.gz", backup_id));
                
                if metadata_path.exists() {
                    // 删除元数据文件
                    fs::remove_file(&metadata_path)?;
                    debug!("删除备份元数据: {:?}", metadata_path);
                    
                    // 删除备份文件
                    if backup_file_zip.exists() {
                        fs::remove_file(&backup_file_zip)?;
                        debug!("删除备份文件: {:?}", backup_file_zip);
                    }
                    if backup_file_tar.exists() {
                        fs::remove_file(&backup_file_tar)?;
                        debug!("删除备份文件: {:?}", backup_file_tar);
                    }
                    
                    found = true;
                    info!("备份删除成功: {}", backup_id);
                    break;
                }
            }
        }
        
        if !found {
            return Err(BackupError::NotFound(backup_id.to_string()));
        }
        
        Ok(())
    }
    
    /// 恢复备份（冷恢复）
    pub fn restore_backup(
        &self,
        backup_id: &str,
        server_dir: &Path,
        check_server_stopped: impl Fn(&str) -> bool,
    ) -> BackupResult<()> {
        // 查找备份
        let backup = self.find_backup_by_id(backup_id)?;
        
        // 检查服务器是否已停止
        if !check_server_stopped(&backup.server_id) {
            error!("服务器 {} 正在运行，无法执行恢复", backup.server_id);
            return Err(BackupError::ServerRunning(backup.server_id.clone()));
        }
        
        // 检查备份文件是否存在
        let backup_file = self.get_backup_file_path(&backup.server_id, backup_id, backup.format);
        if !backup_file.exists() {
            error!("备份文件不存在: {:?}", backup_file);
            return Err(BackupError::CorruptedBackup(backup_file));
        }
        
        info!(
            "开始恢复备份: ID={}, 服务器={}, 格式={}",
            backup_id, backup.server_id, backup.format
        );
        
        // 创建临时目录用于解压备份
        let temp_base = tempfile::tempdir()
            .map_err(|e| BackupError::Io(e))?;
        let extract_dir = temp_base.path().join("extracted");
        
        // 解压备份文件
        match backup.format {
            BackupFormat::Zip => {
                extract_zip(&backup_file, &extract_dir)?;
            }
            BackupFormat::TarGz => {
                // TODO: 实现 tar.gz 支持
                warn!("tar.gz 格式暂未实现");
                extract_zip(&backup_file, &extract_dir)?;
            }
        }
        
        // 恢复备份内容
        self.restore_backup_content(&extract_dir, server_dir, &backup.contents)?;
        
        info!("备份恢复成功: {}", backup_id);
        
        Ok(())
    }
    
    /// 恢复备份内容
    fn restore_backup_content(
        &self,
        temp_dir: &Path,
        server_dir: &Path,
        contents: &[BackupContentType],
    ) -> BackupResult<()> {
        for content_type in contents {
            let source_path = match content_type {
                BackupContentType::Core => temp_dir.to_path_buf(),
                BackupContentType::Config => temp_dir.join("config"),
                BackupContentType::Plugins => temp_dir.join("plugins"),
                BackupContentType::World => temp_dir.join("world"),
                BackupContentType::Logs => temp_dir.join("logs"),
            };
            
            if !source_path.exists() {
                debug!("跳过不存在的备份内容: {:?}", source_path);
                continue;
            }
            
            let dest_path = match content_type {
                BackupContentType::Core => server_dir.to_path_buf(),
                BackupContentType::Config => server_dir.join("config"),
                BackupContentType::Plugins => server_dir.join("plugins"),
                BackupContentType::World => server_dir.join("world"),
                BackupContentType::Logs => server_dir.join("logs"),
            };
            
            debug!("恢复 {:?} -> {:?}", source_path, dest_path);
            
            // 删除现有内容
            if dest_path.exists() {
                fs::remove_dir_all(&dest_path)?;
            }
            
            // 复制备份内容
            if source_path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.copy_dir_all(&source_path, &dest_path)?;
            } else {
                fs::create_dir_all(dest_path.parent().unwrap())?;
                fs::copy(&source_path, &dest_path)?;
            }
        }
        
        Ok(())
    }
    
    /// 根据ID查找备份
    fn find_backup_by_id(&self, backup_id: &str) -> BackupResult<BackupItem> {
        for entry in fs::read_dir(&self.backups_dir)? {
            let entry = entry?;
            let server_backup_dir = entry.path();
            
            if server_backup_dir.is_dir() {
                let metadata_path = server_backup_dir.join(format!("{}.json", backup_id));
                if metadata_path.exists() {
                    return self.load_backup_metadata(&metadata_path);
                }
            }
        }
        
        Err(BackupError::NotFound(backup_id.to_string()))
    }
    
    /// 清理旧备份（保留最新的N个）
    pub fn cleanup_old_backups(&self, server_id: &str, max_backups: u32) -> BackupResult<Vec<String>> {
        let backups = self.get_backup_list(server_id)?;
        
        if backups.len() <= max_backups as usize {
            return Ok(Vec::new());
        }
        
        let to_remove_count = backups.len() - max_backups as usize;
        let mut removed = Vec::new();
        
        // 从列表末尾（最旧的）开始删除
        for backup in backups.into_iter().rev().take(to_remove_count) {
            self.delete_backup(&backup.id)?;
            removed.push(backup.id);
        }
        
        info!(
            "清理服务器 {} 的旧备份，删除了 {} 个",
            server_id, removed.len()
        );
        
        Ok(removed)
    }
}