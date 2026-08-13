//! 服务器控制台日志领域的主错误。

use std::fmt;

use sealantern_interface::error::ConsoleServiceError;

/// 服务器控制台日志操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向
/// [`ConsoleServiceError`] 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum ConsoleError {
    /// 指定的实例不存在（无法定位日志目录）。
    InstanceNotFound,
    /// 客户端提供的输入不合法（如游标为负）。
    InvalidInput,
    /// 底层日志数据库操作失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for ConsoleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InstanceNotFound => write!(formatter, "server instance not found"),
            Self::InvalidInput => write!(formatter, "invalid console log input"),
            Self::OperationFailed { source } => {
                write!(formatter, "console log operation failed: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for ConsoleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<sealantern_infra::persistence::PersistenceError> for ConsoleError {
    fn from(source: sealantern_infra::persistence::PersistenceError) -> Self {
        Self::OperationFailed { source: Box::new(source) }
    }
}

impl From<sealantern_interface::InstanceServiceError> for ConsoleError {
    fn from(source: sealantern_interface::InstanceServiceError) -> Self {
        match source {
            sealantern_interface::InstanceServiceError::InstanceNotFound => Self::InstanceNotFound,
            sealantern_interface::InstanceServiceError::InvalidInput => Self::InvalidInput,
            sealantern_interface::InstanceServiceError::InvalidState
            | sealantern_interface::InstanceServiceError::AlreadyExists
            | sealantern_interface::InstanceServiceError::OperationFailed => {
                Self::OperationFailed { source: Box::new(source) }
            }
            sealantern_interface::InstanceServiceError::Unsupported => Self::Unsupported,
        }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<ConsoleError> for ConsoleServiceError {
    fn from(error: ConsoleError) -> Self {
        match error {
            ConsoleError::InstanceNotFound => Self::InstanceNotFound,
            ConsoleError::InvalidInput => Self::InvalidInput,
            ConsoleError::OperationFailed { .. } => Self::OperationFailed,
            ConsoleError::Unsupported => Self::Unsupported,
        }
    }
}
