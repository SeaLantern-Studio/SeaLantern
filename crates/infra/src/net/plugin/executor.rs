use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use futures::StreamExt;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderMap, LOCATION};
use reqwest::{Method, StatusCode};
use tokio::sync::Semaphore;
use url::{Host, Url};

use super::{
    AllowedNetworkTarget, MAX_HEADER_BYTES, MAX_REQUEST_HEADERS, PluginHttpMethod,
    PluginNetworkAddressPolicy, PluginNetworkError, PluginNetworkExecution, PluginNetworkLimits,
    PluginNetworkRequest, PluginNetworkResponse, PluginNetworkScope, PluginNetworkTrace,
    PluginTransportErrorKind, PluginTransportStage, ResolvedNetworkTarget, canonical_ip,
};

pub(super) const MAX_DNS_ADDRESSES: usize = 32;

pub(super) trait PluginDnsResolver: Send + Sync {
    fn resolve<'a>(
        &'a self,
        host: &'a str,
    ) -> futures::future::BoxFuture<'a, Result<Vec<IpAddr>, PluginNetworkError>>;
}

struct SystemPluginDnsResolver;

impl PluginDnsResolver for SystemPluginDnsResolver {
    fn resolve<'a>(
        &'a self,
        host: &'a str,
    ) -> futures::future::BoxFuture<'a, Result<Vec<IpAddr>, PluginNetworkError>> {
        Box::pin(async move {
            tokio::net::lookup_host((host, 0))
                .await
                .map_err(|error| PluginNetworkError::DnsResolutionFailed(error.kind()))
                .map(|addresses| {
                    addresses
                        .map(|address| canonical_ip(address.ip()))
                        .collect()
                })
        })
    }
}

/// 插件专用安全客户端。
///
/// 该客户端不会暴露底层 `reqwest::Client` 或请求构建器。每次连接都会先解析并
/// 校验全部地址，再把连接固定到该批地址；重定向的每一跳都会重复此过程。
#[derive(Clone)]
pub struct PluginNetworkClient {
    resolver: Arc<dyn PluginDnsResolver>,
}

impl PluginNetworkClient {
    pub fn new() -> Self {
        Self {
            resolver: Arc::new(SystemPluginDnsResolver),
        }
    }

    /// 使用调用方已经批准的精确作用域和硬限制执行请求。
    pub async fn execute(
        &self,
        request: PluginNetworkRequest,
        scope: PluginNetworkScope,
        limits: PluginNetworkLimits,
    ) -> Result<PluginNetworkExecution, PluginNetworkError> {
        validate_limits(limits)?;
        validate_request_headers(&request, &scope.address_policy)?;
        let started = Instant::now();
        tokio::time::timeout(
            limits.total_timeout,
            self.execute_inner(request, scope, limits, started),
        )
        .await
        .map_err(|_| PluginNetworkError::Timeout)?
    }

    async fn execute_inner(
        &self,
        request: PluginNetworkRequest,
        scope: PluginNetworkScope,
        limits: PluginNetworkLimits,
        started: Instant,
    ) -> Result<PluginNetworkExecution, PluginNetworkError> {
        execute_request(self, request, scope, limits, started).await
    }

    pub(super) async fn resolve_and_validate(
        &self,
        target: &Url,
        policy: &PluginNetworkAddressPolicy,
    ) -> Result<Vec<IpAddr>, PluginNetworkError> {
        let mut addresses = match target.host().ok_or(PluginNetworkError::InvalidUrl)? {
            Host::Ipv4(address) => vec![IpAddr::V4(address)],
            Host::Ipv6(address) => vec![canonical_ip(IpAddr::V6(address))],
            Host::Domain(host) => self.resolver.resolve(host).await?,
        };
        addresses.iter_mut().for_each(|address| {
            *address = canonical_ip(*address);
        });
        addresses.sort_unstable();
        addresses.dedup();
        if addresses.is_empty() {
            return Err(PluginNetworkError::EmptyDnsResult);
        }
        if addresses.len() > MAX_DNS_ADDRESSES {
            return Err(PluginNetworkError::TooManyDnsAddresses);
        }
        for address in &addresses {
            validate_address(*address, policy)?;
        }
        Ok(addresses)
    }

    #[cfg(test)]
    pub(super) fn with_resolver(resolver: Arc<dyn PluginDnsResolver>) -> Self {
        Self { resolver }
    }
}

impl Default for PluginNetworkClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 带进程级插件网络并发上限的受约束执行器。
pub struct PluginNetworkExecutor {
    client: PluginNetworkClient,
    in_flight: Semaphore,
    limits: PluginNetworkLimits,
}

impl PluginNetworkExecutor {
    pub fn new(
        max_in_flight: usize,
        limits: PluginNetworkLimits,
    ) -> Result<Self, PluginNetworkError> {
        if max_in_flight == 0 {
            return Err(PluginNetworkError::InvalidConfiguration);
        }
        validate_limits(limits)?;
        Ok(Self {
            client: PluginNetworkClient::new(),
            in_flight: Semaphore::new(max_in_flight),
            limits,
        })
    }

    /// 使用指定安全客户端构造执行器，便于宿主统一管理客户端实例。
    pub fn with_client(
        client: PluginNetworkClient,
        max_in_flight: usize,
        limits: PluginNetworkLimits,
    ) -> Result<Self, PluginNetworkError> {
        if max_in_flight == 0 {
            return Err(PluginNetworkError::InvalidConfiguration);
        }
        validate_limits(limits)?;
        Ok(Self {
            client,
            in_flight: Semaphore::new(max_in_flight),
            limits,
        })
    }

    pub async fn execute(
        &self,
        request: PluginNetworkRequest,
        scope: PluginNetworkScope,
    ) -> Result<PluginNetworkExecution, PluginNetworkError> {
        let _permit = self
            .in_flight
            .try_acquire()
            .map_err(|_| PluginNetworkError::ConcurrencyLimitExceeded)?;
        self.client.execute(request, scope, self.limits).await
    }
}

async fn execute_request(
    client: &PluginNetworkClient,
    request: PluginNetworkRequest,
    scope: PluginNetworkScope,
    limits: PluginNetworkLimits,
    started: Instant,
) -> Result<PluginNetworkExecution, PluginNetworkError> {
    let mut target = parse_target(&request.url, &scope)?;
    let headers = request_headers(&request);
    let mut redirects = 0u8;
    let mut resolved_targets = Vec::new();

    loop {
        let addresses = client
            .resolve_and_validate(&target, &scope.address_policy)
            .await?;
        let host = target.host_str().ok_or(PluginNetworkError::InvalidUrl)?;
        let port = target
            .port_or_known_default()
            .ok_or(PluginNetworkError::InvalidUrl)?;
        resolved_targets.push(ResolvedNetworkTarget {
            host: host.trim_end_matches('.').to_ascii_lowercase(),
            port,
            addresses: addresses.clone(),
        });

        let client = build_pinned_client(&target, &addresses, &scope.address_policy, limits)?;
        let response = client
            .request(method(request.method), target.clone())
            .headers(headers.clone())
            .send()
            .await
            .map_err(|error| transport_error(PluginTransportStage::RequestSend, error))?;

        if is_redirect(response.status()) {
            if redirects >= limits.max_redirects {
                return Err(PluginNetworkError::RedirectLimitExceeded);
            }
            let location = response
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or(PluginNetworkError::RedirectMissingLocation)?;
            target = target
                .join(location)
                .map_err(|_| PluginNetworkError::RedirectMissingLocation)?;
            validate_target(&target, &scope)?;
            redirects += 1;
            continue;
        }

        if declared_body_too_large(response.headers(), limits.max_response_bytes) {
            return Err(PluginNetworkError::ResponseTooLarge);
        }
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = read_bounded_body(response, limits.max_response_bytes).await?;
        let bytes_received = body.len();

        return Ok(PluginNetworkExecution {
            response: PluginNetworkResponse {
                status,
                final_origin: scope.origin,
                content_type,
                body,
            },
            trace: PluginNetworkTrace {
                duration: started.elapsed(),
                redirects_followed: redirects,
                resolved_targets,
                bytes_received,
            },
        });
    }
}

fn build_pinned_client(
    target: &Url,
    addresses: &[IpAddr],
    policy: &PluginNetworkAddressPolicy,
    limits: PluginNetworkLimits,
) -> Result<reqwest::Client, PluginNetworkError> {
    let host = target.host_str().ok_or(PluginNetworkError::InvalidUrl)?;
    let port = target
        .port_or_known_default()
        .ok_or(PluginNetworkError::InvalidUrl)?;
    let mut builder = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0)
        .connect_timeout(limits.connect_timeout)
        .read_timeout(limits.read_timeout);
    if matches!(policy, PluginNetworkAddressPolicy::PublicOnly) {
        builder = builder.https_only(true);
    }
    if matches!(target.host(), Some(Host::Domain(_))) {
        let pinned = addresses
            .iter()
            .copied()
            .map(|address| SocketAddr::new(address, port))
            .collect::<Vec<_>>();
        builder = builder.resolve_to_addrs(host, &pinned);
    }
    builder
        .build()
        .map_err(|error| transport_error(PluginTransportStage::ClientBuild, error))
}

fn validate_limits(limits: PluginNetworkLimits) -> Result<(), PluginNetworkError> {
    if limits.max_response_bytes == 0
        || limits.total_timeout.is_zero()
        || limits.connect_timeout.is_zero()
        || limits.read_timeout.is_zero()
    {
        Err(PluginNetworkError::InvalidConfiguration)
    } else {
        Ok(())
    }
}

pub(super) fn parse_target(
    value: &str,
    scope: &PluginNetworkScope,
) -> Result<Url, PluginNetworkError> {
    let target = Url::parse(value).map_err(|_| PluginNetworkError::InvalidUrl)?;
    validate_target(&target, scope)?;
    Ok(target)
}

fn validate_target(target: &Url, scope: &PluginNetworkScope) -> Result<(), PluginNetworkError> {
    if !target.username().is_empty() || target.password().is_some() {
        return Err(PluginNetworkError::CredentialsNotAllowed);
    }
    if !scope.origin.matches(target) {
        return Err(PluginNetworkError::OriginMismatch);
    }
    if matches!(scope.address_policy, PluginNetworkAddressPolicy::PublicOnly)
        && target.scheme() != "https"
    {
        return Err(PluginNetworkError::SchemeNotAllowed);
    }
    Ok(())
}

pub(super) fn validate_request_headers(
    request: &PluginNetworkRequest,
    policy: &PluginNetworkAddressPolicy,
) -> Result<(), PluginNetworkError> {
    if matches!(policy, PluginNetworkAddressPolicy::PublicOnly) && !request.credentials.is_empty() {
        return Err(PluginNetworkError::CredentialsNotAllowed);
    }
    let header_count = request.headers.values.len() + request.credentials.values.len();
    let header_bytes = request
        .headers
        .bytes
        .checked_add(request.credentials.bytes)
        .ok_or(PluginNetworkError::RequestHeadersTooLarge)?;
    if header_count > MAX_REQUEST_HEADERS || header_bytes > MAX_HEADER_BYTES {
        return Err(PluginNetworkError::RequestHeadersTooLarge);
    }
    Ok(())
}

fn request_headers(request: &PluginNetworkRequest) -> HeaderMap {
    let mut headers = request.headers.values.clone();
    headers.extend(request.credentials.values.clone());
    headers
}

fn method(method: PluginHttpMethod) -> Method {
    match method {
        PluginHttpMethod::Get => Method::GET,
        PluginHttpMethod::Head => Method::HEAD,
    }
}

pub(super) fn validate_address(
    address: IpAddr,
    policy: &PluginNetworkAddressPolicy,
) -> Result<(), PluginNetworkError> {
    let address = canonical_ip(address);
    let allowed = match policy {
        PluginNetworkAddressPolicy::PublicOnly => is_public_ip(address),
        PluginNetworkAddressPolicy::AuthenticatedOrPrivate { allowed_targets } => {
            !is_always_forbidden(address)
                && allowed_targets
                    .iter()
                    .any(|target: &AllowedNetworkTarget| target.contains(address))
        }
    };
    if allowed {
        Ok(())
    } else {
        Err(PluginNetworkError::AddressNotAllowed(address))
    }
}

pub(super) fn is_public_ip(address: IpAddr) -> bool {
    match canonical_ip(address) {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let address = u32::from(address);
    ![
        (0x0000_0000, 8),
        (0x0a00_0000, 8),
        (0x6440_0000, 10),
        (0x7f00_0000, 8),
        (0xa9fe_0000, 16),
        (0xac10_0000, 12),
        (0xc000_0000, 24),
        (0xc000_0200, 24),
        (0xc0a8_0000, 16),
        (0xc612_0000, 15),
        (0xc633_6400, 24),
        (0xcb00_7100, 24),
        (0xe000_0000, 4),
        (0xf000_0000, 4),
    ]
    .iter()
    .any(|(network, prefix)| ipv4_in_network(address, *network, *prefix))
}

fn ipv4_in_network(address: u32, network: u32, prefix: u8) -> bool {
    let mask = u32::MAX << (32 - prefix);
    address & mask == network & mask
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    let address = u128::from(address);
    // 公网模式保守地只接受全局单播 2000::/3，并排除特殊用途过渡/文档网段。
    ipv6_in_network(address, 0x2000_0000_0000_0000_0000_0000_0000_0000, 3)
        && ![
            (0x2001_0db8_0000_0000_0000_0000_0000_0000, 32),
            (0x2002_0000_0000_0000_0000_0000_0000_0000, 16),
        ]
        .iter()
        .any(|(network, prefix)| ipv6_in_network(address, *network, *prefix))
}

fn ipv6_in_network(address: u128, network: u128, prefix: u8) -> bool {
    let mask = u128::MAX << (128 - prefix);
    address & mask == network & mask
}

fn is_always_forbidden(address: IpAddr) -> bool {
    match canonical_ip(address) {
        IpAddr::V4(address) => {
            address.is_unspecified()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_multicast()
                || address.is_broadcast()
        }
        IpAddr::V6(address) => {
            address.is_unspecified()
                || address.is_loopback()
                || address.is_unicast_link_local()
                || address.is_multicast()
        }
    }
}

pub(super) fn is_redirect(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}

pub(super) fn declared_body_too_large(headers: &HeaderMap, max_bytes: usize) -> bool {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > max_bytes)
}

async fn read_bounded_body(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Bytes, PluginNetworkError> {
    let mut stream = response.bytes_stream();
    let mut body = Vec::with_capacity(max_bytes.min(16 * 1024));
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|error| transport_error(PluginTransportStage::ResponseBody, error))?;
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or(PluginNetworkError::ResponseTooLarge)?;
        if next_len > max_bytes {
            return Err(PluginNetworkError::ResponseTooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(Bytes::from(body))
}

fn transport_error(stage: PluginTransportStage, error: reqwest::Error) -> PluginNetworkError {
    if error.is_timeout() {
        PluginNetworkError::Timeout
    } else {
        let kind = if error.is_connect() {
            PluginTransportErrorKind::Connect
        } else if error.is_decode() {
            PluginTransportErrorKind::Decode
        } else if error.is_body() {
            PluginTransportErrorKind::Body
        } else if error.is_request() {
            PluginTransportErrorKind::Request
        } else {
            PluginTransportErrorKind::Other
        };
        PluginNetworkError::TransportFailed { stage, kind }
    }
}
