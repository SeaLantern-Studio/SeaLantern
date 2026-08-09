//! 应用更新检查领域的主错误。

use std::fmt;

use sealantern_extra::update::UpdateCheckError as ExtraUpdateCheckError;
use sealantern_interface::UpdateCheckServiceError;

/// 应用更新检查失败的内部错误。
#[derive(Debug)]
pub enum UpdateCheckError {
    /// 更新源客户端初始化或远程检查失败。
    CheckFailed { source: ExtraUpdateCheckError },
    /// 更新源返回了应用层无法识别的数据。
    InvalidResponse { detail: String },
    /// 当前宿主不支持更新检查。
    Unsupported,
}

impl fmt::Display for UpdateCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CheckFailed { source } => write!(formatter, "update check failed: {source}"),
            Self::InvalidResponse { detail } => {
                write!(formatter, "invalid update response: {detail}")
            }
            Self::Unsupported => formatter.write_str("update check not supported"),
        }
    }
}

impl std::error::Error for UpdateCheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CheckFailed { source } => Some(source),
            Self::InvalidResponse { .. } | Self::Unsupported => None,
        }
    }
}

impl From<ExtraUpdateCheckError> for UpdateCheckError {
    fn from(source: ExtraUpdateCheckError) -> Self {
        Self::CheckFailed { source }
    }
}

impl From<UpdateCheckError> for UpdateCheckServiceError {
    fn from(error: UpdateCheckError) -> Self {
        match error {
            UpdateCheckError::CheckFailed { .. } | UpdateCheckError::InvalidResponse { .. } => {
                Self::CheckFailed
            }
            UpdateCheckError::Unsupported => Self::Unsupported,
        }
    }
}
