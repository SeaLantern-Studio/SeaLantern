//! 实例启动配置领域的主错误。

use std::fmt;

use sealantern_contract::ServerStartupServiceError;

/// 实例启动配置操作失败的应用层主错误。
///
/// 携带底层失败细节（source），供应用层日志排查；向
/// [`ServerStartupServiceError`] 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum ServerStartupError {
    /// 客户端提供的输入不合法（如最小内存大于最大内存）。
    InvalidInput,
    /// 底层实例文档读写失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl fmt::Display for ServerStartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(formatter, "invalid server startup config"),
            Self::OperationFailed { source } => {
                write!(formatter, "server startup config operation failed: {source}")
            }
        }
    }
}

impl std::error::Error for ServerStartupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source } => Some(source.as_ref()),
            Self::InvalidInput => None,
        }
    }
}

impl From<sealantern_feature::config::instance::DocumentError> for ServerStartupError {
    fn from(source: sealantern_feature::config::instance::DocumentError) -> Self {
        Self::OperationFailed { source: Box::new(source) }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
impl From<ServerStartupError> for ServerStartupServiceError {
    fn from(error: ServerStartupError) -> Self {
        match error {
            ServerStartupError::InvalidInput => Self::InvalidInput,
            ServerStartupError::OperationFailed { .. } => Self::OperationFailed,
        }
    }
}
