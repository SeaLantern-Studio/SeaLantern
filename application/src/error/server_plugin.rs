//! 服务器插件（plugins 目录）领域的主错误。

use std::fmt;

use sealantern_contract::error::ServerPluginServiceError;

/// 服务器插件管理操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向
/// [`ServerPluginServiceError`] 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum ServerPluginError {
    /// 指定的目标实例不存在。
    InstanceNotFound,
    /// 客户端提供的输入不合法（如文件名含路径分量、不是 jar 文件名）。
    InvalidInput,
    /// 目标插件不在实例的 plugins 目录中。
    NotFound,
    /// 底层文件系统或归档操作失败。
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

impl fmt::Display for ServerPluginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InstanceNotFound => write!(formatter, "server instance not found"),
            Self::InvalidInput => write!(formatter, "invalid server plugin input"),
            Self::NotFound => write!(formatter, "server plugin not found"),
            Self::OperationFailed { source } => {
                write!(formatter, "server plugin operation failed: {source}")
            }
            Self::Internal { source } => {
                write!(formatter, "internal server plugin task failed: {source}")
            }
        }
    }
}

impl std::error::Error for ServerPluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } | Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<sealantern_contract::InstanceServiceError> for ServerPluginError {
    fn from(source: sealantern_contract::InstanceServiceError) -> Self {
        // 实例注册表查询失败：按底层操作失败处理，细节留在 source 里供日志排查。
        Self::OperationFailed { source: Box::new(source) }
    }
}

impl From<sealantern_feature::server_plugin::ServerPluginError> for ServerPluginError {
    fn from(source: sealantern_feature::server_plugin::ServerPluginError) -> Self {
        use sealantern_feature::server_plugin::ServerPluginError as FeaturePluginError;

        match source {
            FeaturePluginError::InvalidFileName(_) => Self::InvalidInput,
            FeaturePluginError::NotFound(_) => Self::NotFound,
            FeaturePluginError::Io(_) | FeaturePluginError::Archive(_) => {
                Self::OperationFailed { source: Box::new(source) }
            }
        }
    }
}

impl From<tokio::task::JoinError> for ServerPluginError {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Internal { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<ServerPluginError> for ServerPluginServiceError {
    fn from(error: ServerPluginError) -> Self {
        match error {
            ServerPluginError::InstanceNotFound => Self::InstanceNotFound,
            ServerPluginError::InvalidInput => Self::InvalidInput,
            ServerPluginError::NotFound => Self::PluginNotFound,
            ServerPluginError::OperationFailed { .. } | ServerPluginError::Internal { .. } => {
                Self::OperationFailed
            }
        }
    }
}
