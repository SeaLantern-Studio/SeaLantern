//! 系统资源信息领域的主错误。

use std::fmt;

use sealantern_interface::error::SystemServiceError;

/// 系统资源信息操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向 [`SystemServiceError`]
/// 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum SystemError {
    /// 指定的进程不存在或无权访问。
    ProcessNotFound,
    /// 指定的路径不存在或不可访问。
    PathNotFound,
    /// 底层系统采集 / IO 操作失败。
    OperationFailed {
        /// 底层来源错误。
        source: std::io::Error,
    },
    /// 内部异步任务失败（如阻塞任务被取消或 panic）。
    Internal {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 无法确定默认运行路径（标准数据目录、文档目录与当前目录均不可用）。
    DefaultRunPathUnresolved {
        /// 底层来源错误。
        source: std::io::Error,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for SystemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessNotFound => write!(formatter, "process not found"),
            Self::PathNotFound => write!(formatter, "path not found"),
            Self::OperationFailed { source } => {
                write!(formatter, "system operation failed: {source}")
            }
            Self::Internal { source } => write!(formatter, "internal system task failed: {source}"),
            Self::DefaultRunPathUnresolved { source } => {
                write!(formatter, "default run path unresolved: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for SystemError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } => Some(source),
            Self::Internal { source } => Some(source.as_ref()),
            Self::DefaultRunPathUnresolved { source } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SystemError {
    fn from(source: std::io::Error) -> Self {
        Self::OperationFailed { source }
    }
}

impl From<tokio::task::JoinError> for SystemError {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Internal { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<SystemError> for SystemServiceError {
    fn from(error: SystemError) -> Self {
        match error {
            SystemError::ProcessNotFound => Self::ProcessNotFound,
            SystemError::PathNotFound => Self::PathNotFound,
            SystemError::OperationFailed { .. } | SystemError::Internal { .. } => {
                Self::OperationFailed
            }
            SystemError::DefaultRunPathUnresolved { .. } => Self::DefaultRunPathUnresolved,
            SystemError::Unsupported => Self::Unsupported,
        }
    }
}
