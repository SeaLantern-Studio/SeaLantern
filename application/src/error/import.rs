//! 导入已有服务器操作的领域错误。

use std::fmt;
use std::path::PathBuf;

use sealantern_core::provisioning::{
    ImportExistingServerError as CoreImportError, SourceDirectoryError,
};
use sealantern_interface::InstanceServiceError;

/// 导入已有服务器目录失败的应用层主错误。
///
/// 由 [`crate::service::CoreInstanceService::import_existing_server`] 返回，
/// 聚合源目录校验、去重、检查构建、供给计划与实例创建各阶段的失败，供 Tauri /
/// Axum 命令层映射为各自面向前端的错误码。
#[derive(Debug)]
pub enum ImportExistingServerError {
    /// 源目录不存在或不可访问。
    SourceUnavailable(PathBuf),
    /// 给定路径存在但不是目录。
    SourceNotDirectory(PathBuf),
    /// 该目录已被导入为实例（去重拦截）。
    AlreadyImported,
    /// 阻塞的检查 / 构建任务被取消或 panic。
    InspectionPanicked,
    /// 检查服务器目录并构建导入规格失败。
    Build(CoreImportError),
    /// 供给计划校验失败（启动目标 / 实例规格非法）。
    PlanInvalid,
    /// 实例列表查询失败。
    ListFailed(InstanceServiceError),
    /// 导入实例创建持久化失败。
    CreateFailed(InstanceServiceError),
}

impl fmt::Display for ImportExistingServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceUnavailable(path) => {
                write!(formatter, "the selected directory does not exist: {}", path.display())
            }
            Self::SourceNotDirectory(path) => {
                write!(formatter, "the selected path is not a directory: {}", path.display())
            }
            Self::AlreadyImported => {
                write!(formatter, "the selected directory is already imported as a server instance")
            }
            Self::InspectionPanicked => {
                write!(formatter, "import spec build task panicked or was cancelled")
            }
            Self::Build(error) => write!(formatter, "failed to build import spec: {error}"),
            Self::PlanInvalid => write!(formatter, "imported instance spec is invalid"),
            Self::ListFailed(error) => write!(formatter, "failed to list instances: {error}"),
            Self::CreateFailed(error) => {
                write!(formatter, "failed to create imported instance: {error}")
            }
        }
    }
}

impl std::error::Error for ImportExistingServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Build(error) => Some(error),
            Self::ListFailed(error) => Some(error),
            Self::CreateFailed(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SourceDirectoryError> for ImportExistingServerError {
    fn from(error: SourceDirectoryError) -> Self {
        match error {
            SourceDirectoryError::Unavailable(path) => Self::SourceUnavailable(path),
            SourceDirectoryError::NotDirectory(path) => Self::SourceNotDirectory(path),
        }
    }
}

impl From<CoreImportError> for ImportExistingServerError {
    fn from(error: CoreImportError) -> Self {
        Self::Build(error)
    }
}
