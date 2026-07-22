//! 市场操作相关的错误类型定义。
//!
//! 本模块定义了 [`MarketError`] 枚举，涵盖了与远程市场 API
//! 交互过程中可能出现的各类错误，包括 HTTP 请求失败、
//! JSON 解析错误、资源未找到、下载失败以及配置错误等。

use std::fmt;

/// 市场操作过程中可能发生的所有错误。
///
/// 该枚举实现了 [`std::error::Error`] trait。
#[derive(Debug)]
pub enum MarketError {
    /// HTTP 请求失败，包含操作名和来源错误。
    Http {
        /// 正在尝试的操作。
        operation: &'static str,
        /// 底层的错误信息。
        source: String,
    },
    /// JSON 解析或序列化失败。
    Json {
        /// 正在尝试的操作。
        operation: &'static str,
        /// 具体的错误描述。
        message: String,
    },
    /// 请求的资源（插件、模组、版本等）未找到。
    NotFound {
        /// 未找到的资源标识。
        resource: String,
    },
    /// 资源文件下载失败。
    Download(String),
    /// 市场模块配置错误。
    Config(String),
}

impl fmt::Display for MarketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketError::Http { operation, source } => {
                write!(f, "failed to {operation}: {source}")
            }
            MarketError::Json { operation, message } => {
                write!(f, "failed to {operation}: {message}")
            }
            MarketError::NotFound { resource } => {
                write!(f, "resource not found: {resource}")
            }
            MarketError::Download(msg) => {
                write!(f, "download error: {msg}")
            }
            MarketError::Config(msg) => {
                write!(f, "config error: {msg}")
            }
        }
    }
}

impl std::error::Error for MarketError {}

impl MarketError {
    /// 将 HTTP 错误与操作名称一同包装。
    pub(crate) fn http(operation: &'static str, source: impl Into<String>) -> Self {
        MarketError::Http { operation, source: source.into() }
    }

    /// 将 JSON 解析错误与操作名称一同包装。
    pub(crate) fn json(operation: &'static str, message: impl Into<String>) -> Self {
        MarketError::Json { operation, message: message.into() }
    }

    pub(crate) fn config(message: impl Into<String>) -> Self {
        MarketError::Config(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_error_includes_operation_and_source() {
        let err = MarketError::http("fetch mod list", "connection refused");
        assert_eq!(err.to_string(), "failed to fetch mod list: connection refused");
    }

    #[test]
    fn json_error_includes_operation_and_message() {
        let err = MarketError::json("parse manifest", "missing field 'version'");
        assert_eq!(err.to_string(), "failed to parse manifest: missing field 'version'");
    }

    #[test]
    fn config_displays_message() {
        let err = MarketError::config("missing API key");
        assert_eq!(err.to_string(), "config error: missing API key");
    }

    #[test]
    fn error_trait_is_implemented() {
        fn assert_error(_: &dyn std::error::Error) {}
        assert_error(&MarketError::config("test"));
    }
}
