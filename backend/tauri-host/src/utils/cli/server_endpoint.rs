use crate::models::server::RconConfig;

use super::cli_env::effective_cli_web_bind_host;

pub(super) fn render_cli_web_browser_url(port: u16, server_id: &str) -> String {
    format!("http://{}:{}/console/{}", cli_web_browser_host(), port, server_id)
}

pub(super) fn render_cli_web_binding_hint(port: u16) -> Option<String> {
    let bind_host = effective_cli_web_bind_host();
    if bind_host == "0.0.0.0" {
        Some(format!(
            "CLI Web 当前绑定 0.0.0.0:{}，容器/远端访问时请替换为实际宿主地址",
            port
        ))
    } else {
        None
    }
}

pub(super) fn render_docker_rcon_operator_hint(rcon: &RconConfig, server_id: &str) -> String {
    format!(
        "命令通道: RCON {}:{}，可执行 sealantern server send {} <command>",
        rcon.host, rcon.port, server_id
    )
}

fn cli_web_browser_host() -> String {
    let bind_host = effective_cli_web_bind_host();
    if bind_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        bind_host
    }
}

#[cfg(test)]
mod tests {
    use super::{
        render_cli_web_binding_hint, render_cli_web_browser_url, render_docker_rcon_operator_hint,
    };
    use crate::models::server::RconConfig;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
            if let Some(value) = &self.original {
                std::env::set_var(self.name, value);
            } else {
                std::env::remove_var(self.name);
            }
        }
    }

    #[test]
    fn browser_url_maps_zero_bind_to_loopback_for_local_clickability() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");

        let url = render_cli_web_browser_url(8000, "docker-1");
        assert_eq!(url, "http://127.0.0.1:8000/console/docker-1");
    }

    #[test]
    fn binding_hint_only_exists_for_all_interface_bind() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");

        let hint = render_cli_web_binding_hint(8000).expect("hint should exist");
        assert!(hint.contains("0.0.0.0:8000"));
    }

    #[test]
    fn browser_url_defaults_to_loopback_without_explicit_bind_override() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _http_guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _http_bind = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web_bind = EnvGuard::remove("SEALANTERN_WEB_BIND");

        let url = render_cli_web_browser_url(8000, "docker-1");
        assert_eq!(url, "http://127.0.0.1:8000/console/docker-1");
        assert!(render_cli_web_binding_hint(8000).is_none());
    }

    #[test]
    fn docker_rcon_operator_hint_includes_send_command_guidance() {
        let hint = render_docker_rcon_operator_hint(
            &RconConfig {
                host: "host.docker.internal".to_string(),
                port: 25575,
                password: "secret".to_string(),
            },
            "docker-1",
        );

        assert!(hint.contains("host.docker.internal:25575"));
        assert!(hint.contains("sealantern server send docker-1 <command>"));
    }
}
