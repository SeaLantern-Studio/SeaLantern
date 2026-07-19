//! 统一 HTTP 客户端。
//!
//! 解决项目内 `reqwest::Client` 各处各自构建、代理配置无法统一管控的问题。
//! 提供已加载全局配置（代理、超时、UA 等）的客户端实例，
//! 上层直接拿来用，不需要关心底层配置细节。
//!
//! # TODO:完善代理配置
//!
//! 由于重构尚未完成，直接读取 SeaLantern 配置文件不可行，
//! 目前 `from_settings()` 返回默认配置（无代理）。
//! 待配置模块就绪后，应接入配置模块读取代理、超时等全局设置。

use std::time::Duration;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::net::error::NetError;
use crate::net::request::RequestBuilder;
use crate::observability;

/// 重试策略。
///
/// 控制请求失败时的重试行为，使用指数退避算法。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数（不包含首次请求）
    pub max_retries: u32,
    /// 基础等待时间，每次重试翻倍
    pub base_delay: Duration,
    /// 最大等待时间上限
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

/// 超时策略。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    /// 连接超时
    pub connect: Duration,
    /// 读取超时
    pub read: Duration,
    /// 总体超时
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

/// 客户端配置。
///
/// 组装创建 HTTP 客户端所需的所有参数，包括代理、超时、UA 和重试策略。
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
    /// HTTP 代理地址，格式如 `http://127.0.0.1:7890`
    pub proxy: Option<String>,
    /// 超时策略
    pub timeout: TimeoutPolicy,
    /// User-Agent
    pub user_agent: String,
    /// 重试策略
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

/// 异步 HTTP 客户端。
///
/// 封装 `reqwest::Client`，在构建时自动加载代理等全局配置。
/// 上层通过此结构体发起请求，无需关心底层客户端如何组装。
///
/// # Parameters
///
/// - `inner`: 内部 `reqwest::Client`，实际承载 HTTP 请求
/// - `retry_policy`: 请求失败时的重试策略
#[derive(Debug, Clone)]
pub struct NetClient {
    inner: reqwest::Client,
    retry_policy: RetryPolicy,
}

impl NetClient {
    /// 从配置创建客户端。
    ///
    /// 根据传入的 `ClientConfig` 自动配置代理、超时、UA 等参数。
    /// 代理格式无效或客户端构建失败时返回 `NetError::Config`。
    ///
    /// # Parameters
    ///
    /// - `config`: 客户端配置，包含代理、超时、UA 和重试策略
    ///
    /// # Returns
    ///
    /// 返回配置好的 `NetClient` 实例；配置错误时返回 `NetError::Config`
    pub fn from_config(config: &ClientConfig) -> Result<Self, NetError> {
        let mut builder = reqwest::Client::builder()
            .connect_timeout(config.timeout.connect)
            .read_timeout(config.timeout.read)
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

    /// 使用默认全局配置创建客户端。
    ///
    /// 当前返回无代理的默认配置，待配置模块就绪后将读取全局设置。
    pub fn new() -> Result<Self, NetError> {
        Self::from_settings()
    }

    /// 从应用全局设置加载客户端配置。
    ///
    /// 读取 SeaLantern 全局设置中的代理、超时等配置项，
    /// 并据此创建客户端。
    ///
    /// # TODO:完善代理配置
    ///
    /// 接入全局设置读取逻辑，当前返回默认配置。
    pub fn from_settings() -> Result<Self, NetError> {
        // TODO: 从全局设置读取代理、超时等配置
        Self::from_config(&ClientConfig::default())
    }

    /// 返回内部的 `reqwest::Client` 引用。
    ///
    /// 供上层在需要直接操作 `reqwest::Client` 时使用。
    pub fn get_reqwest_client(&self) -> &reqwest::Client {
        &self.inner
    }

    /// 返回当前重试策略。
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// 创建一个 GET 请求构建器。
    ///
    /// # Parameters
    ///
    /// - `url`: 请求地址
    ///
    /// # Returns
    ///
    /// 返回 `RequestBuilder`，可链式配置 header、重试策略后调用 `.send()`。
    pub fn get(&self, url: impl reqwest::IntoUrl) -> Result<RequestBuilder<'_>, NetError> {
        RequestBuilder::new(self, Method::GET, url)
    }

    /// 创建一个 POST 请求构建器。
    pub fn post(&self, url: impl reqwest::IntoUrl) -> Result<RequestBuilder<'_>, NetError> {
        RequestBuilder::new(self, Method::POST, url)
    }

    /// 探测远端文件信息。
    ///
    /// 发送 `Range: bytes=0-0` 请求，判断服务器是否支持分片下载，
    /// 并获取文件总大小。
    ///
    /// # Parameters
    ///
    /// - `url`: 文件下载地址
    ///
    /// # Returns
    ///
    /// 返回 `RemoteFileInfo`，包含文件总大小和是否支持 Range 分片。
    ///
    /// # Errors
    ///
    /// 服务器不支持 Range 且未返回 `Content-Length` 时返回 `NetError::Parse`。
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

/// 远端文件信息。
#[derive(Debug, Clone, Copy)]
pub struct RemoteFileInfo {
    /// 文件总大小（字节）
    pub total_size: u64,
    /// 服务器是否支持 Range 分片请求
    pub supports_range: bool,
}

/// 从 `Content-Range` 头部解析文件总大小。
///
/// 格式: `bytes 0-0/12345` → 返回 `Ok(12345)`
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

/// 阻塞 HTTP 客户端。
///
/// 与 `NetClient` 功能相同，但使用 `reqwest::blocking` 实现，
/// 适用于同步代码场景。仅在 `blocking` feature 启用时编译。
///
/// # Parameters
///
/// - `inner`: 内部 `reqwest::blocking::Client`
/// - `retry_policy`: 请求失败时的重试策略
#[cfg(feature = "blocking")]
#[derive(Debug)]
pub struct NetBlockingClient {
    inner: reqwest::blocking::Client,
    retry_policy: RetryPolicy,
}

#[cfg(feature = "blocking")]
impl NetBlockingClient {
    /// 从配置创建阻塞客户端。
    ///
    /// # Parameters
    ///
    /// - `config`: 客户端配置，包含代理、超时、UA 和重试策略
    ///
    /// # Returns
    ///
    /// 返回配置好的 `NetBlockingClient` 实例；配置错误时返回 `NetError::Config`
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

    /// 使用默认全局配置创建阻塞客户端。
    ///
    /// 当前返回无代理的默认配置，待配置模块就绪后将读取全局设置。
    pub fn new() -> Result<Self, NetError> {
        Self::from_settings()
    }

    /// 从应用全局设置加载。
    ///
    /// # TODO:完善代理配置
    ///
    /// 接入全局设置读取逻辑，当前返回默认配置。
    pub fn from_settings() -> Result<Self, NetError> {
        Self::from_config(&ClientConfig::default())
    }

    /// 返回内部的 `reqwest::blocking::Client` 引用。
    pub fn get_reqwest_client(&self) -> &reqwest::blocking::Client {
        &self.inner
    }

    /// 返回当前重试策略。
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
