//! 设置服务错误类型。

use std::fmt;

use sealantern_infra::fs::FsError;
use sealantern_interface::error::SettingsServiceError;

/// 设置服务主错误类型。
///
/// 携带底层失败细节（source），供应用层日志排查；向 [`SettingsServiceError`]
/// 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum SettingsError {
    /// 配置加载、锁定、读取或持久化失败。
    StorageFailed {
        /// 底层文件系统错误。
        source: FsError,
    },
    /// 未分类的配置操作失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StorageFailed { source } => {
                write!(formatter, "settings storage failed: {source}")
            }
            Self::OperationFailed { source } => {
                write!(formatter, "settings operation failed: {source}")
            }
        }
    }
}

impl std::error::Error for SettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StorageFailed { source } => Some(source),
            Self::OperationFailed { source } => Some(source.as_ref()),
        }
    }
}

impl From<FsError> for SettingsError {
    fn from(source: FsError) -> Self {
        Self::StorageFailed { source }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<SettingsError> for SettingsServiceError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::StorageFailed { .. } => Self::StorageFailed,
            SettingsError::OperationFailed { .. } => Self::OperationFailed,
        }
    }
}
