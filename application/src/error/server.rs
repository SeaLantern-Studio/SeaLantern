//! 服务器进程管理领域的主错误。

use std::fmt;

use sealantern_interface::error::ServerServiceError;

/// 服务器进程管理操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向 [`ServerServiceError`]
/// 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum ServerError {
    /// 指定的实例不存在。
    InstanceNotFound,
    /// 服务器进程当前状态不允许该操作。
    InvalidState,
    /// 客户端提供的输入不合法。
    InvalidInput,
    /// 底层进程 / IO 操作失败。
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

impl fmt::Display for ServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InstanceNotFound => write!(formatter, "server instance not found"),
            Self::InvalidState => {
                write!(formatter, "server is in an invalid state for this operation")
            }
            Self::InvalidInput => write!(formatter, "invalid input"),
            Self::OperationFailed { source } => {
                write!(formatter, "server operation failed: {source}")
            }
            Self::Internal { source } => write!(formatter, "internal server task failed: {source}"),
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for ServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } => Some(source.as_ref()),
            Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(source: std::io::Error) -> Self {
        Self::OperationFailed { source: Box::new(source) }
    }
}

impl From<tokio::task::JoinError> for ServerError {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Internal { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<ServerError> for ServerServiceError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::InstanceNotFound => Self::InstanceNotFound,
            ServerError::InvalidState => Self::InvalidState,
            ServerError::InvalidInput => Self::InvalidInput,
            ServerError::OperationFailed { .. } | ServerError::Internal { .. } => {
                Self::OperationFailed
            }
            ServerError::Unsupported => Self::Unsupported,
        }
    }
}
