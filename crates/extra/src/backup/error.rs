use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("备份文件已存在: {0}")]
    AlreadyExists(PathBuf),

    #[error("备份不存在: {0}")]
    NotFound(String),

    #[error("服务器目录不存在: {0}")]
    ServerNotFound(String),

    #[error("服务器正在运行，无法执行冷备份: {0}")]
    ServerRunning(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("压缩错误: {0}")]
    Archive(#[from] sealantern_infra::archive::ArchiveError),

    #[error("持久化错误: {0}")]
    Persistence(#[from] sealantern_infra::persistence::PersistenceError),

    #[error("无效的备份ID: {0}")]
    InvalidBackupId(String),

    #[error("无法创建备份目录: {0}")]
    CannotCreateBackupDir(PathBuf),

    #[error("备份文件损坏: {0}")]
    CorruptedBackup(PathBuf),

    #[error("序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("设置验证失败: {0}")]
    Validation(String),
}

pub type BackupResult<T> = Result<T, BackupError>;