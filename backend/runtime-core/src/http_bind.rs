use std::net::{IpAddr, SocketAddr};

pub fn resolve_http_bind_addr(default_port: u16) -> String {
    resolve_http_bind_addr_checked(default_port)
        .unwrap_or_else(|_| format!("127.0.0.1:{}", default_port))
}

pub fn resolve_http_bind_addr_checked(default_port: u16) -> Result<String, String> {
    env_var_trimmed("SEALANTERN_HTTP_BIND")
        .or_else(|| env_var_trimmed("SEALANTERN_WEB_BIND"))
        .map(|value| normalize_bind_addr_checked(&value, default_port))
        .transpose()
        .map(|value| value.unwrap_or_else(|| format!("127.0.0.1:{}", default_port)))
}

pub fn resolve_http_bind_host(default_port: u16) -> String {
    let bind_addr = resolve_http_bind_addr(default_port);
    extract_bind_host(&bind_addr)
}

fn env_var_trimmed(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_bind_addr_checked(value: &str, default_port: u16) -> Result<String, String> {
    if let Ok(socket_addr) = value.parse::<SocketAddr>() {
        return Ok(socket_addr.to_string());
    }

    if let Ok(ip_addr) = value.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip_addr, default_port).to_string());
    }

    if value.contains(':') {
        validate_host_port_like_bind(value)?;
        Ok(value.to_string())
    } else {
        Ok(format!("{}:{}", value, default_port))
    }
}

fn validate_host_port_like_bind(value: &str) -> Result<(), String> {
    if value.starts_with('[') {
        let Some((_, remainder)) = value.split_once(']') else {
            return Err(format!("HTTP 绑定地址无效 '{}': 缺少 IPv6 结束方括号", value));
        };

        let Some(port) = remainder.strip_prefix(':') else {
            return Err(format!("HTTP 绑定地址无效 '{}': IPv6 地址后缺少端口", value));
        };

        return validate_bind_port(value, port);
    }

    let Some((_, port)) = value.rsplit_once(':') else {
        return Err(format!("HTTP 绑定地址无效 '{}': 缺少端口", value));
    };

    validate_bind_port(value, port)
}

fn validate_bind_port(raw: &str, port: &str) -> Result<(), String> {
    port.parse::<u16>()
        .map(|_| ())
        .map_err(|e| format!("HTTP 绑定地址无效 '{}': 端口无效: {}", raw, e))
}

fn extract_bind_host(bind_addr: &str) -> String {
    if let Ok(socket_addr) = bind_addr.parse::<SocketAddr>() {
        return socket_addr.ip().to_string();
    }

    if let Some(stripped) = bind_addr.strip_prefix('[') {
        if let Some((host, _)) = stripped.split_once(']') {
            return host.to_string();
        }
    }

    bind_addr
        .rsplit_once(':')
        .map(|(host, _)| host.to_string())
        .unwrap_or_else(|| bind_addr.to_string())
}

#[cfg(test)]
mod tests {
    use crate::test_support::{lock_env, EnvGuard};
    use super::{
        extract_bind_host, resolve_http_bind_addr, resolve_http_bind_addr_checked,
        resolve_http_bind_host,
    };

    #[test]
    fn http_bind_defaults_to_loopback() {
        let _lock = lock_env();
        let _http = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web = EnvGuard::remove("SEALANTERN_WEB_BIND");

        assert_eq!(resolve_http_bind_addr(3000), "127.0.0.1:3000");
        assert_eq!(resolve_http_bind_host(3000), "127.0.0.1");
    }

    #[test]
    fn http_bind_prefers_explicit_http_bind_over_web_bind() {
        let _lock = lock_env();
        let _http = EnvGuard::set("SEALANTERN_HTTP_BIND", "0.0.0.0:3000");
        let _web = EnvGuard::set("SEALANTERN_WEB_BIND", "127.0.0.1");

        assert_eq!(resolve_http_bind_addr(3000), "0.0.0.0:3000");
        assert_eq!(resolve_http_bind_host(3000), "0.0.0.0");
    }

    #[test]
    fn http_bind_host_extracts_named_host_and_ipv6() {
        assert_eq!(extract_bind_host("localhost:3000"), "localhost");
        assert_eq!(extract_bind_host("[::1]:3000"), "::1");
    }

    #[test]
    fn http_bind_checked_rejects_invalid_port_in_host_port_form() {
        let _lock = lock_env();
        let _http = EnvGuard::set("SEALANTERN_HTTP_BIND", "localhost:notaport");
        let _web = EnvGuard::remove("SEALANTERN_WEB_BIND");

        let error = resolve_http_bind_addr_checked(3000)
            .expect_err("checked HTTP bind resolution should reject invalid host:port values");

        assert!(error.contains("HTTP 绑定地址无效"), "unexpected error: {}", error);
        assert!(error.contains("localhost:notaport"), "unexpected error: {}", error);
        assert_eq!(resolve_http_bind_addr(3000), "127.0.0.1:3000");
    }

    #[test]
    fn http_bind_checked_rejects_ipv6_without_port() {
        let _lock = lock_env();
        let _http = EnvGuard::set("SEALANTERN_HTTP_BIND", "[::1]");
        let _web = EnvGuard::remove("SEALANTERN_WEB_BIND");

        let error = resolve_http_bind_addr_checked(3000)
            .expect_err("checked HTTP bind resolution should reject bracketed IPv6 without port");

        assert!(error.contains("IPv6 地址后缺少端口"), "unexpected error: {}", error);
        assert_eq!(resolve_http_bind_addr(3000), "127.0.0.1:3000");
    }
}
