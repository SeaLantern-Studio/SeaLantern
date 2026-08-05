//! 实例管理领域的主错误。

use std::fmt;

use sealantern_infra::fs::FsError;

use sealantern_interface::error::InstanceServiceError;

/// 实例管理操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向 [`InstanceServiceError`]
/// 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum InstanceError {
    /// 指定的实例不存在。
    NotFound,
    /// 目标实例标识已存在（创建冲突）。
    AlreadyExists,
    /// 实例数据校验失败（由 core 判定，含具体违规项）。
    Invalid {
        /// 底层校验错误细节。
        source: sealantern_core::instance::InstanceError,
    },
    /// 实例当前状态不允许该操作（如未运行时停止、已运行时重复启动）。
    InvalidState,
    /// 底层 IO / 供给 / 进程操作失败。
    OperationFailed {
        /// 底层来源错误（如存储/序列化）。
        source: FsError,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for InstanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(formatter, "server instance not found"),
            Self::AlreadyExists => write!(formatter, "server instance already exists"),
            Self::Invalid { source } => write!(formatter, "instance data is invalid: {source}"),
            Self::InvalidState => {
                write!(formatter, "server instance is in an invalid state for this operation")
            }
            Self::OperationFailed { source } => {
                write!(formatter, "server instance operation failed: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for InstanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid { source } => Some(source),
            Self::OperationFailed { source } => Some(source),
            _ => None,
        }
    }
}

impl From<FsError> for InstanceError {
    fn from(source: FsError) -> Self {
        Self::OperationFailed { source }
    }
}

impl From<sealantern_core::instance::InstanceError> for InstanceError {
    fn from(source: sealantern_core::instance::InstanceError) -> Self {
        Self::Invalid { source }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<InstanceError> for InstanceServiceError {
    fn from(error: InstanceError) -> Self {
        match error {
            InstanceError::NotFound => Self::InstanceNotFound,
            InstanceError::AlreadyExists => Self::AlreadyExists,
            InstanceError::Invalid { .. } | InstanceError::InvalidState => Self::InvalidState,
            InstanceError::OperationFailed { .. } => Self::OperationFailed,
            InstanceError::Unsupported => Self::Unsupported,
        }
    }
}
