use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use sealantern_infra::fs::write_atomic_blocking;
use sealantern_infra::platform::get_app_data_dir;
use serde::{Deserialize, Serialize};
use tempfile::Builder;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::archive;
use super::error::{BackupError, BackupResult};
use super::models::{
    BackupContentType, BackupFormat, BackupItem, CreateBackupRequest, is_safe_path_component,
};

const MAX_METADATA_BYTES: u64 = 1024 * 1024;
const MANAGED_CONTENT_DIRECTORIES: &[&str] = &["config", "plugins", "world", "logs"];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredBackupMetadata {
    #[serde(flatten)]
    backup: BackupItem,
}

struct BackupRecord {
    metadata: StoredBackupMetadata,
    metadata_path: PathBuf,
    archive_path: PathBuf,
}

/// 备份管理器
pub struct BackupManager {
    backups_dir: PathBuf,
}

impl BackupManager {
    /// 创建新的备份管理器
    pub fn new() -> BackupResult<Self> {
        Self::from_backups_dir(get_app_data_dir().join("backups"))
    }

    fn from_backups_dir(backups_dir: PathBuf) -> BackupResult<Self> {
        fs::create_dir_all(&backups_dir)
            .map_err(|_| BackupError::CannotCreateBackupDir(backups_dir.clone()))?;
        let metadata = fs::symlink_metadata(&backups_dir)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(BackupError::Validation(format!(
                "备份路径不是普通目录: {:?}",
                backups_dir
            )));
        }

        debug!("备份管理器初始化完成，备份目录: {:?}", backups_dir);
        Ok(Self { backups_dir })
    }

    #[cfg(test)]
    pub(crate) fn new_at(backups_dir: PathBuf) -> BackupResult<Self> {
        Self::from_backups_dir(backups_dir)
    }

    fn get_server_backup_dir(&self, server_id: &str) -> BackupResult<PathBuf> {
        validate_server_id(server_id)?;
        Ok(self.backups_dir.join(server_id))
    }

    fn get_backup_metadata_path(&self, server_id: &str, backup_id: &str) -> BackupResult<PathBuf> {
        validate_server_id(server_id)?;
        validate_backup_id(backup_id)?;
        Ok(self
            .backups_dir
            .join(server_id)
            .join(format!("{backup_id}.json")))
    }

    fn get_backup_file_path(
        &self,
        server_id: &str,
        backup_id: &str,
        format: BackupFormat,
    ) -> BackupResult<PathBuf> {
        validate_server_id(server_id)?;
        validate_backup_id(backup_id)?;
        Ok(self
            .backups_dir
            .join(server_id)
            .join(format!("{backup_id}.{}", format.extension())))
    }

    fn generate_backup_id() -> String {
        format!("backup-{}", Uuid::new_v4())
    }

    fn generate_backup_name(server_id: &str, created_at: &DateTime<Utc>) -> String {
        format!("{}_{}.backup", server_id, created_at.format("%Y%m%d_%H%M%S"))
    }

    /// 获取服务器的备份列表。
    pub fn get_backup_list(&self, server_id: &str) -> BackupResult<Vec<BackupItem>> {
        let server_backup_dir = self.get_server_backup_dir(server_id)?;
        match fs::symlink_metadata(&server_backup_dir) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(BackupError::Validation(format!(
                        "服务器备份路径不是普通目录: {:?}",
                        server_backup_dir
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                debug!("服务器备份目录不存在: {:?}", server_backup_dir);
                return Ok(Vec::new());
            }
            Err(error) => return Err(BackupError::Io(error)),
        }

        let mut backups = Vec::new();
        for entry in fs::read_dir(&server_backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || path.extension() != Some(OsStr::new("json"))
            {
                continue;
            }

            let Some(backup_id) = path.file_stem().and_then(OsStr::to_str) else {
                warn!("忽略包含非UTF-8名称的备份元数据: {:?}", path);
                continue;
            };

            match self.load_backup_metadata(&path).and_then(|metadata| {
                validate_stored_metadata(&metadata, server_id, backup_id)?;
                Ok(metadata.backup)
            }) {
                Ok(backup) => backups.push(backup),
                Err(error) => {
                    warn!("无法加载备份元数据 {:?}: {}", path, error);
                }
            }
        }

        backups.sort_by(|left, right| {
            let left_time = DateTime::parse_from_rfc3339(&left.created_at)
                .ok()
                .map(|value| value.with_timezone(&Utc));
            let right_time = DateTime::parse_from_rfc3339(&right.created_at)
                .ok()
                .map(|value| value.with_timezone(&Utc));
            right_time
                .cmp(&left_time)
                .then_with(|| right.id.cmp(&left.id))
        });

        info!("获取服务器 {} 的备份列表，共 {} 个备份", server_id, backups.len());
        Ok(backups)
    }

    fn load_backup_metadata(&self, path: &Path) -> BackupResult<StoredBackupMetadata> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(BackupError::CorruptedBackup(path.to_path_buf()));
        }
        if metadata.len() > MAX_METADATA_BYTES {
            return Err(BackupError::CorruptedBackup(path.to_path_buf()));
        }

        let mut content = String::new();
        File::open(path)?
            .take(MAX_METADATA_BYTES + 1)
            .read_to_string(&mut content)?;
        if content.len() as u64 > MAX_METADATA_BYTES {
            return Err(BackupError::CorruptedBackup(path.to_path_buf()));
        }
        Ok(serde_json::from_str(&content)?)
    }

    fn save_backup_metadata(&self, metadata: &StoredBackupMetadata) -> BackupResult<()> {
        let path =
            self.get_backup_metadata_path(&metadata.backup.server_id, &metadata.backup.id)?;
        let content = serde_json::to_vec_pretty(metadata)?;
        write_atomic_blocking(&path, &content)?;
        debug!("保存备份元数据: {:?}", path);
        Ok(())
    }

    /// 创建备份（冷备份）。
    pub fn create_backup(
        &self,
        request: CreateBackupRequest,
        server_dir: &Path,
        check_server_stopped: impl Fn(&str) -> bool,
    ) -> BackupResult<BackupItem> {
        validate_server_id(&request.server_id)?;
        let contents = normalize_contents(&request.contents)?;

        if !check_server_stopped(&request.server_id) {
            error!("服务器 {} 正在运行，无法执行冷备份", request.server_id);
            return Err(BackupError::ServerRunning(request.server_id));
        }

        let canonical_server_dir = canonical_server_dir(&request.server_id, server_dir)?;
        let server_backup_dir = self.ensure_server_backup_dir(&request.server_id)?;

        let backup_id = Self::generate_backup_id();
        let created_at = Utc::now();
        let name = request
            .name
            .unwrap_or_else(|| Self::generate_backup_name(&request.server_id, &created_at));
        let backup_file =
            self.get_backup_file_path(&request.server_id, &backup_id, request.format)?;

        info!(
            "开始创建备份: ID={}, 服务器={}, 内容={:?}",
            backup_id, request.server_id, contents
        );

        let temporary = Builder::new()
            .prefix(".backup-create-")
            .tempdir_in(&server_backup_dir)?;
        let included_contents =
            self.prepare_backup_content(&canonical_server_dir, temporary.path(), &contents)?;
        if included_contents.is_empty() {
            return Err(BackupError::Validation(
                "请求的备份内容在服务器目录中均不存在".to_string(),
            ));
        }

        archive::create_archive(
            temporary.path(),
            &backup_file,
            request.format,
            request.compression_level,
        )?;
        drop(temporary);

        let size = match fs::metadata(&backup_file) {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                remove_file_with_warning(&backup_file);
                return Err(BackupError::Io(error));
            }
        };
        let metadata = StoredBackupMetadata {
            backup: BackupItem {
                id: backup_id,
                server_id: request.server_id,
                name,
                format: request.format,
                size,
                created_at: created_at.to_rfc3339(),
                contents: included_contents,
            },
        };

        if let Err(error) = self.save_backup_metadata(&metadata) {
            remove_file_with_warning(&backup_file);
            return Err(error);
        }

        info!("备份创建成功: ID={}, 大小={}字节", metadata.backup.id, metadata.backup.size);
        Ok(metadata.backup)
    }

    fn prepare_backup_content(
        &self,
        server_dir: &Path,
        temporary: &Path,
        contents: &[BackupContentType],
    ) -> BackupResult<Vec<BackupContentType>> {
        fs::create_dir_all(temporary)?;
        let mut included = Vec::new();

        for content_type in contents {
            if *content_type == BackupContentType::Core {
                self.copy_core_content(server_dir, temporary)?;
                included.push(*content_type);
                continue;
            }

            let source = server_dir.join(content_type.directory_name());
            match fs::symlink_metadata(&source) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(BackupError::Validation(format!(
                        "备份内容目录不能是符号链接: {:?}",
                        source
                    )));
                }
                Ok(metadata) if metadata.is_dir() => {
                    let destination = temporary.join(content_type.directory_name());
                    self.copy_dir_all(&source, &destination)?;
                    included.push(*content_type);
                }
                Ok(_) => {
                    return Err(BackupError::Validation(format!(
                        "备份内容路径不是目录: {:?}",
                        source
                    )));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    debug!("跳过不存在的备份内容: {:?}", source);
                }
                Err(error) => return Err(BackupError::Io(error)),
            }
        }

        Ok(included)
    }

    fn ensure_server_backup_dir(&self, server_id: &str) -> BackupResult<PathBuf> {
        let path = self.get_server_backup_dir(server_id)?;
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                Err(BackupError::Validation(format!("服务器备份路径不是普通目录: {:?}", path)))
            }
            Ok(_) => Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(&path)?;
                let metadata = fs::symlink_metadata(&path)?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(BackupError::Validation(format!(
                        "服务器备份路径不是普通目录: {:?}",
                        path
                    )));
                }
                Ok(path)
            }
            Err(error) => Err(BackupError::Io(error)),
        }
    }

    fn copy_core_content(&self, source: &Path, destination: &Path) -> BackupResult<()> {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let file_name = entry.file_name();
            if is_managed_content_directory(&file_name) {
                continue;
            }
            self.copy_entry(&entry.path(), &destination.join(file_name))?;
        }
        Ok(())
    }

    fn copy_entry(&self, source: &Path, destination: &Path) -> BackupResult<()> {
        let metadata = fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() {
            return Err(BackupError::Validation(format!("备份内容不能包含符号链接: {:?}", source)));
        }
        if metadata.is_dir() {
            self.copy_dir_all(source, destination)
        } else if metadata.is_file() {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, destination)?;
            Ok(())
        } else {
            Err(BackupError::Validation(format!("备份内容包含不支持的特殊文件: {:?}", source)))
        }
    }

    fn copy_dir_all(&self, source: &Path, destination: &Path) -> BackupResult<()> {
        match fs::symlink_metadata(destination) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(BackupError::Validation(format!(
                    "复制目标不是普通目录: {:?}",
                    destination
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(destination)?;
            }
            Err(error) => return Err(BackupError::Io(error)),
        }

        let source_metadata = fs::symlink_metadata(source)?;
        if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
            return Err(BackupError::Validation(format!("复制源不是普通目录: {:?}", source)));
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            self.copy_entry(&entry.path(), &destination.join(entry.file_name()))?;
        }
        Ok(())
    }

    /// 删除备份。
    pub fn delete_backup(&self, backup_id: &str) -> BackupResult<()> {
        let record = self.find_backup_record(backup_id)?;
        remove_archive_if_present(&record.archive_path)?;
        fs::remove_file(&record.metadata_path)?;
        info!("备份删除成功: {}", backup_id);
        Ok(())
    }

    /// 恢复备份（冷恢复）。
    pub fn restore_backup(
        &self,
        backup_id: &str,
        server_id: &str,
        server_dir: &Path,
        check_server_stopped: impl Fn(&str) -> bool,
    ) -> BackupResult<()> {
        let record = self.find_backup_record(backup_id)?;
        let target_dir = self.validate_restore_target(&record, server_id, server_dir)?;

        if !check_server_stopped(&record.metadata.backup.server_id) {
            error!("服务器 {} 正在运行，无法执行恢复", record.metadata.backup.server_id);
            return Err(BackupError::ServerRunning(record.metadata.backup.server_id.clone()));
        }
        ensure_archive_file(&record.archive_path)?;

        info!(
            "开始恢复备份: ID={}, 服务器={}, 格式={}",
            backup_id, record.metadata.backup.server_id, record.metadata.backup.format
        );

        let extract_base = tempfile::tempdir()?;
        let extract_dir = extract_base.path().join("extracted");
        archive::extract_archive(
            &record.archive_path,
            &extract_dir,
            record.metadata.backup.format,
        )?;

        let parent = target_dir.parent().ok_or_else(|| {
            BackupError::Validation(format!("目标路径无父目录: {:?}", target_dir))
        })?;
        fs::create_dir_all(parent)?;
        let staging_base = Builder::new()
            .prefix(".backup-restore-")
            .tempdir_in(parent)?;
        let staged_dir = staging_base.path().join("server");
        fs::create_dir(&staged_dir)?;
        self.copy_dir_all(&target_dir, &staged_dir)?;
        self.apply_restored_content(
            &extract_dir,
            &staged_dir,
            &record.metadata.backup.contents,
            &record.archive_path,
        )?;
        publish_staged_directory(&staged_dir, &target_dir)?;

        info!("备份恢复成功: {}", backup_id);
        Ok(())
    }

    fn validate_restore_target(
        &self,
        record: &BackupRecord,
        server_id: &str,
        server_dir: &Path,
    ) -> BackupResult<PathBuf> {
        validate_server_id(server_id)?;
        if record.metadata.backup.server_id != server_id {
            return Err(BackupError::Validation(format!(
                "备份属于服务器 {}，不能恢复到服务器 {}",
                record.metadata.backup.server_id, server_id
            )));
        }
        canonical_server_dir(server_id, server_dir)
    }

    fn apply_restored_content(
        &self,
        extracted: &Path,
        staged: &Path,
        contents: &[BackupContentType],
        archive_path: &Path,
    ) -> BackupResult<()> {
        for content_type in contents {
            let source = extracted.join(content_type.directory_name());
            if !is_normal_directory(&source)? {
                return Err(BackupError::CorruptedBackup(archive_path.to_path_buf()));
            }

            if *content_type == BackupContentType::Core {
                self.replace_core_content(&source, staged)?;
                continue;
            }

            let destination = staged.join(content_type.directory_name());
            remove_path_if_present(&destination)?;
            self.copy_dir_all(&source, &destination)?;
        }
        Ok(())
    }

    fn replace_core_content(&self, source: &Path, destination: &Path) -> BackupResult<()> {
        for entry in fs::read_dir(destination)? {
            let entry = entry?;
            if is_managed_content_directory(&entry.file_name()) {
                continue;
            }
            remove_path_if_present(&entry.path())?;
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            if is_managed_content_directory(&entry.file_name()) {
                continue;
            }
            let destination_entry = destination.join(entry.file_name());
            remove_path_if_present(&destination_entry)?;
            self.copy_entry(&entry.path(), &destination_entry)?;
        }
        Ok(())
    }

    fn find_backup_record(&self, backup_id: &str) -> BackupResult<BackupRecord> {
        validate_backup_id(backup_id)?;
        for entry in fs::read_dir(&self.backups_dir)? {
            let entry = entry?;
            let server_backup_dir = entry.path();
            let metadata = fs::symlink_metadata(&server_backup_dir)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                continue;
            }
            let Some(server_id) = server_backup_dir.file_name().and_then(OsStr::to_str) else {
                continue;
            };
            if !is_safe_path_component(server_id) {
                continue;
            }

            let metadata_path = server_backup_dir.join(format!("{backup_id}.json"));
            match fs::symlink_metadata(&metadata_path) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                    return Err(BackupError::CorruptedBackup(metadata_path));
                }
                Ok(_) => {
                    let metadata = self.load_backup_metadata(&metadata_path)?;
                    validate_stored_metadata(&metadata, server_id, backup_id)?;
                    let archive_path = server_backup_dir
                        .join(format!("{backup_id}.{}", metadata.backup.format.extension()));
                    return Ok(BackupRecord { metadata, metadata_path, archive_path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(BackupError::Io(error)),
            }
        }
        Err(BackupError::NotFound(backup_id.to_string()))
    }

    /// 清理旧备份（保留最新的N个）。
    pub fn cleanup_old_backups(
        &self,
        server_id: &str,
        max_backups: u32,
    ) -> BackupResult<Vec<String>> {
        let backups = self.get_backup_list(server_id)?;
        if backups.len() <= max_backups as usize {
            return Ok(Vec::new());
        }

        let to_remove_count = backups.len() - max_backups as usize;
        let mut removed = Vec::new();
        for backup in backups.into_iter().rev().take(to_remove_count) {
            self.delete_backup(&backup.id)?;
            removed.push(backup.id);
        }

        info!("清理服务器 {} 的旧备份，删除了 {} 个备份", server_id, removed.len());
        Ok(removed)
    }
}

fn validate_stored_metadata(
    metadata: &StoredBackupMetadata,
    expected_server_id: &str,
    expected_backup_id: &str,
) -> BackupResult<()> {
    validate_server_id(&metadata.backup.server_id)?;
    validate_backup_id(&metadata.backup.id)?;
    if metadata.backup.server_id != expected_server_id {
        return Err(BackupError::Validation(format!(
            "备份元数据服务器ID与目录不一致: {}",
            metadata.backup.id
        )));
    }
    if metadata.backup.id != expected_backup_id {
        return Err(BackupError::Validation(format!(
            "备份元数据ID与文件名不一致: {}",
            metadata.backup.id
        )));
    }
    if metadata.backup.contents.is_empty() {
        return Err(BackupError::Validation(format!("备份元数据没有内容: {}", metadata.backup.id)));
    }
    for (index, content) in metadata.backup.contents.iter().enumerate() {
        if metadata.backup.contents[..index].contains(content) {
            return Err(BackupError::Validation(format!(
                "备份元数据包含重复内容: {}",
                metadata.backup.id
            )));
        }
    }
    DateTime::parse_from_rfc3339(&metadata.backup.created_at).map_err(|error| {
        BackupError::Validation(format!("备份元数据创建时间无效 {}: {}", metadata.backup.id, error))
    })?;
    Ok(())
}

fn normalize_contents(contents: &[BackupContentType]) -> BackupResult<Vec<BackupContentType>> {
    if contents.is_empty() {
        return Err(BackupError::Validation("备份内容不能为空".to_string()));
    }

    let mut normalized = Vec::with_capacity(contents.len());
    if contents.contains(&BackupContentType::Core) {
        normalized.push(BackupContentType::Core);
    }
    for content in contents {
        if *content != BackupContentType::Core && !normalized.contains(content) {
            normalized.push(*content);
        }
    }
    Ok(normalized)
}

fn canonical_server_dir(server_id: &str, path: &Path) -> BackupResult<PathBuf> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(BackupError::ServerNotFound(server_id.to_string()));
        }
        Err(error) => return Err(BackupError::Io(error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BackupError::Validation(format!("服务器路径不是普通目录: {:?}", path)));
    }
    Ok(fs::canonicalize(path)?)
}

fn is_normal_directory(path: &Path) -> BackupResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(!metadata.file_type().is_symlink() && metadata.is_dir()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(BackupError::Io(error)),
    }
}

fn is_managed_content_directory(name: &OsStr) -> bool {
    MANAGED_CONTENT_DIRECTORIES
        .iter()
        .any(|managed| name == OsStr::new(managed))
}

fn remove_path_if_present(path: &Path) -> BackupResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
            fs::remove_file(path)?;
        }
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path)?;
        }
        Ok(_) => {
            return Err(BackupError::Validation(format!("无法删除不支持的目标类型: {:?}", path)));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(BackupError::Io(error)),
    }
    Ok(())
}

fn publish_staged_directory(staged: &Path, destination: &Path) -> BackupResult<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| BackupError::Validation(format!("目标路径无父目录: {:?}", destination)))?;
    let name = destination
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| BackupError::Validation(format!("目标路径无有效名称: {:?}", destination)))?;
    let rollback = parent.join(format!(".{name}.backup-rollback-{}", Uuid::new_v4()));

    fs::rename(destination, &rollback)?;
    match fs::rename(staged, destination) {
        Ok(()) => {
            if let Err(error) = fs::remove_dir_all(&rollback) {
                warn!("恢复已完成，但清理旧服务器目录失败 {:?}: {}", rollback, error);
            }
            Ok(())
        }
        Err(error) => {
            if let Err(rollback_error) = fs::rename(&rollback, destination) {
                return Err(BackupError::Validation(format!(
                    "恢复发布失败，且无法回滚服务器目录: {}; 回滚错误: {}",
                    error, rollback_error
                )));
            }
            Err(BackupError::Io(error))
        }
    }
}

fn ensure_archive_file(path: &Path) -> BackupResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(BackupError::CorruptedBackup(path.to_path_buf()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(BackupError::CorruptedBackup(path.to_path_buf()))
        }
        Err(error) => Err(BackupError::Io(error)),
    }
}

fn remove_archive_if_present(path: &Path) -> BackupResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(BackupError::CorruptedBackup(path.to_path_buf()))
        }
        Ok(_) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(BackupError::Io(error)),
    }
}

fn remove_file_with_warning(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        warn!("清理失败的备份文件失败 {:?}: {}", path, error);
    }
}

fn validate_server_id(server_id: &str) -> BackupResult<()> {
    if is_safe_path_component(server_id) {
        Ok(())
    } else {
        Err(BackupError::Validation(format!("无效的服务器ID: {}", server_id)))
    }
}

fn validate_backup_id(backup_id: &str) -> BackupResult<()> {
    if is_safe_path_component(backup_id) {
        Ok(())
    } else {
        Err(BackupError::InvalidBackupId(backup_id.to_string()))
    }
}
