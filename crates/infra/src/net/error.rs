//! 网络请求错误类型。
//!
//! 涵盖配置错误、请求失败、
//! 服务端错误响应、超时、解析失败以及取消等场景。
//! 上层可通过 `NetError` 统一处理各类网络异常。

use std::fmt;

/// 网络请求过程中可能发生的错误。
///
/// 按来源和性质分类如下：
///
/// | Variant | Typical Scenario |
/// |---------|-----------------|
/// | `Config` | 代理格式无效、客户端构建失败 |
/// | `Request` | 已被分类并转成文字的请求失败 |
/// | `Transport` | 未被更细分类的传输层失败，保留底层 `reqwest::Error` |
/// | `Response` | 服务端返回 4xx / 5xx |
/// | `Timeout` | 连接超时或读取超时 |
/// | `Parse` | JSON 或文本解析失败 |
/// | `Io` | 底层 IO 错误（如文件读写） |
/// | `Cancelled` | 请求被用户取消 |
///
/// 只有 `Io` 与 `Transport` 能通过 [`std::error::Error::source`] 走到下层
/// 原因；其余变体携带的是已分类的文字，有意不再保留错误链。
#[derive(Debug)]
pub enum NetError {
    /// 配置错误。
    ///
    /// 例如：代理地址格式无效、无法构建 HTTP 客户端。
    Config(String),
    /// 请求执行失败，错误已被压平成文字。
    ///
    /// 例如：调用方自行构造的失败描述。若需要保留底层原因，应优先使用
    /// [`NetError::Transport`]。
    Request(String),
    /// 传输层失败，保留原始 `reqwest::Error`。
    ///
    /// 例如：连接被拒绝、DNS 解析失败、TLS 握手失败。分类时无法归入
    /// 超时或状态码错误的失败都落在这里，以便调用方继续遍历错误链。
    Transport(reqwest::Error),
    /// 服务端返回非成功状态码。
    ///
    /// 第二个字段是响应体快照（可能被截断），用于诊断；它不参与错误链。
    Response(u16, String),
    /// 请求超时。
    ///
    /// 连接超时和读取超时均归入此类别。
    Timeout,
    /// 响应解析失败。
    ///
    /// 例如：JSON 格式无效、非 UTF-8 文本等。
    Parse(String),
    /// IO 错误。
    Io(std::io::Error),
    /// 请求被主动取消。
    Cancelled,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::Config(msg) => write!(f, "config error: {}", msg),
            NetError::Request(msg) => write!(f, "request failed: {}", msg),
            NetError::Transport(err) => write!(f, "transport error: {}", err),
            NetError::Response(code, body) => write!(f, "server returned {}: {}", code, body),
            NetError::Timeout => write!(f, "request timed out"),
            NetError::Parse(msg) => write!(f, "parse error: {}", msg),
            NetError::Io(err) => write!(f, "I/O error: {}", err),
            NetError::Cancelled => write!(f, "request cancelled"),
        }
    }
}

impl std::error::Error for NetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // `Io` and `Transport` wrap a lower-level error and can be walked
        // further. The remaining variants deliberately carry already-classified
        // text with no preserved cause.
        match self {
            NetError::Io(err) => Some(err),
            NetError::Transport(err) => Some(err),
            _ => None,
        }
    }
}

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
        assert_eq!(err.to_string(), "config error: bad proxy");
    }

    #[test]
    fn display_request_error() {
        let err = NetError::Request("connection refused".into());
        assert_eq!(err.to_string(), "request failed: connection refused");
    }

    #[test]
    fn display_response_error() {
        let err = NetError::Response(404, "Not Found".into());
        assert_eq!(err.to_string(), "server returned 404: Not Found");
    }

    #[test]
    fn display_timeout() {
        let err = NetError::Timeout;
        assert_eq!(err.to_string(), "request timed out");
    }

    #[test]
    fn display_parse_error() {
        let err = NetError::Parse("invalid json".into());
        assert_eq!(err.to_string(), "parse error: invalid json");
    }

    #[test]
    fn display_cancelled() {
        let err = NetError::Cancelled;
        assert_eq!(err.to_string(), "request cancelled");
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
        assert_eq!(err_ref.to_string(), "request timed out");
    }

    #[test]
    fn io_error_exposes_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let net_err = NetError::from(io_err);
        let source = std::error::Error::source(&net_err);
        assert!(source.is_some());
    }

    #[test]
    fn classified_errors_have_no_source() {
        assert!(std::error::Error::source(&NetError::Timeout).is_none());
        assert!(std::error::Error::source(&NetError::Cancelled).is_none());
        assert!(std::error::Error::source(&NetError::Config("x".into())).is_none());
        assert!(std::error::Error::source(&NetError::Request("x".into())).is_none());
    }

    #[test]
    fn transport_error_exposes_source() {
        // 用真实 reqwest 错误构造，确认 Transport 会把它作为 source 暴露，
        // 而不是像 Request 那样压平成文字。
        let runtime = tokio::runtime::Runtime::new().expect("runtime should build");
        let err = runtime.block_on(async {
            reqwest::Client::new()
                .get("http://127.0.0.1:1/")
                .send()
                .await
                .expect_err("connecting to a closed port should fail")
        });
        let net_err = NetError::Transport(err);
        assert!(std::error::Error::source(&net_err).is_some());
        assert!(net_err.to_string().starts_with("transport error: "));
    }

    #[test]
    fn debug_output() {
        let err = NetError::Config("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Config"));
        assert!(debug.contains("test"));
    }
}
