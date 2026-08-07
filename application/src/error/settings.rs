//! 设置服务错误类型。

use sealantern_infra::fs::FsError;
use sealantern_interface::SettingsServiceError;

/// 设置服务主错误类型。
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// 配置操作失败。
    #[error("settings operation failed: {0}")]
    OperationFailed(#[from] FsError),

    /// 内部异步任务错误。
    #[error("internal task error: {0}")]
    Internal(#[from] tokio::task::JoinError),
}

impl From<SettingsError> for SettingsServiceError {
    fn from(value: SettingsError) -> Self {
        match value {
            SettingsError::OperationFailed(_) => SettingsServiceError::OperationFailed,
            SettingsError::Internal(_) => SettingsServiceError::OperationFailed,
        }
    }
}