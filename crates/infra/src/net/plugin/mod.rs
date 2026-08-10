//! 插件受约束出站网络执行。
//!
//! 上层负责插件身份、能力声明、审批与策略审计；本模块只负责确保实际网络连接
//! 不超出已经批准的 origin、地址范围和资源限制。

mod executor;

use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

use bytes::Bytes;
use ipnet::IpNet;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use url::{Host, Url};

pub use executor::{PluginNetworkClient, PluginNetworkExecutor};

/// 规范化后的精确 HTTP(S) origin。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkOrigin {
    scheme: String,
    host: String,
    port: u16,
}

impl NetworkOrigin {
    pub fn parse(value: &str) -> Result<Self, PluginNetworkError> {
        let url = Url::parse(value).map_err(|_| PluginNetworkError::InvalidOrigin)?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(PluginNetworkError::InvalidOrigin);
        }

        let host = normalized_url_host(&url).ok_or(PluginNetworkError::InvalidOrigin)?;
        let port = url
            .port_or_known_default()
            .ok_or(PluginNetworkError::InvalidOrigin)?;
        Ok(Self {
            scheme: url.scheme().to_owned(),
            host,
            port,
        })
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    fn matches(&self, url: &Url) -> bool {
        url.scheme() == self.scheme
            && normalized_url_host(url).as_deref() == Some(self.host.as_str())
            && url.port_or_known_default() == Some(self.port)
    }
}

/// 上层为私有网络能力明确批准的目标。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowedNetworkTarget {
    Exact(IpAddr),
    Network(IpNet),
}

impl AllowedNetworkTarget {
    fn contains(&self, address: IpAddr) -> bool {
        let address = canonical_ip(address);
        match self {
            Self::Exact(expected) => canonical_ip(*expected) == address,
            Self::Network(network) => network.contains(&address),
        }
    }
}

/// 已由上层能力策略选定的地址访问类别。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginNetworkAddressPolicy {
    /// 仅允许 HTTPS 公网单播地址，且不允许认证请求头。
    PublicOnly,
    /// 仅允许显式列出的地址或网段；回环、链路本地等地址仍始终拒绝。
    AuthenticatedOrPrivate {
        allowed_targets: Vec<AllowedNetworkTarget>,
    },
}

/// 已获批准的精确网络执行作用域。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginNetworkScope {
    origin: NetworkOrigin,
    address_policy: PluginNetworkAddressPolicy,
}

impl PluginNetworkScope {
    /// 构造已经通过上层策略审批的执行作用域。
    ///
    /// 公网能力在构造阶段即要求 HTTPS，避免将错误配置延迟到请求执行时发现。
    pub fn new(
        origin: NetworkOrigin,
        address_policy: PluginNetworkAddressPolicy,
    ) -> Result<Self, PluginNetworkError> {
        if matches!(address_policy, PluginNetworkAddressPolicy::PublicOnly)
            && origin.scheme() != "https"
        {
            return Err(PluginNetworkError::SchemeNotAllowed);
        }
        Ok(Self { origin, address_policy })
    }

    pub fn origin(&self) -> &NetworkOrigin {
        &self.origin
    }

    pub fn address_policy(&self) -> &PluginNetworkAddressPolicy {
        &self.address_policy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginHttpMethod {
    Get,
    Head,
}

const MAX_REQUEST_HEADERS: usize = 32;
const MAX_HEADER_NAME_BYTES: usize = 64;
const MAX_HEADER_VALUE_BYTES: usize = 4096;
const MAX_HEADER_BYTES: usize = 16 * 1024;

/// 经过名称白名单和资源限制校验的普通请求头。
#[derive(Debug, Clone, Default)]
pub struct PluginRequestHeaders {
    values: HeaderMap,
    bytes: usize,
}

impl PluginRequestHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: &str, value: &str) -> Result<(), PluginNetworkError> {
        let name = parse_header_name(name)?;
        if !is_allowed_regular_header(&name) {
            return Err(PluginNetworkError::ForbiddenHeader(name.to_string()));
        }
        let value = parse_header_value(value)?;
        insert_bounded_header(&mut self.values, &mut self.bytes, name, value)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// 仅由宿主凭据注入层构造的敏感请求头。
///
/// `Debug` 输出不会包含凭据名称或内容。
#[derive(Clone, Default)]
pub struct PluginNetworkCredentials {
    values: HeaderMap,
    bytes: usize,
}

impl fmt::Debug for PluginNetworkCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PluginNetworkCredentials")
            .field("header_count", &self.values.len())
            .finish_non_exhaustive()
    }
}

impl PluginNetworkCredentials {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: &str, value: &str) -> Result<(), PluginNetworkError> {
        let name = parse_header_name(name)?;
        if !is_allowed_credential_header(&name) {
            return Err(PluginNetworkError::ForbiddenHeader(name.to_string()));
        }
        let value = parse_header_value(value)?;
        insert_bounded_header(&mut self.values, &mut self.bytes, name, value)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// 声明式请求。插件无法通过该类型取得底层 HTTP 客户端、HeaderMap 或 socket。
#[derive(Debug, Clone)]
pub struct PluginNetworkRequest {
    method: PluginHttpMethod,
    url: String,
    headers: PluginRequestHeaders,
    credentials: PluginNetworkCredentials,
}

impl PluginNetworkRequest {
    pub fn new(method: PluginHttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: PluginRequestHeaders::new(),
            credentials: PluginNetworkCredentials::new(),
        }
    }

    pub fn with_headers(mut self, headers: PluginRequestHeaders) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_credentials(mut self, credentials: PluginNetworkCredentials) -> Self {
        self.credentials = credentials;
        self
    }

    pub fn method(&self) -> PluginHttpMethod {
        self.method
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PluginNetworkLimits {
    pub total_timeout: Duration,
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub max_response_bytes: usize,
    pub max_redirects: u8,
}

impl Default for PluginNetworkLimits {
    fn default() -> Self {
        Self {
            total_timeout: Duration::from_secs(15),
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(10),
            max_response_bytes: 1024 * 1024,
            max_redirects: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNetworkTarget {
    pub host: String,
    pub port: u16,
    pub addresses: Vec<IpAddr>,
}

/// 不包含 URL 查询串、凭据、请求头和响应体的执行轨迹。
#[derive(Debug, Clone)]
pub struct PluginNetworkTrace {
    pub duration: Duration,
    pub redirects_followed: u8,
    pub resolved_targets: Vec<ResolvedNetworkTarget>,
    pub bytes_received: usize,
}

#[derive(Debug, Clone)]
pub struct PluginNetworkResponse {
    pub status: u16,
    pub final_origin: NetworkOrigin,
    pub content_type: Option<String>,
    pub body: Bytes,
}

#[derive(Debug, Clone)]
pub struct PluginNetworkExecution {
    pub response: PluginNetworkResponse,
    pub trace: PluginNetworkTrace,
}

/// 传输失败发生的阶段，不包含 URL 或请求数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginTransportStage {
    ClientBuild,
    RequestSend,
    ResponseBody,
}

/// 从 `reqwest::Error` 提取的脱敏失败类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginTransportErrorKind {
    Connect,
    Decode,
    Body,
    Request,
    Other,
}

/// 网络执行失败分类；插件策略拒绝理由仍由上层定义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginNetworkError {
    InvalidConfiguration,
    InvalidOrigin,
    InvalidUrl,
    OriginMismatch,
    SchemeNotAllowed,
    CredentialsNotAllowed,
    ForbiddenHeader(String),
    InvalidHeaderName,
    InvalidHeaderValue,
    RequestHeadersTooLarge,
    AddressNotAllowed(IpAddr),
    EmptyDnsResult,
    TooManyDnsAddresses,
    DnsResolutionFailed(std::io::ErrorKind),
    RedirectMissingLocation,
    RedirectLimitExceeded,
    ResponseTooLarge,
    ConcurrencyLimitExceeded,
    Timeout,
    TransportFailed {
        stage: PluginTransportStage,
        kind: PluginTransportErrorKind,
    },
}

impl fmt::Display for PluginNetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => formatter.write_str("插件网络执行器配置无效"),
            Self::InvalidOrigin => formatter.write_str("插件网络 origin 无效"),
            Self::InvalidUrl => formatter.write_str("插件网络 URL 无效"),
            Self::OriginMismatch => formatter.write_str("插件网络目标超出已批准 origin"),
            Self::SchemeNotAllowed => formatter.write_str("插件网络协议不受允许"),
            Self::CredentialsNotAllowed => formatter.write_str("插件网络 URL 不允许携带凭据"),
            Self::ForbiddenHeader(name) => write!(formatter, "插件网络请求头不受允许: {name}"),
            Self::InvalidHeaderName => formatter.write_str("插件网络请求头名称无效"),
            Self::InvalidHeaderValue => formatter.write_str("插件网络请求头值无效"),
            Self::RequestHeadersTooLarge => formatter.write_str("插件网络请求头超出资源限制"),
            Self::AddressNotAllowed(address) => {
                write!(formatter, "插件网络目标地址不受允许: {address}")
            }
            Self::EmptyDnsResult => formatter.write_str("插件网络 DNS 未返回地址"),
            Self::TooManyDnsAddresses => formatter.write_str("插件网络 DNS 返回地址过多"),
            Self::DnsResolutionFailed(kind) => {
                write!(formatter, "插件网络 DNS 解析失败: {kind:?}")
            }
            Self::RedirectMissingLocation => formatter.write_str("插件网络重定向缺少有效 Location"),
            Self::RedirectLimitExceeded => formatter.write_str("插件网络重定向次数超限"),
            Self::ResponseTooLarge => formatter.write_str("插件网络响应体超限"),
            Self::ConcurrencyLimitExceeded => formatter.write_str("插件网络并发已达上限"),
            Self::Timeout => formatter.write_str("插件网络请求超时"),
            Self::TransportFailed { stage, kind } => {
                write!(formatter, "插件网络传输失败: {stage:?}/{kind:?}")
            }
        }
    }
}

impl std::error::Error for PluginNetworkError {}

fn parse_header_name(value: &str) -> Result<HeaderName, PluginNetworkError> {
    if value.is_empty() || value.len() > MAX_HEADER_NAME_BYTES {
        return Err(PluginNetworkError::InvalidHeaderName);
    }
    HeaderName::from_bytes(value.as_bytes()).map_err(|_| PluginNetworkError::InvalidHeaderName)
}

fn parse_header_value(value: &str) -> Result<HeaderValue, PluginNetworkError> {
    if value.len() > MAX_HEADER_VALUE_BYTES {
        return Err(PluginNetworkError::RequestHeadersTooLarge);
    }
    HeaderValue::from_str(value).map_err(|_| PluginNetworkError::InvalidHeaderValue)
}

fn insert_bounded_header(
    headers: &mut HeaderMap,
    bytes: &mut usize,
    name: HeaderName,
    value: HeaderValue,
) -> Result<(), PluginNetworkError> {
    let old_bytes = headers
        .get(&name)
        .map_or(0, |old| name.as_str().len() + old.as_bytes().len());
    let new_bytes = name.as_str().len() + value.as_bytes().len();
    let next_bytes = bytes
        .checked_sub(old_bytes)
        .and_then(|current| current.checked_add(new_bytes))
        .ok_or(PluginNetworkError::RequestHeadersTooLarge)?;
    let adds_name = !headers.contains_key(&name);
    if (adds_name && headers.len() >= MAX_REQUEST_HEADERS) || next_bytes > MAX_HEADER_BYTES {
        return Err(PluginNetworkError::RequestHeadersTooLarge);
    }
    headers.insert(name, value);
    *bytes = next_bytes;
    Ok(())
}

fn is_allowed_regular_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "accept"
            | "accept-language"
            | "cache-control"
            | "content-type"
            | "if-match"
            | "if-modified-since"
            | "if-none-match"
            | "if-unmodified-since"
            | "pragma"
            | "range"
    )
}

fn is_allowed_credential_header(name: &HeaderName) -> bool {
    matches!(name.as_str(), "authorization" | "cookie" | "x-api-key")
}

fn normalized_url_host(url: &Url) -> Option<String> {
    match url.host()? {
        Host::Domain(host) => Some(host.trim_end_matches('.').to_ascii_lowercase()),
        Host::Ipv4(address) => Some(address.to_string()),
        Host::Ipv6(address) => Some(address.to_string()),
    }
}

fn canonical_ip(address: IpAddr) -> IpAddr {
    match address {
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(address)),
        other => other,
    }
}

#[cfg(test)]
mod tests;
