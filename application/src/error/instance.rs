//! 实例管理领域的主错误。

use std::fmt;
use std::path::PathBuf;

use sealantern_infra::fs::FsError;

use sealantern_core::provisioning::{
    ExistingInstanceError, ImportExistingServerError as CoreImportError, SourceDirectoryError,
};
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
    /// 客户端提供的输入不合法（如无效启动模式）。
    InvalidInput,
    /// 导入源目录不存在或不可访问。
    SourceUnavailable(PathBuf),
    /// 给定导入路径存在但不是目录。
    SourceNotDirectory(PathBuf),
    /// 导入源目录已是受管实例（重复导入）。
    SourceAlreadyImported,
    /// 阻塞的导入检查 / 构建任务被取消或 panic。
    InspectionPanicked,
    /// 检查服务器目录并构建导入规格失败。
    ImportBuild {
        /// 底层检查 / 构建错误。
        source: CoreImportError,
    },
    /// 导入供给计划校验失败（启动目标 / 实例规格非法）。
    ImportPlanInvalid {
        /// 底层供给计划错误。
        source: ExistingInstanceError,
    },
    /// 导入前查询实例列表失败。
    ImportListFailed {
        /// 底层列表查询错误。
        source: InstanceServiceError,
    },
    /// 导入实例创建持久化失败。
    ImportCreateFailed {
        /// 底层创建持久化错误。
        source: InstanceServiceError,
    },
    /// 导入操作底层执行失败（解压、复制等）。
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
            Self::SourceUnavailable(path) => {
                write!(formatter, "import source directory is unavailable: {}", path.display())
            }
            Self::SourceNotDirectory(path) => {
                write!(formatter, "the selected path is not a directory: {}", path.display())
            }
            Self::SourceAlreadyImported => {
                write!(formatter, "source directory is already imported as a server instance")
            }
            Self::InspectionPanicked => {
                write!(formatter, "import spec build task panicked or was cancelled")
            }
            Self::ImportBuild { source } => {
                write!(formatter, "failed to build import spec: {source}")
            }
            Self::ImportPlanInvalid { source } => {
                write!(formatter, "imported instance plan is invalid: {source}")
            }
            Self::ImportListFailed { source } => {
                write!(formatter, "failed to list instances: {source}")
            }
            Self::ImportCreateFailed { source } => {
                write!(formatter, "failed to create imported instance: {source}")
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
            Self::ImportBuild { source } => Some(source),
            Self::ImportPlanInvalid { source } => Some(source),
            Self::ImportListFailed { source } => Some(source),
            Self::ImportCreateFailed { source } => Some(source),
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

impl From<SourceDirectoryError> for InstanceError {
    fn from(error: SourceDirectoryError) -> Self {
        match error {
            SourceDirectoryError::Unavailable(path) => Self::SourceUnavailable(path),
            SourceDirectoryError::NotDirectory(path) => Self::SourceNotDirectory(path),
        }
    }
}

impl From<CoreImportError> for InstanceError {
    fn from(source: CoreImportError) -> Self {
        Self::ImportBuild { source }
    }
}

impl From<ExistingInstanceError> for InstanceError {
    fn from(source: ExistingInstanceError) -> Self {
        Self::ImportPlanInvalid { source }
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
            InstanceError::SourceUnavailable(_) => Self::SourceUnavailable,
            InstanceError::SourceNotDirectory(_) => Self::SourceNotDirectory,
            InstanceError::SourceAlreadyImported => Self::SourceAlreadyImported,
            InstanceError::InspectionPanicked => Self::InspectionPanicked,
            InstanceError::ImportBuild { .. } => Self::BuildFailed,
            InstanceError::ImportPlanInvalid { .. } => Self::PlanInvalid,
            InstanceError::ImportListFailed { .. } => Self::ListFailed,
            InstanceError::ImportCreateFailed { .. } => Self::CreateFailed,
            InstanceError::ImportFailed => Self::OperationFailed,
            InstanceError::OperationFailed { .. } => Self::OperationFailed,
            InstanceError::Unsupported => Self::Unsupported,
            InstanceError::Internal(_) => Self::OperationFailed,
        }
    }
}
