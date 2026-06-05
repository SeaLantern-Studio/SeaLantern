//! HTTP 适配层入口

pub mod command_registry;

pub mod server;

pub use server::run_http_server;

use std::net::{IpAddr, SocketAddr};

pub(crate) fn resolve_http_bind_addr(default_port: u16) -> String {
    env_var_trimmed("SEALANTERN_HTTP_BIND")
        .or_else(|| env_var_trimmed("SEALANTERN_WEB_BIND"))
        .map(|value| normalize_bind_addr(&value, default_port))
        .unwrap_or_else(|| format!("127.0.0.1:{}", default_port))
}

pub(crate) fn resolve_http_bind_host(default_port: u16) -> String {
    let bind_addr = resolve_http_bind_addr(default_port);
    extract_bind_host(&bind_addr)
}

fn env_var_trimmed(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_bind_addr(value: &str, default_port: u16) -> String {
    if let Ok(socket_addr) = value.parse::<SocketAddr>() {
        return socket_addr.to_string();
    }

    if let Ok(ip_addr) = value.parse::<IpAddr>() {
        return SocketAddr::new(ip_addr, default_port).to_string();
    }

    if value.contains(':') {
        value.to_string()
    } else {
        format!("{}:{}", value, default_port)
    }
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
    use super::{extract_bind_host, resolve_http_bind_addr, resolve_http_bind_host};
    use std::sync::Mutex;

    static ENV_LOCK: once_cell::sync::Lazy<Mutex<()>> =
        once_cell::sync::Lazy::new(|| Mutex::new(()));

    struct EnvGuard {
        name: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let original = std::env::var(name).ok();
            std::env::set_var(name, value);
            Self { name, original }
        }

        fn remove(name: &'static str) -> Self {
            let original = std::env::var(name).ok();
            std::env::remove_var(name);
            Self { name, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(original) = &self.original {
                std::env::set_var(self.name, original);
            } else {
                std::env::remove_var(self.name);
            }
        }
    }

    #[test]
    fn http_bind_defaults_to_loopback() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _http = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web = EnvGuard::remove("SEALANTERN_WEB_BIND");

        assert_eq!(resolve_http_bind_addr(3000), "127.0.0.1:3000");
        assert_eq!(resolve_http_bind_host(3000), "127.0.0.1");
    }

    #[test]
    fn http_bind_prefers_explicit_http_bind_over_web_bind() {
        let _lock = ENV_LOCK.lock().expect("env lock");
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
}
