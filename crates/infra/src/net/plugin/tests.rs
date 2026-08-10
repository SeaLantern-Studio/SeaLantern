use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use futures::future::BoxFuture;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, HOST};
use reqwest::StatusCode;
use url::Url;

use super::executor::{
    declared_body_too_large, is_public_ip, is_redirect, parse_target, validate_address,
    validate_headers, PluginDnsResolver, MAX_DNS_ADDRESSES,
};
use super::{
    AllowedNetworkTarget, NetworkOrigin, PluginNetworkAddressPolicy, PluginNetworkClient,
    PluginNetworkError, PluginNetworkExecutor, PluginNetworkLimits, PluginNetworkScope,
};

struct FixedResolver(Vec<IpAddr>);

struct ErrorResolver;

impl PluginDnsResolver for FixedResolver {
    fn resolve<'a>(
        &'a self,
        _host: &'a str,
    ) -> BoxFuture<'a, Result<Vec<IpAddr>, PluginNetworkError>> {
        Box::pin(async { Ok(self.0.clone()) })
    }
}

impl PluginDnsResolver for ErrorResolver {
    fn resolve<'a>(
        &'a self,
        _host: &'a str,
    ) -> BoxFuture<'a, Result<Vec<IpAddr>, PluginNetworkError>> {
        Box::pin(async {
            Err(PluginNetworkError::DnsResolutionFailed(ErrorKind::ConnectionAborted))
        })
    }
}

fn public_scope(origin: &str) -> PluginNetworkScope {
    PluginNetworkScope::new(
        NetworkOrigin::parse(origin).expect("测试 origin 应有效"),
        PluginNetworkAddressPolicy::PublicOnly,
    )
    .expect("测试作用域应有效")
}

#[test]
fn origin_is_normalized_and_rejects_non_origin_components() {
    let origin = NetworkOrigin::parse("https://EXAMPLE.com:443").expect("origin 应有效");
    assert_eq!(origin.scheme(), "https");
    assert_eq!(origin.host(), "example.com");
    assert_eq!(origin.port(), 443);

    for invalid in [
        "ftp://example.com",
        "https://example.com/path",
        "https://user:secret@example.com",
        "https://example.com?token=secret",
        "https://example.com#fragment",
    ] {
        assert_eq!(
            NetworkOrigin::parse(invalid),
            Err(PluginNetworkError::InvalidOrigin),
            "{invalid}"
        );
    }
}

#[test]
fn target_must_stay_inside_exact_origin() {
    let scope = public_scope("https://api.example.com");
    assert!(parse_target("https://api.example.com/v1?q=ok", &scope).is_ok());
    assert!(parse_target("https://api.example.com:444/v1", &scope).is_err());
    assert!(parse_target("https://other.example.com/v1", &scope).is_err());
    assert!(parse_target("http://api.example.com/v1", &scope).is_err());
    assert!(parse_target("https://user@api.example.com/v1", &scope).is_err());
}

#[test]
fn public_scope_rejects_http_origin_during_construction() {
    assert_eq!(
        PluginNetworkScope::new(
            NetworkOrigin::parse("http://api.example.com").expect("HTTP origin 本身应可解析"),
            PluginNetworkAddressPolicy::PublicOnly,
        ),
        Err(PluginNetworkError::SchemeNotAllowed)
    );
}

#[test]
fn public_policy_rejects_special_use_and_mapped_addresses() {
    for address in [
        "0.1.2.3",
        "10.0.0.1",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.169.254",
        "172.16.0.1",
        "192.0.2.1",
        "192.168.1.1",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",
        "64:ff9b::7f00:1",
        "2001:db8::1",
    ] {
        let address: IpAddr = address.parse().expect("测试 IP 应有效");
        assert!(!is_public_ip(address), "{address}");
    }
    assert!(is_public_ip("8.8.8.8".parse().expect("测试 IP 应有效")));
    assert!(is_public_ip("2606:4700:4700::1111".parse().expect("测试 IP 应有效")));
}

#[test]
fn public_policy_accepts_addresses_adjacent_to_excluded_networks() {
    for address in [
        "9.255.255.255",
        "11.0.0.0",
        "100.63.255.255",
        "100.128.0.0",
        "126.255.255.255",
        "128.0.0.0",
        "169.253.255.255",
        "169.255.0.0",
        "172.15.255.255",
        "172.32.0.0",
        "192.0.1.255",
        "192.0.3.0",
        "198.17.255.255",
        "198.20.0.0",
        "198.51.99.255",
        "198.51.101.0",
        "203.0.112.255",
        "203.0.114.0",
        "2001:db7:ffff:ffff:ffff:ffff:ffff:ffff",
        "2001:db9::",
    ] {
        let address: IpAddr = address.parse().expect("测试 IP 应有效");
        assert!(is_public_ip(address), "{address}");
    }
}

#[test]
fn private_policy_requires_explicit_target_and_keeps_hard_denials() {
    let policy = PluginNetworkAddressPolicy::AuthenticatedOrPrivate {
        allowed_targets: vec![AllowedNetworkTarget::Network(
            "192.168.1.0/24".parse().expect("测试网段应有效"),
        )],
    };
    assert!(validate_address("192.168.1.20".parse().expect("测试 IP 应有效"), &policy).is_ok());
    assert!(validate_address("192.168.2.20".parse().expect("测试 IP 应有效"), &policy).is_err());

    let loopback_policy = PluginNetworkAddressPolicy::AuthenticatedOrPrivate {
        allowed_targets: vec![AllowedNetworkTarget::Network(
            "127.0.0.0/8".parse().expect("测试网段应有效"),
        )],
    };
    assert!(
        validate_address("127.0.0.1".parse().expect("测试 IP 应有效"), &loopback_policy).is_err()
    );
}

#[test]
fn public_requests_reject_credentials_and_transport_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer secret"));
    assert!(validate_headers(&headers, &PluginNetworkAddressPolicy::PublicOnly).is_err());

    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("internal.example"));
    let private =
        PluginNetworkAddressPolicy::AuthenticatedOrPrivate { allowed_targets: Vec::new() };
    assert!(validate_headers(&headers, &private).is_err());
}

#[test]
fn only_http_redirect_statuses_are_followed() {
    for status in [301, 302, 303, 307, 308] {
        assert!(is_redirect(StatusCode::from_u16(status).expect("测试状态码应有效")));
    }
    assert!(!is_redirect(StatusCode::NOT_MODIFIED));
    assert!(!is_redirect(StatusCode::MULTIPLE_CHOICES));
}

#[test]
fn declared_response_size_is_checked_before_streaming() {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static("1025"));
    assert!(declared_body_too_large(&headers, 1024));
    assert!(!declared_body_too_large(&headers, 1025));
}

#[test]
fn executor_rejects_zero_resource_limits() {
    let limits = PluginNetworkLimits {
        max_response_bytes: 0,
        ..PluginNetworkLimits::default()
    };
    assert!(matches!(
        PluginNetworkExecutor::new(1, limits),
        Err(PluginNetworkError::InvalidConfiguration)
    ));
    assert!(matches!(
        PluginNetworkExecutor::new(0, PluginNetworkLimits::default()),
        Err(PluginNetworkError::InvalidConfiguration)
    ));
}

#[test]
fn exact_private_target_accepts_ipv4_mapped_equivalent() {
    let policy = PluginNetworkAddressPolicy::AuthenticatedOrPrivate {
        allowed_targets: vec![AllowedNetworkTarget::Exact(
            "192.168.1.20".parse().expect("测试 IP 应有效"),
        )],
    };
    assert!(
        validate_address("::ffff:192.168.1.20".parse().expect("测试 IP 应有效"), &policy).is_ok()
    );
}

#[tokio::test]
async fn client_rejects_dns_answer_when_any_address_is_private() {
    let client = PluginNetworkClient::with_resolver(Arc::new(FixedResolver(vec![
        "8.8.8.8".parse().expect("测试 IP 应有效"),
        "192.168.1.20".parse().expect("测试 IP 应有效"),
    ])));
    let target = Url::parse("https://api.example.com/data").expect("测试 URL 应有效");

    assert_eq!(
        client
            .resolve_and_validate(&target, &PluginNetworkAddressPolicy::PublicOnly)
            .await,
        Err(PluginNetworkError::AddressNotAllowed(
            "192.168.1.20".parse().expect("测试 IP 应有效")
        ))
    );
}

#[tokio::test]
async fn client_rejects_empty_dns_answer() {
    let client = PluginNetworkClient::with_resolver(Arc::new(FixedResolver(Vec::new())));
    let target = Url::parse("https://api.example.com/data").expect("测试 URL 应有效");

    assert_eq!(
        client
            .resolve_and_validate(&target, &PluginNetworkAddressPolicy::PublicOnly)
            .await,
        Err(PluginNetworkError::EmptyDnsResult)
    );
}

#[tokio::test]
async fn client_rejects_too_many_dns_addresses_before_address_validation() {
    let addresses = (1..=(MAX_DNS_ADDRESSES + 1))
        .map(|suffix| IpAddr::V4(Ipv4Addr::new(9, 9, 0, suffix as u8)))
        .collect();
    let client = PluginNetworkClient::with_resolver(Arc::new(FixedResolver(addresses)));
    let target = Url::parse("https://api.example.com/data").expect("测试 URL 应有效");

    assert_eq!(
        client
            .resolve_and_validate(&target, &PluginNetworkAddressPolicy::PublicOnly)
            .await,
        Err(PluginNetworkError::TooManyDnsAddresses)
    );
}

#[tokio::test]
async fn client_propagates_dns_resolution_error_kind() {
    let client = PluginNetworkClient::with_resolver(Arc::new(ErrorResolver));
    let target = Url::parse("https://api.example.com/data").expect("测试 URL 应有效");

    assert_eq!(
        client
            .resolve_and_validate(&target, &PluginNetworkAddressPolicy::PublicOnly)
            .await,
        Err(PluginNetworkError::DnsResolutionFailed(ErrorKind::ConnectionAborted))
    );
}
