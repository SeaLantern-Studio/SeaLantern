//! 实例启动配置（SeaLantern/config.toml）领域的主错误。

use std::fmt;

use sealantern_contract::error::ServerStartupServiceError;

/// 启动配置管理操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向
/// [`ServerStartupServiceError`] 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum ServerStartupError {
    /// 指定的目标实例不存在。
    InstanceNotFound,
    /// 客户端提供的输入不合法（如最小内存大于最大内存、配置无法解析）。
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
}

impl fmt::Display for ServerStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InstanceNotFound => write!(formatter, "server instance not found"),
            Self::InvalidInput => write!(formatter, "invalid server startup config"),
            Self::OperationFailed { source } => {
                write!(formatter, "server startup config operation failed: {source}")
            }
            Self::Internal { source } => {
                write!(formatter, "internal server startup config task failed: {source}")
            }
        }
    }
}

impl std::error::Error for ServerStartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } | Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<sealantern_feature::server_startup::ServerStartupError> for ServerStartupError {
    fn from(source: sealantern_feature::server_startup::ServerStartupError) -> Self {
        use sealantern_feature::server_startup::ServerStartupError as FeatureStartupError;

        match source {
            FeatureStartupError::InvalidInput(_) => Self::InvalidInput,
            FeatureStartupError::Io(_) => Self::OperationFailed { source: Box::new(source) },
        }
    }
}

impl From<sealantern_contract::InstanceServiceError> for ServerStartupError {
    fn from(source: sealantern_contract::InstanceServiceError) -> Self {
        // 实例注册表查询失败：按底层操作失败处理，细节留在 source 里供日志排查。
        Self::OperationFailed { source: Box::new(source) }
    }
}

impl From<tokio::task::JoinError> for ServerStartupError {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Internal { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<ServerStartupError> for ServerStartupServiceError {
    fn from(error: ServerStartupError) -> Self {
        match error {
            ServerStartupError::InstanceNotFound => Self::InstanceNotFound,
            ServerStartupError::InvalidInput => Self::InvalidInput,
            ServerStartupError::OperationFailed { .. } | ServerStartupError::Internal { .. } => {
                Self::OperationFailed
            }
        }
    }
}
