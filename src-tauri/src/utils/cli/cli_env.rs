use std::path::{Path, PathBuf};

pub(super) fn env_var_trimmed(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn configured_web_bind_host() -> Option<String> {
    env_var_trimmed("SEALANTERN_WEB_BIND")
}

pub(super) fn configured_docker_rcon_host_override() -> Option<String> {
    env_var_trimmed("SEALANTERN_DOCKER_RCON_HOST")
}

pub(super) fn configured_servers_container_root() -> Option<PathBuf> {
    env_var_trimmed("SEALANTERN_SERVERS_CONTAINER_ROOT").map(PathBuf::from)
}

pub(super) fn configured_servers_host_root() -> Option<PathBuf> {
    env_var_trimmed("SEALANTERN_SERVERS_HOST_ROOT").map(PathBuf::from)
}

pub(super) fn has_docker_host_path_mapping() -> bool {
    configured_servers_host_root().is_some() && configured_servers_container_root().is_some()
}

pub(super) fn is_headless_http_environment() -> bool {
    Path::new("/.dockerenv").exists() || std::env::var("SEALANTERN_HEADLESS_HTTP").is_ok()
}

pub(super) fn is_container_like_environment() -> bool {
    is_headless_http_environment() || configured_servers_container_root().is_some()
}

pub(super) fn effective_cli_web_bind_host() -> String {
    configured_web_bind_host().unwrap_or_else(|| {
        if is_headless_http_environment() {
            "0.0.0.0".to_string()
        } else {
            "127.0.0.1".to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        configured_servers_container_root, configured_servers_host_root, configured_web_bind_host,
        effective_cli_web_bind_host, env_var_trimmed, has_docker_host_path_mapping,
        is_container_like_environment, is_headless_http_environment,
    };
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
    fn env_var_trimmed_discards_blank_values() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("SEALANTERN_TEST_TRIMMED", "   ");
        assert!(env_var_trimmed("SEALANTERN_TEST_TRIMMED").is_none());
    }

    #[test]
    fn configured_web_bind_host_returns_trimmed_value() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("SEALANTERN_WEB_BIND", " 192.168.1.10 ");
        assert_eq!(configured_web_bind_host().as_deref(), Some("192.168.1.10"));
    }

    #[test]
    fn effective_web_bind_preserves_explicit_zero_bind_without_marking_headless() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _headless_guard = EnvGuard::remove("SEALANTERN_HEADLESS_HTTP");
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");

        assert!(!is_headless_http_environment());
        assert_eq!(effective_cli_web_bind_host(), "0.0.0.0");
    }

    #[test]
    fn docker_host_path_mapping_requires_both_roots() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _host_guard = EnvGuard::remove("SEALANTERN_SERVERS_HOST_ROOT");
        let _container_guard = EnvGuard::remove("SEALANTERN_SERVERS_CONTAINER_ROOT");
        assert!(!has_docker_host_path_mapping());

        std::env::set_var("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        assert!(!has_docker_host_path_mapping());

        std::env::set_var("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");
        assert!(has_docker_host_path_mapping());
        assert_eq!(
            configured_servers_host_root()
                .expect("host root should exist")
                .to_string_lossy(),
            "E:/srv/sealantern/servers"
        );
        assert_eq!(
            configured_servers_container_root()
                .expect("container root should exist")
                .to_string_lossy(),
            "/app/data/servers"
        );
    }

    #[test]
    fn headless_environment_detects_explicit_flag() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        assert!(is_headless_http_environment());
        assert_eq!(effective_cli_web_bind_host(), "0.0.0.0");
    }

    #[test]
    fn container_like_environment_detects_container_root_mapping() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");
        assert!(is_container_like_environment());
    }
}
