//! 服务器定时任务领域的主错误。

use std::fmt;

use sealantern_extra::server::cron_task::CronTaskError as ExtraCronTaskError;
use sealantern_interface::CronTaskServiceError;

/// 服务器定时任务操作失败的应用层主错误。
#[derive(Debug)]
pub enum CronTaskError {
    /// 指定任务不存在。
    TaskNotFound { source: ExtraCronTaskError },
    /// 任务输入或 Cron 表达式不合法。
    InvalidInput { source: ExtraCronTaskError },
    /// JSON 持久化失败。
    StorageFailed { source: ExtraCronTaskError },
    /// 服务器动作执行失败。
    ExecutionFailed { source: ExtraCronTaskError },
    /// 该能力尚未实现。
    Unsupported,
}

impl fmt::Display for CronTaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskNotFound { source } => write!(formatter, "cron task not found: {source}"),
            Self::InvalidInput { source } => write!(formatter, "invalid cron task: {source}"),
            Self::StorageFailed { source } => {
                write!(formatter, "cron task storage failed: {source}")
            }
            Self::ExecutionFailed { source } => {
                write!(formatter, "cron task execution failed: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for CronTaskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TaskNotFound { source }
            | Self::InvalidInput { source }
            | Self::StorageFailed { source }
            | Self::ExecutionFailed { source } => Some(source),
            Self::Unsupported => None,
        }
    }
}

impl From<ExtraCronTaskError> for CronTaskError {
    fn from(source: ExtraCronTaskError) -> Self {
        match source {
            ExtraCronTaskError::TaskNotFound(_) => Self::TaskNotFound { source },
            ExtraCronTaskError::InvalidTask(_) | ExtraCronTaskError::InvalidCron { .. } => {
                Self::InvalidInput { source }
            }
            ExtraCronTaskError::Storage(_) => Self::StorageFailed { source },
            ExtraCronTaskError::Execution { .. } => Self::ExecutionFailed { source },
        }
    }
}

impl From<CronTaskError> for CronTaskServiceError {
    fn from(error: CronTaskError) -> Self {
        match error {
            CronTaskError::TaskNotFound { .. } => Self::TaskNotFound,
            CronTaskError::InvalidInput { .. } => Self::InvalidInput,
            CronTaskError::StorageFailed { .. } => Self::StorageFailed,
            CronTaskError::ExecutionFailed { .. } => Self::ExecutionFailed,
            CronTaskError::Unsupported => Self::Unsupported,
        }
    }
}
