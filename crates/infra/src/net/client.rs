//! Unified HTTP client.
//!
//! Solves the problem of `reqwest::Client` being constructed ad-hoc across the project
//! with no centralized proxy configuration management.
//! Provides a pre-configured client instance with global settings (proxy, timeout, UA, etc.),
//! so upper layers can use it directly without worrying about underlying configuration details.
//!
//! # TODO: Improve proxy configuration
//!
//! Since the refactoring is not yet complete, reading the SeaLantern config file directly is not feasible.
//! Currently `from_settings()` returns a default configuration (no proxy).
//! Once the configuration module is ready, it should be integrated to read proxy, timeout and other global settings.

use std::time::Duration;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::net::error::NetError;
use crate::net::request::RequestBuilder;
use crate::observability;

/// Retry policy.
///
/// Controls retry behavior on request failure using exponential backoff.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retries (excluding the initial request)
    pub max_retries: u32,
    /// Base delay, doubled on each retry
    pub base_delay: Duration,
    /// Upper bound for delay
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
        }
    }
}

/// Timeout policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    /// Connection timeout
    pub connect: Duration,
    /// Read timeout
    pub read: Duration,
    /// Total timeout
    pub total: Duration,
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(15),
            read: Duration::from_secs(30),
            total: Duration::from_secs(120),
        }
    }
}

/// Client configuration.
///
/// Assembles all parameters needed to create an HTTP client,
/// including proxy, timeout, UA, and retry policy.
///
/// # Examples
///
/// ```ignore
/// let config = ClientConfig {
///     proxy: Some("http://127.0.0.1:7890".into()),
///     ..Default::default()
/// };
/// let client = NetClient::from_config(&config)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// HTTP proxy address, format like `http://127.0.0.1:7890`
    pub proxy: Option<String>,
    /// Timeout policy
    pub timeout: TimeoutPolicy,
    /// User-Agent
    pub user_agent: String,
    /// Retry policy
    pub retry_policy: RetryPolicy,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            timeout: TimeoutPolicy::default(),
            user_agent: String::from("SeaLantern/0.1.0"),
            retry_policy: RetryPolicy::default(),
        }
    }
}

/// Async HTTP client.
///
/// Wraps `reqwest::Client` and automatically loads global configuration such as proxy at construction.
/// Upper layers use this struct to make requests without worrying about how the underlying client is assembled.
///
/// # Parameters
///
/// - `inner`: Internal `reqwest::Client` that actually carries out HTTP requests
/// - `retry_policy`: Retry policy for request failures
#[derive(Debug, Clone)]
pub struct NetClient {
    inner: reqwest::Client,
    retry_policy: RetryPolicy,
}

impl NetClient {
    /// Creates a client from configuration.
    ///
    /// Automatically configures proxy, timeout, UA, etc. from the provided `ClientConfig`.
    /// Returns `NetError::Config` if the proxy format is invalid or client construction fails.
    ///
    /// # Parameters
    ///
    /// - `config`: Client configuration including proxy, timeout, UA and retry policy
    ///
    /// # Returns
    ///
    /// Returns a configured `NetClient` instance; returns `NetError::Config` on configuration error
    pub fn from_config(config: &ClientConfig) -> Result<Self, NetError> {
        let mut builder = reqwest::Client::builder()
            .connect_timeout(config.timeout.connect)
            .read_timeout(config.timeout.read)
            .timeout(config.timeout.total)
            .user_agent(&config.user_agent);

        if let Some(ref proxy_url) = config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url).map_err(|e| {
                observability::proxy_config_invalid(proxy_url, &e);
                NetError::Config(format!("代理配置无效: {}", e))
            })?;
            builder = builder.proxy(proxy);
        }

        let inner = builder
            .build()
            .map_err(|e| NetError::Config(format!("创建 HTTP 客户端失败: {}", e)))?;

        Ok(Self { inner, retry_policy: config.retry_policy })
    }

    /// Creates a client with default global configuration.
    ///
    /// Currently returns a default configuration without proxy;
    /// will read global settings once the configuration module is ready.
    pub fn new() -> Result<Self, NetError> {
        Self::from_settings()
    }

    /// Loads client configuration from application global settings.
    ///
    /// Reads proxy, timeout and other configuration items from SeaLantern global settings,
    /// and creates a client accordingly.
    ///
    /// # TODO: Improve proxy configuration
    ///
    /// Integrate global settings reading; currently returns default configuration.
    pub fn from_settings() -> Result<Self, NetError> {
        // TODO: read proxy, timeout and other settings from global configuration
        Self::from_config(&ClientConfig::default())
    }

    /// Returns a reference to the internal `reqwest::Client`.
    ///
    /// For use by upper layers that need direct access to `reqwest::Client`.
    pub fn get_reqwest_client(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Returns the current retry policy.
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// Creates a GET request builder.
    ///
    /// # Parameters
    ///
    /// - `url`: The request URL
    ///
    /// # Returns
    ///
    /// Returns a `RequestBuilder` that can be chained with headers, retry policy, and then `.send()`.
    pub fn get(&self, url: impl reqwest::IntoUrl) -> Result<RequestBuilder<'_>, NetError> {
        RequestBuilder::new(self, Method::GET, url)
    }

    /// Creates a POST request builder.
    pub fn post(&self, url: impl reqwest::IntoUrl) -> Result<RequestBuilder<'_>, NetError> {
        RequestBuilder::new(self, Method::POST, url)
    }

    /// Probes a remote file for information.
    ///
    /// Sends a `Range: bytes=0-0` request to determine whether the server supports
    /// chunked downloads, and obtains the total file size.
    ///
    /// # Parameters
    ///
    /// - `url`: The file download URL
    ///
    /// # Returns
    ///
    /// Returns `RemoteFileInfo` containing the total file size and whether Range is supported.
    ///
    /// # Errors
    ///
    /// Returns `NetError::Parse` if the server does not support Range and no `Content-Length` header is present.
    pub async fn probe(&self, url: &str) -> Result<RemoteFileInfo, NetError> {
        let resp = self.get(url)?.header("Range", "bytes=0-0").send().await?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(NetError::Response(
                resp.status().as_u16(),
                format!("探测请求失败，状态码: {}", resp.status()),
            ));
        }

        let supports_range = resp.status() == reqwest::StatusCode::PARTIAL_CONTENT;

        let total_size = if supports_range {
            parse_content_range(resp.headers())?
        } else {
            resp.content_length()
                .ok_or_else(|| NetError::Parse("服务器未返回 Content-Length".into()))?
        };

        Ok(RemoteFileInfo { total_size, supports_range })
    }
}

/// Remote file information.
#[derive(Debug, Clone, Copy)]
pub struct RemoteFileInfo {
    /// Total file size (bytes)
    pub total_size: u64,
    /// Whether the server supports Range requests
    pub supports_range: bool,
}

/// Parses the total file size from a `Content-Range` header.
///
/// Format: `bytes 0-0/12345` -> returns `Ok(12345)`
fn parse_content_range(headers: &reqwest::header::HeaderMap) -> Result<u64, NetError> {
    let value = headers
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| NetError::Parse("服务器返回 206 但缺少 Content-Range 头部".into()))?;

    let total = value
        .rsplit('/')
        .next()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .ok_or_else(|| NetError::Parse(format!("Content-Range 格式无法解析: {}", value)))?;

    Ok(total)
}

/// Blocking HTTP client.
///
/// Functionally identical to `NetClient`, but implemented using `reqwest::blocking`.
/// Suitable for synchronous code scenarios. Only compiled when the `blocking` feature is enabled.
///
/// # Parameters
///
/// - `inner`: Internal `reqwest::blocking::Client`
/// - `retry_policy`: Retry policy for request failures
#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct NetBlockingClient {
    inner: reqwest::blocking::Client,
    retry_policy: RetryPolicy,
}

#[cfg(feature = "blocking")]
impl NetBlockingClient {
    /// Creates a blocking client from configuration.
    ///
    /// # Parameters
    ///
    /// - `config`: Client configuration including proxy, timeout, UA and retry policy
    ///
    /// # Returns
    ///
    /// Returns a configured `NetBlockingClient` instance; returns `NetError::Config` on configuration error
    pub fn from_config(config: &ClientConfig) -> Result<Self, NetError> {
        let mut builder = reqwest::blocking::Client::builder()
            .connect_timeout(config.timeout.connect)
            .timeout(config.timeout.total)
            .user_agent(&config.user_agent);

        if let Some(ref proxy_url) = config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url).map_err(|e| {
                observability::proxy_config_invalid(proxy_url, &e);
                NetError::Config(format!("代理配置无效: {}", e))
            })?;
            builder = builder.proxy(proxy);
        }

        let inner = builder
            .build()
            .map_err(|e| NetError::Config(format!("创建阻塞 HTTP 客户端失败: {}", e)))?;

        Ok(Self { inner, retry_policy: config.retry_policy })
    }

    /// Creates a blocking client with default global configuration.
    ///
    /// Currently returns a default configuration without proxy;
    /// will read global settings once the configuration module is ready.
    pub fn new() -> Result<Self, NetError> {
        Self::from_settings()
    }

    /// Loads configuration from application global settings.
    ///
    /// # TODO: Improve proxy configuration
    ///
    /// Integrate global settings reading; currently returns default configuration.
    pub fn from_settings() -> Result<Self, NetError> {
        Self::from_config(&ClientConfig::default())
    }

    /// Returns a reference to the internal `reqwest::blocking::Client`.
    pub fn get_reqwest_client(&self) -> &reqwest::blocking::Client {
        &self.inner
    }

    /// Returns the current retry policy.
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_proxy() {
        let config = ClientConfig::default();
        assert!(config.proxy.is_none());
        assert_eq!(config.timeout.connect, Duration::from_secs(15));
    }

    #[test]
    fn retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.base_delay, Duration::from_secs(1));
        assert_eq!(policy.max_delay, Duration::from_secs(10));
    }

    #[test]
    fn timeout_policy_defaults() {
        let policy = TimeoutPolicy::default();
        assert_eq!(policy.connect, Duration::from_secs(15));
        assert_eq!(policy.read, Duration::from_secs(30));
        assert_eq!(policy.total, Duration::from_secs(120));
    }

    #[test]
    fn from_config_without_proxy_succeeds() {
        let config = ClientConfig::default();
        let client = NetClient::from_config(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn from_config_with_valid_proxy_succeeds() {
        let config = ClientConfig {
            proxy: Some("http://127.0.0.1:7890".into()),
            ..Default::default()
        };
        let client = NetClient::from_config(&config);
        assert!(client.is_ok());
    }
}
