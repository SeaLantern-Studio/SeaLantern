//! Network request error types.
//!
//! Covers scenarios such as configuration errors, request failures,
//! server error responses, timeouts, parse failures, and cancellations.
//! Upper layers can handle various network exceptions uniformly through `NetError`.

use std::fmt;

/// Errors that may occur during network requests.
///
/// Categorized by source and nature as follows:
///
/// | Variant | Typical Scenario |
/// |---------|-----------------|
/// | `Config` | Invalid proxy format, client construction failure |
/// | `Request` | Connection refused, DNS resolution failure, TLS handshake failure |
/// | `Response` | Server returned 4xx / 5xx |
/// | `Timeout` | Connection timeout or read timeout |
/// | `Parse` | JSON or text parsing failure |
/// | `Io` | Underlying IO error (e.g. file read/write) |
/// | `Cancelled` | Request cancelled by user |
#[derive(Debug)]
pub enum NetError {
    /// Configuration error.
    ///
    /// For example: invalid proxy address format, unable to build HTTP client.
    Config(String),
    /// Request execution failure.
    ///
    /// For example: connection refused, DNS resolution failure, TLS handshake failure, etc.
    Request(String),
    /// Server returned a non-success status code.
    ///
    /// Contains the HTTP status code and a truncated response body.
    Response(u16, String),
    /// Request timeout.
    ///
    /// Both connection timeout and read timeout fall into this category.
    Timeout,
    /// Response parsing failure.
    ///
    /// For example: invalid JSON format, non-UTF-8 text, etc.
    Parse(String),
    /// IO error.
    Io(std::io::Error),
    /// Request was actively cancelled.
    Cancelled,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::Config(msg) => write!(f, "配置错误: {}", msg),
            NetError::Request(msg) => write!(f, "请求失败: {}", msg),
            NetError::Response(code, body) => write!(f, "服务端返回 {}: {}", code, body),
            NetError::Timeout => write!(f, "请求超时"),
            NetError::Parse(msg) => write!(f, "解析失败: {}", msg),
            NetError::Io(err) => write!(f, "IO 错误: {}", err),
            NetError::Cancelled => write!(f, "请求已取消"),
        }
    }
}

impl std::error::Error for NetError {}

impl From<std::io::Error> for NetError {
    fn from(err: std::io::Error) -> Self {
        NetError::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_config_error() {
        let err = NetError::Config("bad proxy".into());
        assert_eq!(err.to_string(), "配置错误: bad proxy");
    }

    #[test]
    fn display_request_error() {
        let err = NetError::Request("connection refused".into());
        assert_eq!(err.to_string(), "请求失败: connection refused");
    }

    #[test]
    fn display_response_error() {
        let err = NetError::Response(404, "Not Found".into());
        assert_eq!(err.to_string(), "服务端返回 404: Not Found");
    }

    #[test]
    fn display_timeout() {
        let err = NetError::Timeout;
        assert_eq!(err.to_string(), "请求超时");
    }

    #[test]
    fn display_parse_error() {
        let err = NetError::Parse("invalid json".into());
        assert_eq!(err.to_string(), "解析失败: invalid json");
    }

    #[test]
    fn display_cancelled() {
        let err = NetError::Cancelled;
        assert_eq!(err.to_string(), "请求已取消");
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let net_err = NetError::from(io_err);
        assert!(matches!(net_err, NetError::Io(_)));
        assert!(net_err.to_string().contains("file not found"));
    }

    #[test]
    fn error_trait_impl() {
        let err = NetError::Timeout;
        let err_ref: &dyn std::error::Error = &err;
        assert_eq!(err_ref.to_string(), "请求超时");
    }

    #[test]
    fn debug_output() {
        let err = NetError::Config("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Config"));
        assert!(debug.contains("test"));
    }
}
