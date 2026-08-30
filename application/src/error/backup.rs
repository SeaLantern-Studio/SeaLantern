//! 服务器备份领域的主错误。

use std::fmt;

use sealantern_contract::error::BackupServiceError;

/// 服务器备份操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向
/// [`BackupServiceError`] 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum BackupError {
    /// 指定的备份或服务器实例不存在。
    NotFound,
    /// 客户端提供的输入不合法（如无效的备份 ID、服务器 ID）。
    InvalidInput,
    /// 服务器正在运行，无法执行冷备份 / 恢复。
    ServerRunning,
    /// 底层备份文件操作失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 内部异步任务失败（如阻塞任务被取消或 panic）。
    Internal {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for BackupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(formatter, "backup or server instance not found"),
            Self::InvalidInput => write!(formatter, "invalid backup input"),
            Self::ServerRunning => {
                write!(formatter, "server is running; stop it before backup or restore")
            }
            Self::OperationFailed { source } => {
                write!(formatter, "backup operation failed: {source}")
            }
            Self::Internal { source } => write!(formatter, "internal backup task failed: {source}"),
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for BackupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } | Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<sealantern_feature::backup::BackupError> for BackupError {
    fn from(source: sealantern_feature::backup::BackupError) -> Self {
        use sealantern_feature::backup::BackupError as FeatureBackupError;

        match source {
            FeatureBackupError::NotFound(_) => Self::NotFound,
            FeatureBackupError::ServerNotFound(_) => Self::NotFound,
            FeatureBackupError::InvalidBackupId(_) => Self::InvalidInput,
            FeatureBackupError::Validation(_) => Self::InvalidInput,
            FeatureBackupError::ServerRunning(_) => Self::ServerRunning,
            FeatureBackupError::AlreadyExists(_)
            | FeatureBackupError::Io(_)
            | FeatureBackupError::FileSystem(_)
            | FeatureBackupError::Zip(_)
            | FeatureBackupError::Archive(_)
            | FeatureBackupError::Persistence(_)
            | FeatureBackupError::CannotCreateBackupDir(_)
            | FeatureBackupError::CorruptedBackup(_)
            | FeatureBackupError::InsufficientMemory { .. }
            | FeatureBackupError::Json(_) => Self::OperationFailed { source: Box::new(source) },
            FeatureBackupError::TaskFailed(_) => Self::Internal { source: Box::new(source) },
        }
    }
}

impl From<sealantern_contract::InstanceServiceError> for BackupError {
    fn from(source: sealantern_contract::InstanceServiceError) -> Self {
        match source {
            sealantern_contract::InstanceServiceError::InstanceNotFound => Self::NotFound,
            sealantern_contract::InstanceServiceError::InvalidInput => Self::InvalidInput,
            sealantern_contract::InstanceServiceError::InvalidState
            | sealantern_contract::InstanceServiceError::AlreadyExists
            | sealantern_contract::InstanceServiceError::SourceUnavailable
            | sealantern_contract::InstanceServiceError::SourceNotDirectory
            | sealantern_contract::InstanceServiceError::SourceAlreadyImported
            | sealantern_contract::InstanceServiceError::NoLaunchCandidate
            | sealantern_contract::InstanceServiceError::InspectionPanicked
            | sealantern_contract::InstanceServiceError::BuildFailed
            | sealantern_contract::InstanceServiceError::PlanInvalid
            | sealantern_contract::InstanceServiceError::ListFailed
            | sealantern_contract::InstanceServiceError::CreateFailed
            | sealantern_contract::InstanceServiceError::OperationFailed => {
                Self::OperationFailed { source: Box::new(source) }
            }
            sealantern_contract::InstanceServiceError::Unsupported => Self::Unsupported,
        }
    }
}

impl From<sealantern_contract::ServerServiceError> for BackupError {
    fn from(source: sealantern_contract::ServerServiceError) -> Self {
        match source {
            sealantern_contract::ServerServiceError::InstanceNotFound => Self::NotFound,
            sealantern_contract::ServerServiceError::InvalidInput => Self::InvalidInput,
            sealantern_contract::ServerServiceError::InvalidState
            | sealantern_contract::ServerServiceError::OperationFailed => {
                Self::OperationFailed { source: Box::new(source) }
            }
            sealantern_contract::ServerServiceError::Unsupported => Self::Unsupported,
        }
    }
}

impl From<tokio::task::JoinError> for BackupError {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Internal { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<BackupError> for BackupServiceError {
    fn from(error: BackupError) -> Self {
        match error {
            BackupError::NotFound => Self::NotFound,
            BackupError::InvalidInput => Self::InvalidInput,
            BackupError::ServerRunning => Self::ServerRunning,
            BackupError::OperationFailed { .. } | BackupError::Internal { .. } => {
                Self::OperationFailed
            }
            BackupError::Unsupported => Self::Unsupported,
        }
    }
}
