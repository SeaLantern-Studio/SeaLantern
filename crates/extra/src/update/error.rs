//! 更新检查编排错误。

use std::fmt;

/// 更新检查失败的底层错误。
#[derive(Debug)]
pub enum UpdateCheckError {
    /// HTTP 客户端初始化失败。
    ClientInitialization {
        source: sealantern_infra::net::NetError,
    },
    /// 单一更新源检查失败。
    ProviderFailed {
        provider: &'static str,
        message: String,
    },
    /// Linux 通用发行版的两个更新源均检查失败。
    ProvidersFailed { cnb: String, github: String },
}

impl fmt::Display for UpdateCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientInitialization { source } => {
                write!(formatter, "update HTTP client initialization failed: {source}")
            }
            Self::ProviderFailed { provider, message } => {
                write!(formatter, "{provider} update check failed: {message}")
            }
            Self::ProvidersFailed { cnb, github } => {
                write!(formatter, "update checks failed; cnb: {cnb}; github: {github}")
            }
        }
    }
}

impl std::error::Error for UpdateCheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ClientInitialization { source } => Some(source),
            Self::ProviderFailed { .. } | Self::ProvidersFailed { .. } => None,
        }
    }
}
