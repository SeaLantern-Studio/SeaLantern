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
    /// 客户端提供的导入输入不合法（如无效启动模式）。
    InvalidInput,
    /// 导入源目录不可用（不存在或不可读）。
    ImportSourceUnavailable,
    /// 导入源目录已是受管实例（重复导入）。
    ImportSourceAlreadyImported,
    /// 导入源目录中未找到可启动的服务端文件。
    ImportNoLaunchCandidate,
    /// 导入操作底层执行失败（检查、解压、复制等）。
    ImportFailed,
    /// 底层 IO / 供给 / 进程操作失败。
    OperationFailed {
        /// 底层来源错误（如存储/序列化）。
        source: FsError,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
    /// 内部错误（如设置管理器加载失败）。
    Internal(String),
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
            Self::InvalidInput => write!(formatter, "invalid server import input"),
            Self::ImportSourceUnavailable => {
                write!(formatter, "import source directory is unavailable")
            }
            Self::ImportSourceAlreadyImported => {
                write!(formatter, "source directory is already imported as a server instance")
            }
            Self::ImportNoLaunchCandidate => {
                write!(formatter, "no launchable server artifact was found in the source directory")
            }
            Self::ImportFailed => write!(formatter, "server import operation failed"),
            Self::OperationFailed { source } => {
                write!(formatter, "server instance operation failed: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
            Self::Internal(msg) => write!(formatter, "internal error: {msg}"),
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
            InstanceError::Invalid { .. } => Self::InvalidInput,
            InstanceError::InvalidState => Self::InvalidState,
            InstanceError::InvalidInput => Self::InvalidInput,
            InstanceError::ImportSourceUnavailable => Self::SourceUnavailable,
            InstanceError::ImportSourceAlreadyImported => Self::SourceAlreadyImported,
            InstanceError::ImportNoLaunchCandidate => Self::NoLaunchCandidate,
            InstanceError::ImportFailed => Self::OperationFailed,
            InstanceError::OperationFailed { .. } => Self::OperationFailed,
            InstanceError::Unsupported => Self::Unsupported,
            InstanceError::Internal(_) => Self::OperationFailed,
        }
    }
}
