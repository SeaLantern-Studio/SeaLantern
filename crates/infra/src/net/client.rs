//! 统一 HTTP 客户端。
//!
//! 防止 `reqwest::Client` 的构造分散在项目各处。
//! 调用方提供已解析的代理、超时、User-Agent 和重试行为设置。
//! 应用程序配置和系统代理检测保留在此模块之外。

use std::time::Duration;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::net::error::NetError;
use crate::net::proxy::EffectiveProxy;
use crate::net::request::RequestBuilder;
use crate::observability;

/// 重试策略。
///
/// 控制请求失败时的重试行为，使用指数退避策略。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数（不包括初始请求）
    pub max_retries: u32,
    /// 基础延迟，每次重试翻倍
    pub base_delay: Duration,
    /// 延迟上限
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
    /// 总超时
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
/// 组装创建 HTTP 客户端所需的所有参数，
/// 包括代理、超时、UA 和重试策略。
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
    /// 用户代理标识
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
/// 封装由调用方提供的配置构建的 `reqwest::Client`。
/// 上层使用此结构体，无需直接组装 HTTP 客户端。
///
/// # Parameters
///
/// - `inner`: 内部 `reqwest::Client`，实际执行 HTTP 请求
/// - `retry_policy`: 请求失败的重试策略
#[derive(Debug, Clone)]
pub struct NetClient {
    inner: reqwest::Client,
    retry_policy: RetryPolicy,
}

impl NetClient {
    /// 从配置创建客户端。
    ///
    /// 从提供的 `ClientConfig` 自动配置代理、超时、UA 等。
    /// 如果代理格式无效或客户端构建失败，返回 `NetError::Config`。
    ///
    /// # Parameters
    ///
    /// - `config`: 客户端配置，包括代理、超时、UA 和重试策略
    ///
    /// # Returns
    ///
    /// 返回配置好的 `NetClient` 实例；配置错误时返回 `NetError::Config`
    pub fn from_config(config: &ClientConfig) -> Result<Self, NetError> {
        let effective_proxy = config
            .proxy
            .as_deref()
            .map(EffectiveProxy::proxy)
            .unwrap_or(EffectiveProxy::Direct);
        Self::from_config_with_effective_proxy(config, &effective_proxy)
    }

    /// 从基础设置和已解析的代理策略创建客户端。
    pub fn from_config_with_effective_proxy(
        config: &ClientConfig,
        effective_proxy: &EffectiveProxy,
    ) -> Result<Self, NetError> {
        let mut builder = reqwest::Client::builder()
            .connect_timeout(config.timeout.connect)
            .read_timeout(config.timeout.read)
            .timeout(config.timeout.total)
            .user_agent(&config.user_agent);

        builder = apply_async_proxy_routes(builder, effective_proxy)?;

        let inner = builder.build().map_err(|_| {
            observability::http_client_build_failed();
            NetError::Config("无法创建 HTTP 客户端".into())
        })?;

        Ok(Self { inner, retry_policy: config.retry_policy })
    }

    /// 使用默认配置创建客户端。
    pub fn new() -> Result<Self, NetError> {
        Self::from_settings()
    }

    /// 从默认配置创建客户端，用于兼容性。
    ///
    /// 有意不从 `infra` 读取应用程序设置；组合
    /// 根节点必须自行解析它们并使用 [`Self::from_config`]。
    pub fn from_settings() -> Result<Self, NetError> {
        Self::from_config(&ClientConfig::default())
    }

    /// 返回内部 `reqwest::Client` 的引用。
    ///
    /// 供需要直接访问 `reqwest::Client` 的上层使用。
    pub fn get_reqwest_client(&self) -> &reqwest::Client {
        &self.inner
    }

    /// 返回当前的重试策略。
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// 创建 GET 请求构建器。
    ///
    /// # Parameters
    ///
    /// - `url`: 请求 URL
    ///
    /// # Returns
    ///
    /// 返回一个 `RequestBuilder`，可以链式添加头部、重试策略，然后调用 `.send()`。
    pub fn get(&self, url: impl reqwest::IntoUrl) -> Result<RequestBuilder<'_>, NetError> {
        RequestBuilder::new(self, Method::GET, url)
    }

    /// 创建 POST 请求构建器。
    pub fn post(&self, url: impl reqwest::IntoUrl) -> Result<RequestBuilder<'_>, NetError> {
        RequestBuilder::new(self, Method::POST, url)
    }

    /// 探测远程文件信息。
    ///
    /// 发送 `Range: bytes=0-0` 请求，判断服务器是否支持
    /// 分块下载，并获取文件总大小。
    ///
    /// # Parameters
    ///
    /// - `url`: 文件下载 URL
    ///
    /// # Returns
    ///
    /// 返回 `RemoteFileInfo`，包含文件总大小以及是否支持 Range。
    ///
    /// # Errors
    ///
    /// 如果服务器不支持 Range 且没有 `Content-Length` 头部，返回 `NetError::Parse`。
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

/// 远程文件信息。
#[derive(Debug, Clone, Copy)]
pub struct RemoteFileInfo {
    /// 文件总大小（字节）
    pub total_size: u64,
    /// 服务器是否支持 Range 请求
    pub supports_range: bool,
}

/// 从 `Content-Range` 头部解析文件总大小。
///
/// 格式：`bytes 0-0/12345` -> 返回 `Ok(12345)`
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
/// 功能上与 `NetClient` 相同，但使用 `reqwest::blocking` 实现。
/// 适用于同步代码场景。仅在启用 `blocking` 特性时编译。
///
/// # Parameters
///
/// - `inner`: 内部 `reqwest::blocking::Client`
/// - `retry_policy`: 请求失败的重试策略
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
    /// - `config`: 客户端配置，包括代理、超时、UA 和重试策略
    ///
    /// # Returns
    ///
    /// 返回配置好的 `NetBlockingClient` 实例；配置错误时返回 `NetError::Config`
    pub fn from_config(config: &ClientConfig) -> Result<Self, NetError> {
        let effective_proxy = config
            .proxy
            .as_deref()
            .map(EffectiveProxy::proxy)
            .unwrap_or(EffectiveProxy::Direct);
        Self::from_config_with_effective_proxy(config, &effective_proxy)
    }

    /// 从基础设置和已解析的代理策略创建阻塞客户端。
    pub fn from_config_with_effective_proxy(
        config: &ClientConfig,
        effective_proxy: &EffectiveProxy,
    ) -> Result<Self, NetError> {
        let mut builder = reqwest::blocking::Client::builder()
            .connect_timeout(config.timeout.connect)
            .timeout(config.timeout.total)
            .user_agent(&config.user_agent);

        builder = apply_blocking_proxy_routes(builder, effective_proxy)?;

        let inner = builder.build().map_err(|_| {
            observability::http_client_build_failed();
            NetError::Config("无法创建阻塞 HTTP 客户端".into())
        })?;

        Ok(Self { inner, retry_policy: config.retry_policy })
    }

    /// 使用默认配置创建阻塞客户端。
    pub fn new() -> Result<Self, NetError> {
        Self::from_settings()
    }

    /// 从默认配置创建阻塞客户端，用于兼容性。
    ///
    /// 有意不从 `infra` 读取应用程序设置；组合
    /// 根节点必须自行解析它们并使用 [`Self::from_config`]。
    pub fn from_settings() -> Result<Self, NetError> {
        Self::from_config(&ClientConfig::default())
    }

    /// 返回内部 `reqwest::blocking::Client` 的引用。
    pub fn get_reqwest_client(&self) -> &reqwest::blocking::Client {
        &self.inner
    }

    /// 返回当前的重试策略。
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }
}

fn no_proxy(routes: &crate::net::proxy::ProxyRoutes) -> Option<reqwest::NoProxy> {
    if routes.no_proxy().is_empty() {
        None
    } else {
        reqwest::NoProxy::from_string(&routes.no_proxy().join(","))
    }
}

fn proxy_config_error(scope: &str) -> NetError {
    observability::proxy_config_invalid(scope);
    NetError::Config(format!("{scope} 代理配置无效"))
}

fn configured_proxy_routes(
    effective_proxy: &EffectiveProxy,
) -> Result<[Option<reqwest::Proxy>; 2], NetError> {
    let Some(routes) = effective_proxy.routes_ref() else {
        return Ok([None, None]);
    };
    let no_proxy = no_proxy(routes);

    let http_proxy = routes
        .http_proxy()
        .map(|proxy_url| {
            reqwest::Proxy::http(proxy_url)
                .map_err(|_| proxy_config_error("HTTP"))
                .map(|proxy| proxy.no_proxy(no_proxy.clone()))
        })
        .transpose()?;
    let https_proxy = routes
        .https_proxy()
        .map(|proxy_url| {
            reqwest::Proxy::https(proxy_url)
                .map_err(|_| proxy_config_error("HTTPS"))
                .map(|proxy| proxy.no_proxy(no_proxy))
        })
        .transpose()?;

    Ok([http_proxy, https_proxy])
}

fn apply_async_proxy_routes(
    mut builder: reqwest::ClientBuilder,
    effective_proxy: &EffectiveProxy,
) -> Result<reqwest::ClientBuilder, NetError> {
    for proxy in configured_proxy_routes(effective_proxy)?
        .into_iter()
        .flatten()
    {
        builder = builder.proxy(proxy);
    }
    Ok(builder)
}

#[cfg(feature = "blocking")]
fn apply_blocking_proxy_routes(
    mut builder: reqwest::blocking::ClientBuilder,
    effective_proxy: &EffectiveProxy,
) -> Result<reqwest::blocking::ClientBuilder, NetError> {
    for proxy in configured_proxy_routes(effective_proxy)?
        .into_iter()
        .flatten()
    {
        builder = builder.proxy(proxy);
    }
    Ok(builder)
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

    #[test]
    fn effective_proxy_routes_build_a_client() {
        let effective_proxy = EffectiveProxy::routes(
            crate::net::proxy::ProxyRoutes::split(
                Some("http://127.0.0.1:7890".into()),
                Some("http://127.0.0.1:7891".into()),
            )
            .with_no_proxy(vec!["localhost".into()]),
        );

        assert!(NetClient::from_config_with_effective_proxy(
            &ClientConfig::default(),
            &effective_proxy
        )
        .is_ok());
    }

    #[test]
    fn invalid_proxy_error_does_not_echo_credentials() {
        let effective_proxy = EffectiveProxy::proxy("http://user:secret@[::1");
        let error =
            NetClient::from_config_with_effective_proxy(&ClientConfig::default(), &effective_proxy)
                .unwrap_err();

        assert!(matches!(error, NetError::Config(_)));
        assert!(!error.to_string().contains("secret"));
    }
}
