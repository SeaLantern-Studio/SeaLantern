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
use reqwest::header::HeaderMap;
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
    pub origin: NetworkOrigin,
    pub address_policy: PluginNetworkAddressPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginHttpMethod {
    Get,
    Head,
}

/// 声明式请求。插件无法通过该类型取得底层 HTTP 客户端或 socket。
#[derive(Debug, Clone)]
pub struct PluginNetworkRequest {
    pub method: PluginHttpMethod,
    pub url: String,
    pub headers: HeaderMap,
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
    AddressNotAllowed(IpAddr),
    EmptyDnsResult,
    TooManyDnsAddresses,
    DnsResolutionFailed,
    RedirectMissingLocation,
    RedirectLimitExceeded,
    ResponseTooLarge,
    ConcurrencyLimitExceeded,
    Timeout,
    TransportFailed,
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
            Self::AddressNotAllowed(address) => {
                write!(formatter, "插件网络目标地址不受允许: {address}")
            }
            Self::EmptyDnsResult => formatter.write_str("插件网络 DNS 未返回地址"),
            Self::TooManyDnsAddresses => formatter.write_str("插件网络 DNS 返回地址过多"),
            Self::DnsResolutionFailed => formatter.write_str("插件网络 DNS 解析失败"),
            Self::RedirectMissingLocation => formatter.write_str("插件网络重定向缺少有效 Location"),
            Self::RedirectLimitExceeded => formatter.write_str("插件网络重定向次数超限"),
            Self::ResponseTooLarge => formatter.write_str("插件网络响应体超限"),
            Self::ConcurrencyLimitExceeded => formatter.write_str("插件网络并发已达上限"),
            Self::Timeout => formatter.write_str("插件网络请求超时"),
            Self::TransportFailed => formatter.write_str("插件网络传输失败"),
        }
    }
}

impl std::error::Error for PluginNetworkError {}

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
