//! 当前系统代理快照的平台代理实现。
//!
//! 本模块通过 `sysproxy-rs` 执行一次性读取，不修改系统设置，也不启动轮询或
//! 操作系统事件监听。

use std::fmt;
use std::net::Ipv6Addr;

use crate::net::proxy::ProxyRoutes;
use crate::net::{SystemProxyProvider, SystemProxySnapshot};

/// 读取或解析系统代理配置时发生的错误。
#[derive(Debug)]
pub enum SystemProxyReadError {
    /// `sysproxy-rs` 无法读取平台配置。
    Read { source: sysproxy::Error },
    /// 当前编译平台不受 `sysproxy-rs` 支持。
    UnsupportedPlatform,
    /// `sysproxy-rs` 返回的端点无法可靠映射为 HTTP 或 HTTPS 代理。
    UnsupportedProxyKind,
    /// 静态代理端点无效。
    InvalidProxy,
}

impl fmt::Display for SystemProxyReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => write!(formatter, "failed to read system proxy: {source}"),
            Self::UnsupportedPlatform => {
                formatter.write_str("system proxy is unsupported on this platform")
            }
            Self::UnsupportedProxyKind => formatter.write_str("system proxy kind is unsupported"),
            Self::InvalidProxy => formatter.write_str("invalid system proxy endpoint"),
        }
    }
}

impl std::error::Error for SystemProxyReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::UnsupportedPlatform | Self::UnsupportedProxyKind | Self::InvalidProxy => None,
        }
    }
}

/// 当前平台的系统代理提供器。
#[derive(Debug, Clone, Copy, Default)]
pub struct PlatformSystemProxyProvider;

impl SystemProxyProvider for PlatformSystemProxyProvider {
    type Error = SystemProxyReadError;

    fn current_system_proxy(&self) -> Result<SystemProxySnapshot, Self::Error> {
        platform_system_proxy()
    }
}

/// 读取当前平台此刻报告的系统代理快照。
pub fn current_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    crate::net::proxy::read_system_proxy(&PlatformSystemProxyProvider)
}

#[cfg(target_os = "windows")]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    let proxy = sysproxy::Sysproxy::get_system_proxy()
        .map_err(|source| SystemProxyReadError::Read { source })?;
    snapshot_from_sysproxy(&proxy)
}

#[cfg(target_os = "linux")]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    let enabled =
        sysproxy::Sysproxy::get_enable().map_err(|source| SystemProxyReadError::Read { source })?;
    if !enabled {
        return Ok(SystemProxySnapshot::direct());
    }

    let http =
        sysproxy::Sysproxy::get_http().map_err(|source| SystemProxyReadError::Read { source })?;
    let https =
        sysproxy::Sysproxy::get_https().map_err(|source| SystemProxyReadError::Read { source })?;
    let bypass =
        sysproxy::Sysproxy::get_bypass().map_err(|source| SystemProxyReadError::Read { source })?;
    let http_proxy = optional_proxy_url(&http.host, http.port)?;
    let https_proxy = optional_proxy_url(&https.host, https.port)?;
    if http_proxy.is_none() && https_proxy.is_none() {
        return Err(SystemProxyReadError::UnsupportedProxyKind);
    }

    let (no_proxy, skipped) = convert_bypass_rules(&bypass);
    report_skipped_bypass_rules(skipped);
    Ok(SystemProxySnapshot::from_routes(
        ProxyRoutes::split(http_proxy, https_proxy).with_no_proxy(no_proxy),
    ))
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    Err(SystemProxyReadError::UnsupportedPlatform)
}

#[cfg(any(target_os = "windows", test))]
fn snapshot_from_sysproxy(
    proxy: &sysproxy::Sysproxy,
) -> Result<SystemProxySnapshot, SystemProxyReadError> {
    if !proxy.enable {
        return Ok(SystemProxySnapshot::direct());
    }

    let proxy_url = proxy_url(&proxy.host, proxy.port)?;
    let (no_proxy, skipped) = convert_bypass_rules(&proxy.bypass);
    report_skipped_bypass_rules(skipped);

    Ok(SystemProxySnapshot::from_routes(
        ProxyRoutes::all(proxy_url).with_no_proxy(no_proxy),
    ))
}

#[cfg(target_os = "linux")]
fn optional_proxy_url(host: &str, port: u16) -> Result<Option<String>, SystemProxyReadError> {
    if host.trim().is_empty() {
        return Ok(None);
    }
    proxy_url(host, port).map(Some)
}

fn proxy_url(host: &str, port: u16) -> Result<String, SystemProxyReadError> {
    let host = host.trim();
    if host.is_empty() || port == 0 {
        return Err(SystemProxyReadError::InvalidProxy);
    }

    let authority = if host.parse::<Ipv6Addr>().is_ok() {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    let proxy_url = format!("http://{authority}");
    reqwest::Proxy::all(&proxy_url)
        .map(|_| proxy_url)
        .map_err(|_| SystemProxyReadError::InvalidProxy)
}

fn convert_bypass_rules(bypass: &str) -> (Vec<String>, usize) {
    let mut converted = Vec::new();
    let mut skipped = 0;

    for rule in bypass
        .split([',', ';'])
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
    {
        if rule.eq_ignore_ascii_case("<local>") {
            skipped += 1;
        } else if let Some(domain) = rule.strip_prefix("*.") {
            if domain.is_empty() || domain.contains('*') {
                skipped += 1;
            } else {
                converted.push(format!(".{domain}"));
            }
        } else if rule != "*" && rule.contains('*') {
            skipped += 1;
        } else {
            converted.push(rule.to_owned());
        }
    }

    (converted, skipped)
}

fn report_skipped_bypass_rules(skipped: usize) {
    if skipped > 0 {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            skipped_rules = skipped,
            "system proxy bypass rules could not be represented exactly"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_proxy(host: &str, port: u16, bypass: &str) -> sysproxy::Sysproxy {
        sysproxy::Sysproxy {
            enable: true,
            host: host.into(),
            port,
            bypass: bypass.into(),
        }
    }

    #[test]
    fn disabled_system_proxy_is_direct() {
        let snapshot = snapshot_from_sysproxy(&sysproxy::Sysproxy::default()).unwrap();

        assert_eq!(snapshot, SystemProxySnapshot::direct());
    }

    #[test]
    fn enabled_system_proxy_applies_to_http_and_https() {
        let snapshot =
            snapshot_from_sysproxy(&enabled_proxy("127.0.0.1", 7890, "localhost;*.example.com"))
                .unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://127.0.0.1:7890"));
        assert_eq!(snapshot.routes().https_proxy(), Some("http://127.0.0.1:7890"));
        assert_eq!(snapshot.routes().no_proxy(), &["localhost", ".example.com"]);
    }

    #[test]
    fn ipv6_proxy_host_is_bracketed() {
        let snapshot = snapshot_from_sysproxy(&enabled_proxy("::1", 7890, "")).unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://[::1]:7890"));
    }

    #[test]
    fn invalid_enabled_proxy_is_rejected() {
        let empty_host = snapshot_from_sysproxy(&enabled_proxy("", 7890, "")).unwrap_err();
        let zero_port = snapshot_from_sysproxy(&enabled_proxy("127.0.0.1", 0, "")).unwrap_err();

        assert!(matches!(empty_host, SystemProxyReadError::InvalidProxy));
        assert!(matches!(zero_port, SystemProxyReadError::InvalidProxy));
    }

    #[test]
    fn bypass_conversion_is_conservative() {
        let (converted, skipped) =
            convert_bypass_rules("localhost;<local>;*.example.com;10.*;*;127.0.0.1;bad*rule");

        assert_eq!(converted, ["localhost", ".example.com", "*", "127.0.0.1"]);
        assert_eq!(skipped, 3);
    }

    #[test]
    fn bypass_conversion_accepts_platform_separators() {
        let (converted, skipped) = convert_bypass_rules("localhost,.example.com;127.0.0.1");

        assert_eq!(converted, ["localhost", ".example.com", "127.0.0.1"]);
        assert_eq!(skipped, 0);
    }
}
