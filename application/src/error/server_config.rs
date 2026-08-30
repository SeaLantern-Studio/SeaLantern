//! 服务器配置（server.properties）领域的主错误。

use std::fmt;

use sealantern_contract::error::ServerConfigServiceError;

/// 服务器配置操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向
/// [`ServerConfigServiceError`] 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum ServerConfigError {
    /// 客户端提供的输入不合法（如路径为空、源码无法解析）。
    InvalidInput,
    /// 底层配置文件读写失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 内部异步任务失败（如阻塞任务被取消或 panic）。
    Internal {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for ServerConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(formatter, "invalid server config input"),
            Self::OperationFailed { source } => {
                write!(formatter, "server config operation failed: {source}")
            }
            Self::Internal { source } => {
                write!(formatter, "internal server config task failed: {source}")
            }
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for ServerConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } | Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<sealantern_feature::config::server::ServerPropertiesError> for ServerConfigError {
    fn from(source: sealantern_feature::config::server::ServerPropertiesError) -> Self {
        use sealantern_feature::config::server::ServerPropertiesError as FeatureServerConfigError;

        match source {
            FeatureServerConfigError::Parse(_) => Self::InvalidInput,
            FeatureServerConfigError::Io(_) => Self::OperationFailed { source: Box::new(source) },
        }
    }
}

impl From<tokio::task::JoinError> for ServerConfigError {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Internal { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<ServerConfigError> for ServerConfigServiceError {
    fn from(error: ServerConfigError) -> Self {
        match error {
            ServerConfigError::InvalidInput => Self::InvalidInput,
            ServerConfigError::OperationFailed { .. } | ServerConfigError::Internal { .. } => {
                Self::OperationFailed
            }
            ServerConfigError::Unsupported => Self::Unsupported,
        }
    }
}
