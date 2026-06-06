use crate::models::server::{PublishedPort, VolumeMount};
use crate::services::global::i18n_service;
use std::collections::HashMap;

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

pub(crate) fn parse_docker_volume_mounts(values: &[String]) -> Result<Vec<VolumeMount>, String> {
    values
        .iter()
        .map(|value| parse_docker_volume_mount(value))
        .collect()
}

pub(crate) fn parse_docker_volume_mount(value: &str) -> Result<VolumeMount, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(cli_docker_t("cli.server_setup.docker.mount_empty"));
    }

    let (without_mode, read_only) = if let Some(prefix) = trimmed.strip_suffix(":ro") {
        (prefix, true)
    } else if let Some(prefix) = trimmed.strip_suffix(":rw") {
        (prefix, false)
    } else {
        (trimmed, false)
    };

    let (source, target) = split_mount_source_target(without_mode, value)?;
    if source.trim().is_empty() || target.trim().is_empty() {
        return Err(cli_docker_t1("cli.server_setup.docker.mount_source_target_required", value));
    }

    Ok(VolumeMount {
        source: source.trim().to_string(),
        target: target.trim().to_string(),
        read_only,
    })
}

pub(crate) fn parse_extra_published_ports(
    values: &[String],
    reserved_host_ports: &[u16],
) -> Result<Vec<PublishedPort>, String> {
    let mut ports = Vec::with_capacity(values.len());
    for value in values {
        let port = parse_published_port(value)?;
        if reserved_host_ports.contains(&port.host_port) {
            return Err(cli_docker_t2(
                "cli.server_setup.docker.publish_host_port_conflict",
                port.host_port.to_string(),
                value,
            ));
        }
        if ports
            .iter()
            .any(|existing: &PublishedPort| existing.host_port == port.host_port)
        {
            return Err(cli_docker_t2(
                "cli.server_setup.docker.publish_host_port_duplicate",
                port.host_port.to_string(),
                value,
            ));
        }
        ports.push(port);
    }
    Ok(ports)
}

pub(crate) fn parse_published_port(value: &str) -> Result<PublishedPort, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(cli_docker_t("cli.server_setup.docker.publish_empty"));
    }

    let (port_pair, protocol) = if let Some((pair, protocol)) = trimmed.rsplit_once('/') {
        let protocol = protocol.trim().to_ascii_lowercase();
        if !matches!(protocol.as_str(), "tcp" | "udp") {
            return Err(cli_docker_t1("cli.server_setup.docker.publish_protocol_invalid", value));
        }
        (pair, protocol)
    } else {
        (trimmed, "tcp".to_string())
    };

    let Some((host_port, container_port)) = port_pair.split_once(':') else {
        return Err(cli_docker_t1("cli.server_setup.docker.publish_format_invalid", value));
    };

    Ok(PublishedPort {
        host_port: parse_non_zero_port(host_port.trim(), "--publish host")?,
        container_port: parse_non_zero_port(container_port.trim(), "--publish container")?,
        protocol,
    })
}

fn parse_non_zero_port(value: &str, label: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| {
            cli_docker_t2("cli.server_setup.docker.non_zero_port_required", label, value)
        })
}

fn split_mount_source_target(value: &str, original: &str) -> Result<(String, String), String> {
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    for (position, ch) in chars.iter().rev() {
        if *ch != ':' {
            continue;
        }

        let source = &value[..*position];
        let target = &value[*position + 1..];
        if target.starts_with('/') || target.starts_with("./") || target.starts_with("../") {
            return Ok((source.to_string(), target.to_string()));
        }
    }

    Err(cli_docker_t1("cli.server_setup.docker.mount_format_invalid", original))
}
