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
    runtime::resolve_http_bind_host(3000)
}

#[cfg(test)]
mod tests {
    use super::{
        configured_servers_container_root, configured_servers_host_root, configured_web_bind_host,
        effective_cli_web_bind_host, env_var_trimmed, has_docker_host_path_mapping,
        is_container_like_environment, is_headless_http_environment,
    };
    use crate::utils::cli::server_test_support::{lock_env, EnvGuard};

    #[test]
    fn env_var_trimmed_discards_blank_values() {
        let _lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_TEST_TRIMMED", "   ");
        assert!(env_var_trimmed("SEALANTERN_TEST_TRIMMED").is_none());
    }

    #[test]
    fn configured_web_bind_host_returns_trimmed_value() {
        let _lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_WEB_BIND", " 192.168.1.10 ");
        assert_eq!(configured_web_bind_host().as_deref(), Some("192.168.1.10"));
    }

    #[test]
    fn effective_web_bind_preserves_explicit_zero_bind_without_marking_headless() {
        let _lock = lock_env();
        let _headless_guard = EnvGuard::remove("SEALANTERN_HEADLESS_HTTP");
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");

        assert!(!is_headless_http_environment());
        assert_eq!(effective_cli_web_bind_host(), "0.0.0.0");
    }

    #[test]
    fn effective_web_bind_defaults_to_loopback_even_in_headless_environment() {
        let _lock = lock_env();
        let _headless_guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _http_bind = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _bind_guard = EnvGuard::remove("SEALANTERN_WEB_BIND");

        assert!(is_headless_http_environment());
        assert_eq!(effective_cli_web_bind_host(), "127.0.0.1");
    }

    #[test]
    fn docker_host_path_mapping_requires_both_roots() {
        let _lock = lock_env();
        let _host_guard = EnvGuard::remove("SEALANTERN_SERVERS_HOST_ROOT");
        let _container_guard = EnvGuard::remove("SEALANTERN_SERVERS_CONTAINER_ROOT");
        assert!(!has_docker_host_path_mapping());

        {
            let _host_guard =
                EnvGuard::set("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
            assert!(!has_docker_host_path_mapping());
        }

        let _host_guard =
            EnvGuard::set("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        let _container_guard =
            EnvGuard::set("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");
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
        let _lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        assert!(is_headless_http_environment());
        assert_eq!(effective_cli_web_bind_host(), "127.0.0.1");
    }

    #[test]
    fn container_like_environment_detects_container_root_mapping() {
        let _lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");
        assert!(is_container_like_environment());
    }
}
