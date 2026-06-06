use std::collections::BTreeMap;

use crate::models::server::{DockerCommandMode, PublishedPort, RconConfig};
use crate::services::global::i18n_service;

use super::cli_env::{configured_docker_rcon_host_override, is_headless_http_environment};
use super::server_ports::{is_port_available_for_binding, PortBindingKind};
use super::server_shared::{trace_cli_action, trace_cli_error};
use std::collections::HashMap;

const DEFAULT_RCON_PORT: u16 = 25575;

fn cli_docker_t(key: &str) -> String {
    i18n_service().t(key)
}

fn cli_docker_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

fn cli_docker_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn default_docker_env() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert("EULA".to_string(), "TRUE".to_string());
    env.insert("GUI".to_string(), "FALSE".to_string());
    env.insert("CONSOLE".to_string(), "TRUE".to_string());
    env
}

pub(super) fn build_docker_command_transport(
    command_mode: &DockerCommandMode,
    container_name: &str,
    reserved_ports: &[u16],
    env: &mut BTreeMap<String, String>,
) -> Result<(Vec<PublishedPort>, Option<RconConfig>), String> {
    match command_mode {
        DockerCommandMode::DockerStdio => {
            env.insert("CREATE_CONSOLE_IN_PIPE".to_string(), "true".to_string());
            Ok((Vec::new(), None))
        }
        DockerCommandMode::Rcon => {
            ensure_rcon_is_not_disabled(env)?;
            let container_port = resolve_rcon_container_port(env)?;
            let password = resolve_rcon_password(container_name, env)?;
            let host_port = resolve_rcon_port(container_port, reserved_ports)?;
            let rcon_host = resolve_rcon_host();
            env.insert("ENABLE_RCON".to_string(), "true".to_string());
            env.insert("RCON_PORT".to_string(), container_port.to_string());
            env.insert("RCON_PASSWORD".to_string(), password.clone());

            trace_cli_action(
                "docker_rcon_transport",
                &format!(
                    "container={} host={} host_port={} container_port={}",
                    container_name, rcon_host, host_port, container_port
                ),
            );

            Ok((
                vec![PublishedPort {
                    host_port,
                    container_port,
                    protocol: "tcp".to_string(),
                }],
                Some(RconConfig {
                    host: rcon_host,
                    port: host_port,
                    password,
                }),
            ))
        }
    }
}

fn ensure_rcon_is_not_disabled(env: &BTreeMap<String, String>) -> Result<(), String> {
    let Some(value) = env_get_case_insensitive(env, "ENABLE_RCON") else {
        return Ok(());
    };

    if matches!(value.trim().to_ascii_lowercase().as_str(), "false" | "0" | "no") {
        return Err(cli_docker_t("cli.server_setup.docker.rcon_disabled_by_env"));
    }

    Ok(())
}

fn resolve_rcon_container_port(env: &BTreeMap<String, String>) -> Result<u16, String> {
    let Some(value) = env_get_case_insensitive(env, "RCON_PORT") else {
        return Ok(DEFAULT_RCON_PORT);
    };

    value
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| cli_docker_t1("cli.server_setup.docker.rcon_port_invalid", value.trim()))
}

fn resolve_rcon_password(
    container_name: &str,
    env: &BTreeMap<String, String>,
) -> Result<String, String> {
    if let Some(password) = env_get_case_insensitive(env, "RCON_PASSWORD") {
        if password.is_empty() {
            return Err(cli_docker_t("cli.server_setup.docker.rcon_password_empty"));
        }
        return Ok(password.to_string());
    }

    if env_get_case_insensitive(env, "RCON_PASSWORD_FILE").is_some() {
        return Err(cli_docker_t("cli.server_setup.docker.rcon_password_file_requires_password"));
    }

    Ok(build_default_rcon_password(container_name))
}

fn env_get_case_insensitive<'a>(env: &'a BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    env.iter()
        .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

pub(super) fn build_default_rcon_password(container_name: &str) -> String {
    let sanitized = sanitize_name_like(container_name);
    format!("sl-rcon-{}", sanitized)
}

pub(super) fn sanitize_name_like(name: &str) -> String {
    let mut value = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            value.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_' | '.') {
            value.push(ch);
        } else {
            value.push('-');
        }
    }
    let trimmed = value.trim_matches('-');
    if trimmed.is_empty() {
        "server".to_string()
    } else {
        trimmed.to_string()
    }
}

fn resolve_rcon_port(requested_port: u16, reserved_ports: &[u16]) -> Result<u16, String> {
    if !reserved_ports.contains(&requested_port) && is_port_available(requested_port) {
        return Ok(requested_port);
    }

    let mut port = requested_port;
    while port < u16::MAX {
        port = port.saturating_add(1);
        if !reserved_ports.contains(&port) && is_port_available(port) {
            println!(
                "{}",
                cli_docker_t2(
                    "cli.server_setup.docker.rcon_port_shifted",
                    requested_port.to_string(),
                    port.to_string(),
                )
            );
            return Ok(port);
        }
    }

    let error =
        cli_docker_t1("cli.server_setup.docker.rcon_port_unavailable", requested_port.to_string());
    trace_cli_error("docker_rcon_port_unavailable", "", &error);
    Err(error)
}

fn is_port_available(port: u16) -> bool {
    is_port_available_for_binding(port, PortBindingKind::MinecraftGame)
}

fn resolve_rcon_host() -> String {
    resolve_rcon_host_with(
        configured_docker_rcon_host_override().as_deref(),
        is_headless_http_environment(),
    )
}

fn resolve_rcon_host_with(explicit_host: Option<&str>, is_container_like: bool) -> String {
    if let Some(host) = explicit_host.map(str::trim).filter(|host| !host.is_empty()) {
        return host.to_string();
    }

    if is_container_like {
        "host.docker.internal".to_string()
    } else {
        "127.0.0.1".to_string()
    }
}
#[cfg(test)]
mod tests {
    use super::{
        build_default_rcon_password, build_docker_command_transport, resolve_rcon_host_with,
        resolve_rcon_port,
    };
    use crate::models::server::DockerCommandMode;
    use std::collections::BTreeMap;

    #[test]
    fn docker_rcon_transport_builds_default_port_and_config() {
        let mut env = BTreeMap::new();

        let (ports, rcon) = build_docker_command_transport(
            &DockerCommandMode::Rcon,
            "sealantern-paper-test",
            &[],
            &mut env,
        )
        .expect("rcon transport should build");

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].container_port, 25575);
        assert_eq!(env.get("ENABLE_RCON").map(String::as_str), Some("true"));
        assert_eq!(env.get("RCON_PORT").map(String::as_str), Some("25575"));
        assert!(env.contains_key("RCON_PASSWORD"));

        let rcon = rcon.expect("rcon config should exist");
        assert!(!rcon.host.trim().is_empty());
        assert!(rcon.port >= 25575);
        assert_eq!(rcon.password, "sl-rcon-sealantern-paper-test");
    }

    #[test]
    fn docker_rcon_transport_respects_explicit_rcon_env_overrides() {
        let mut env = BTreeMap::from([
            ("RCON_PORT".to_string(), "28016".to_string()),
            ("RCON_PASSWORD".to_string(), "top-secret".to_string()),
        ]);

        let (ports, rcon) = build_docker_command_transport(
            &DockerCommandMode::Rcon,
            "sealantern-paper-test",
            &[],
            &mut env,
        )
        .expect("rcon transport with overrides should build");

        assert_eq!(ports[0].container_port, 28016);
        assert_eq!(env.get("RCON_PORT").map(String::as_str), Some("28016"));
        assert_eq!(env.get("RCON_PASSWORD").map(String::as_str), Some("top-secret"));
        let rcon = rcon.expect("rcon config should exist");
        assert_eq!(rcon.password, "top-secret");
    }

    #[test]
    fn docker_rcon_transport_rejects_disabled_rcon_env() {
        let mut env = BTreeMap::from([("ENABLE_RCON".to_string(), "false".to_string())]);

        let err = build_docker_command_transport(
            &DockerCommandMode::Rcon,
            "sealantern-paper-test",
            &[],
            &mut env,
        )
        .expect_err("disabled rcon env should fail");

        assert!(err.contains("ENABLE_RCON=false"));
    }

    #[test]
    fn docker_rcon_transport_rejects_password_file_without_known_password() {
        let mut env = BTreeMap::from([(
            "RCON_PASSWORD_FILE".to_string(),
            "/run/secrets/rcon_pass".to_string(),
        )]);

        let err = build_docker_command_transport(
            &DockerCommandMode::Rcon,
            "sealantern-paper-test",
            &[],
            &mut env,
        )
        .expect_err("password file without explicit password should fail");

        assert!(err.contains("RCON_PASSWORD_FILE"));
        assert!(err.contains("--command-mode docker_stdio"));
    }

    #[test]
    fn docker_stdio_transport_skips_rcon_side_effects() {
        let mut env = BTreeMap::new();

        let (ports, rcon) = build_docker_command_transport(
            &DockerCommandMode::DockerStdio,
            "sea-test",
            &[],
            &mut env,
        )
        .expect("stdio transport should build");

        assert!(ports.is_empty());
        assert!(rcon.is_none());
        assert_eq!(env.get("CREATE_CONSOLE_IN_PIPE").map(String::as_str), Some("true"));
        assert!(!env.contains_key("ENABLE_RCON"));
    }

    #[test]
    fn default_docker_env_sets_headless_friendly_console_defaults() {
        let env = super::default_docker_env();

        assert_eq!(env.get("EULA").map(String::as_str), Some("TRUE"));
        assert_eq!(env.get("GUI").map(String::as_str), Some("FALSE"));
        assert_eq!(env.get("CONSOLE").map(String::as_str), Some("TRUE"));
    }

    #[test]
    fn default_rcon_password_uses_sanitized_container_name() {
        assert_eq!(build_default_rcon_password("Sea Lantern Docker"), "sl-rcon-sea-lantern-docker");
    }

    #[test]
    fn resolve_rcon_host_prefers_explicit_override() {
        assert_eq!(resolve_rcon_host_with(Some("192.168.1.8"), false), "192.168.1.8");
    }

    #[test]
    fn resolve_rcon_host_uses_host_docker_internal_for_container_like_env() {
        assert_eq!(resolve_rcon_host_with(None, true), "host.docker.internal");
    }

    #[test]
    fn resolve_rcon_host_defaults_to_loopback_on_desktop() {
        assert_eq!(resolve_rcon_host_with(None, false), "127.0.0.1");
    }

    #[test]
    fn resolve_rcon_port_skips_reserved_ports() {
        let resolved = resolve_rcon_port(25575, &[25575, 25576])
            .expect("rcon port should skip reserved values");
        assert!(resolved >= 25577);
    }
}
