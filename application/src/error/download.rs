//! 下载任务管理领域的主错误。

use std::fmt;

use sealantern_contract::error::DownloadServiceError;
use sealantern_infra::download::DownloadError as InfraDownloadError;

/// 下载任务管理操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向 [`DownloadServiceError`]
/// 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum DownloadError {
    /// 指定的下载任务不存在。
    TaskNotFound,
    /// 客户端提供的输入不合法。
    InvalidInput,
    /// 底层网络 / IO 操作失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for DownloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskNotFound => write!(formatter, "download task not found"),
            Self::InvalidInput => write!(formatter, "invalid input"),
            Self::OperationFailed { source } => {
                write!(formatter, "download operation failed: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<InfraDownloadError> for DownloadError {
    fn from(source: InfraDownloadError) -> Self {
        Self::OperationFailed { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<DownloadError> for DownloadServiceError {
    fn from(error: DownloadError) -> Self {
        match error {
            DownloadError::TaskNotFound => Self::TaskNotFound,
            DownloadError::InvalidInput => Self::InvalidInput,
            DownloadError::OperationFailed { .. } => Self::OperationFailed,
            DownloadError::Unsupported => Self::Unsupported,
        }
    }
}
