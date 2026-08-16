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
    let snapshot = crate::net::proxy::read_system_proxy(&PlatformSystemProxyProvider)?;
    let routes = snapshot.routes();
    tracing::debug!(
        target: "sealantern.infra.platform.proxy",
        http_proxy = routes.http_proxy().is_some(),
        https_proxy = routes.https_proxy().is_some(),
        bypass_rules = routes.no_proxy().len(),
        "system proxy snapshot loaded"
    );
    Ok(snapshot)
}

#[cfg(target_os = "windows")]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    let proxy = sysproxy::Sysproxy::get_system_proxy().map_err(|source| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            error = %source,
            "failed to read system proxy via sysproxy"
        );
        SystemProxyReadError::Read { source }
    })?;
    snapshot_from_sysproxy(&proxy)
}

#[cfg(target_os = "linux")]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    let enabled = sysproxy::Sysproxy::get_enable().map_err(|source| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            error = %source,
            "failed to read system proxy enable state"
        );
        SystemProxyReadError::Read { source }
    })?;
    if !enabled {
        return Ok(SystemProxySnapshot::direct());
    }

    let http = sysproxy::Sysproxy::get_http().map_err(|source| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            error = %source,
            "failed to read system HTTP proxy"
        );
        SystemProxyReadError::Read { source }
    })?;
    let https = sysproxy::Sysproxy::get_https().map_err(|source| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            error = %source,
            "failed to read system HTTPS proxy"
        );
        SystemProxyReadError::Read { source }
    })?;
    let bypass = sysproxy::Sysproxy::get_bypass().map_err(|source| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            error = %source,
            "failed to read system proxy bypass rules"
        );
        SystemProxyReadError::Read { source }
    })?;
    let http_proxy = optional_proxy_url(&http.host, http.port)?;
    let https_proxy = optional_proxy_url(&https.host, https.port)?;
    if http_proxy.is_none() && https_proxy.is_none() {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            http_host_empty = http.host.trim().is_empty(),
            https_host_empty = https.host.trim().is_empty(),
            "system proxy is enabled but neither HTTP nor HTTPS endpoint is usable"
        );
        return Err(SystemProxyReadError::UnsupportedProxyKind);
    }

    let (no_proxy, skipped) = convert_bypass_rules(&bypass);
    report_skipped_bypass_rules(skipped);
    Ok(SystemProxySnapshot::from_routes(
        ProxyRoutes::split(http_proxy, https_proxy).with_no_proxy(no_proxy),
    ))
}

#[cfg(target_os = "macos")]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    let settings = sysproxy::Sysproxy::get_proxy_settings().map_err(|source| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            error = %source,
            "failed to read macOS system proxy settings"
        );
        SystemProxyReadError::Read { source }
    })?;
    snapshot_from_macos_settings(&settings)
}

#[cfg(target_os = "macos")]
fn snapshot_from_macos_settings(
    settings: &sysproxy::MacosProxySettings,
) -> Result<SystemProxySnapshot, SystemProxyReadError> {
    if settings.auto_config.enable || settings.auto_discovery {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            pac_enabled = settings.auto_config.enable,
            auto_discovery_enabled = settings.auto_discovery,
            "automatic system proxy configuration cannot be represented safely"
        );
        return Err(SystemProxyReadError::UnsupportedProxyKind);
    }

    let http_proxy = enabled_proxy_url(&settings.http)?;
    let https_proxy = enabled_proxy_url(&settings.https)?;
    if http_proxy.is_none() && https_proxy.is_none() {
        if settings.socks.enable {
            tracing::warn!(
                target: "sealantern.infra.platform.proxy",
                "SOCKS-only system proxy configuration is unsupported"
            );
            return Err(SystemProxyReadError::UnsupportedProxyKind);
        }
        return Ok(SystemProxySnapshot::direct());
    }

    let (no_proxy, skipped) = convert_bypass_rules(&settings.bypass);
    report_skipped_bypass_rules(skipped);
    Ok(SystemProxySnapshot::from_routes(
        ProxyRoutes::split(http_proxy, https_proxy).with_no_proxy(no_proxy),
    ))
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn platform_system_proxy() -> Result<SystemProxySnapshot, SystemProxyReadError> {
    Err(SystemProxyReadError::UnsupportedPlatform)
}

#[cfg(any(target_os = "windows", test))]
fn snapshot_from_sysproxy(
    proxy: &sysproxy::Sysproxy,
) -> Result<SystemProxySnapshot, SystemProxyReadError> {
    tracing::debug!(
        target: "sealantern.infra.platform.proxy",
        enabled = proxy.enable,
        host = diagnostic_proxy_host(&proxy.host),
        port = proxy.port,
        bypass_bytes = proxy.bypass.len(),
        "raw platform proxy settings loaded"
    );
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

/// 记录诊断时对代理主机做脱敏：包含 `@`（可能的用户信息）时整体替换。
fn diagnostic_proxy_host(host: &str) -> &str {
    if host.contains('@') {
        "<redacted>"
    } else {
        host
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn optional_proxy_url(host: &str, port: u16) -> Result<Option<String>, SystemProxyReadError> {
    if host.trim().is_empty() {
        return Ok(None);
    }
    proxy_url(host, port).map(Some)
}

#[cfg(target_os = "macos")]
fn enabled_proxy_url(proxy: &sysproxy::Sysproxy) -> Result<Option<String>, SystemProxyReadError> {
    if !proxy.enable {
        return Ok(None);
    }
    optional_proxy_url(&proxy.host, proxy.port)
}

fn proxy_url(host: &str, port: u16) -> Result<String, SystemProxyReadError> {
    let host = host.trim();
    // 兼容 Windows 上 ProxyServer 携带 scheme 的合法写法（如
    // "http://127.0.0.1"），仅剥离 http / https 前缀，其余原样保留；
    // 剥离后的 userinfo / 非法字符仍由下方白名单校验拦截。
    let host = if host
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    {
        &host[7..]
    } else if host
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        &host[8..]
    } else {
        host
    }
    .trim();
    if host.is_empty() || port == 0 {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            host_empty = host.is_empty(),
            port = port,
            "system proxy endpoint is invalid: empty host or zero port"
        );
        return Err(SystemProxyReadError::InvalidProxy);
    }

    let authority = if host.parse::<Ipv6Addr>().is_ok() {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    let proxy_url = format!("http://{authority}");
    let parsed = url::Url::parse(&proxy_url).map_err(|error| {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            host = diagnostic_proxy_host(host),
            port = port,
            error = %error,
            "system proxy endpoint failed URL parsing"
        );
        SystemProxyReadError::InvalidProxy
    })?;
    let valid = parsed.scheme() == "http"
        && parsed.host().is_some()
        && parsed.port_or_known_default() == Some(port)
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && parsed.path() == "/"
        && parsed.query().is_none()
        && parsed.fragment().is_none();
    if !valid {
        tracing::warn!(
            target: "sealantern.infra.platform.proxy",
            host = diagnostic_proxy_host(host),
            port = port,
            scheme = %parsed.scheme(),
            has_host = parsed.host().is_some(),
            has_userinfo = !parsed.username().is_empty() || parsed.password().is_some(),
            "system proxy endpoint failed validation"
        );
        return Err(SystemProxyReadError::InvalidProxy);
    }
    Ok(proxy_url)
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
        tracing::debug!(
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

    #[cfg(target_os = "macos")]
    fn macos_settings() -> sysproxy::MacosProxySettings {
        sysproxy::MacosProxySettings::default()
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

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_static_proxies_preserve_routes_and_bypass_rules() {
        let mut settings = macos_settings();
        settings.http = enabled_proxy("127.0.0.1", 7890, "");
        settings.https = enabled_proxy("127.0.0.1", 7891, "");
        settings.bypass = "localhost,*.example.com".into();

        let snapshot = snapshot_from_macos_settings(&settings).unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://127.0.0.1:7890"));
        assert_eq!(snapshot.routes().https_proxy(), Some("http://127.0.0.1:7891"));
        assert_eq!(snapshot.routes().no_proxy(), &["localhost", ".example.com"]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_direct_settings_produce_direct_snapshot() {
        let snapshot = snapshot_from_macos_settings(&macos_settings()).unwrap();

        assert_eq!(snapshot, SystemProxySnapshot::direct());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_socks_only_settings_are_rejected() {
        let mut settings = macos_settings();
        settings.socks = enabled_proxy("127.0.0.1", 7892, "");

        let result = snapshot_from_macos_settings(&settings);

        assert!(matches!(result, Err(SystemProxyReadError::UnsupportedProxyKind)));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_automatic_proxy_settings_are_rejected() {
        let mut settings = macos_settings();
        settings.auto_config = sysproxy::Autoproxy {
            enable: true,
            url: "https://example.com/proxy.pac".into(),
        };

        let result = snapshot_from_macos_settings(&settings);

        assert!(matches!(result, Err(SystemProxyReadError::UnsupportedProxyKind)));
    }

    #[test]
    fn ipv6_proxy_host_is_bracketed() {
        let snapshot = snapshot_from_sysproxy(&enabled_proxy("::1", 7890, "")).unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://[::1]:7890"));
    }

    #[test]
    fn default_http_proxy_port_is_valid() {
        let snapshot = snapshot_from_sysproxy(&enabled_proxy("proxy.example.com", 80, "")).unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://proxy.example.com:80"));
    }

    #[test]
    fn diagnostic_host_preserves_normal_hosts_and_redacts_possible_userinfo() {
        assert_eq!(diagnostic_proxy_host("127.0.0.1"), "127.0.0.1");
        assert_eq!(diagnostic_proxy_host("proxy.example.com"), "proxy.example.com");
        assert_eq!(diagnostic_proxy_host("user:password@proxy.example.com"), "<redacted>");
    }

    #[test]
    fn invalid_enabled_proxy_is_rejected() {
        let empty_host = snapshot_from_sysproxy(&enabled_proxy("", 7890, "")).unwrap_err();
        let zero_port = snapshot_from_sysproxy(&enabled_proxy("127.0.0.1", 0, "")).unwrap_err();
        let unsupported_scheme =
            snapshot_from_sysproxy(&enabled_proxy("ftp://example.com", 7890, "")).unwrap_err();

        assert!(matches!(empty_host, SystemProxyReadError::InvalidProxy));
        assert!(matches!(zero_port, SystemProxyReadError::InvalidProxy));
        assert!(matches!(unsupported_scheme, SystemProxyReadError::InvalidProxy));
    }

    #[test]
    fn scheme_prefixed_proxy_host_is_normalized() {
        let snapshot =
            snapshot_from_sysproxy(&enabled_proxy("http://127.0.0.1", 7890, "")).unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://127.0.0.1:7890"));
    }

    #[test]
    fn uppercase_scheme_prefix_is_normalized() {
        let snapshot =
            snapshot_from_sysproxy(&enabled_proxy("HTTP://127.0.0.1", 7890, "")).unwrap();

        assert_eq!(snapshot.routes().http_proxy(), Some("http://127.0.0.1:7890"));
    }

    #[test]
    fn scheme_prefixed_host_with_userinfo_is_still_rejected() {
        let result =
            snapshot_from_sysproxy(&enabled_proxy("http://user:pass@proxy.example.com", 7890, ""));

        assert!(matches!(result, Err(SystemProxyReadError::InvalidProxy)));
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
