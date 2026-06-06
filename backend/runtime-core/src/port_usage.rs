use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortUsageKind {
    #[cfg_attr(not(test), allow(dead_code))]
    LoopbackOnly,
    WildcardOrSpecific,
}

pub fn is_tcp_port_listening(port: u16, kind: PortUsageKind) -> bool {
    is_tcp_port_listening_checked(port, kind).unwrap_or(false)
}

pub fn is_tcp_port_listening_checked(port: u16, kind: PortUsageKind) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        windows_is_tcp_port_listening(port, kind)
    }

    #[cfg(not(target_os = "windows"))]
    {
        unix_is_tcp_port_listening(port, kind)
    }
}

#[cfg(not(target_os = "windows"))]
fn unix_is_tcp_port_listening(port: u16, kind: PortUsageKind) -> Result<bool, String> {
    resolve_port_probe_results([
        (
            "ss",
            run_command("ss", &["-ltn"]).map(|output| parse_unix_ss_for_tcp_port(&output, port, kind)),
        ),
        (
            "netstat",
            run_command("netstat", &["-ltn"])
                .map(|output| parse_unix_netstat_for_tcp_port(&output, port, kind)),
        ),
        (
            "lsof",
            run_command("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"])
                .map(|output| parse_unix_lsof_for_tcp_port(&output, port, kind)),
        ),
    ])
}

#[cfg(any(test, not(target_os = "windows")))]
fn resolve_port_probe_results<const N: usize>(
    probes: [(&str, Result<bool, String>); N],
) -> Result<bool, String> {
    let mut had_successful_probe = false;
    let mut errors = Vec::new();

    for (name, result) in probes {
        match result {
            Ok(true) => return Ok(true),
            Ok(false) => had_successful_probe = true,
            Err(error) => errors.push(format!("{}: {}", name, error)),
        }
    }

    if had_successful_probe {
        Ok(false)
    } else {
        Err(format!("端口占用探测全部失败: {}", errors.join("; ")))
    }
}

#[cfg(not(target_os = "windows"))]
fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("执行 {} 失败: {}", program, err))?;

    if !output.status.success() {
        return Err(format!(
            "{} 返回非零退出码: {}",
            program,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "windows")]
fn windows_is_tcp_port_listening(port: u16, kind: PortUsageKind) -> Result<bool, String> {
    let output = Command::new("netstat")
        .args(["-ano", "-p", "tcp"])
        .output()
        .map_err(|err| format!("执行 netstat 失败: {}", err))?;

    if !output.status.success() {
        return Err(format!(
            "netstat 返回非零退出码: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_windows_netstat_for_tcp_port(&stdout, port, kind))
}

#[cfg(any(test, target_os = "windows"))]
fn parse_windows_netstat_for_tcp_port(
    netstat_output: &str,
    port: u16,
    kind: PortUsageKind,
) -> bool {
    netstat_output.lines().any(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("TCP") {
            return false;
        }

        let columns = trimmed.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 4 {
            return false;
        }

        let local = columns[1];
        let state = columns[3];
        if !state.eq_ignore_ascii_case("LISTENING") {
            return false;
        }

        let Some((host, local_port)) = split_host_port(local) else {
            return false;
        };
        if local_port != port {
            return false;
        }

        match kind {
            PortUsageKind::LoopbackOnly => host == "127.0.0.1" || host == "::1",
            PortUsageKind::WildcardOrSpecific => true,
        }
    })
}

#[cfg(any(test, target_os = "windows"))]
fn split_host_port(endpoint: &str) -> Option<(&str, u16)> {
    split_host_port_impl(endpoint)
}

fn split_host_port_impl(endpoint: &str) -> Option<(&str, u16)> {
    if endpoint.is_empty() {
        return None;
    }

    if let Some(stripped) = endpoint.strip_prefix('[') {
        let (host, remainder) = stripped.split_once(']')?;
        let port = remainder.strip_prefix(':')?.parse::<u16>().ok()?;
        return Some((host, port));
    }

    let index = endpoint.rfind(':')?;
    let host = &endpoint[..index];
    let port = endpoint[index + 1..].parse::<u16>().ok()?;
    Some((host, port))
}

#[cfg(not(target_os = "windows"))]
fn parse_unix_ss_for_tcp_port(ss_output: &str, port: u16, kind: PortUsageKind) -> bool {
    ss_output.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("State") || !trimmed.contains("LISTEN") {
            return false;
        }

        let columns = trimmed.split_whitespace().collect::<Vec<_>>();
        let Some(local) = columns.get(3).copied() else {
            return false;
        };

        endpoint_matches(local, port, kind)
    })
}

#[cfg(not(target_os = "windows"))]
fn parse_unix_netstat_for_tcp_port(netstat_output: &str, port: u16, kind: PortUsageKind) -> bool {
    netstat_output.lines().any(|line| {
        let trimmed = line.trim();
        if !(trimmed.starts_with("tcp") || trimmed.starts_with("tcp6")) {
            return false;
        }

        let columns = trimmed.split_whitespace().collect::<Vec<_>>();
        let Some(local) = columns.get(3).copied() else {
            return false;
        };
        let Some(state) = columns.last().copied() else {
            return false;
        };
        if !state.eq_ignore_ascii_case("LISTEN") {
            return false;
        }

        endpoint_matches(local, port, kind)
    })
}

#[cfg(not(target_os = "windows"))]
fn parse_unix_lsof_for_tcp_port(lsof_output: &str, port: u16, kind: PortUsageKind) -> bool {
    lsof_output.lines().skip(1).any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.contains("(LISTEN)") {
            return false;
        }

        trimmed
            .split_whitespace()
            .find(|token| token.contains(':') && !token.eq_ignore_ascii_case("TCP"))
            .is_some_and(|endpoint| endpoint_matches_from_lsof(endpoint, port, kind))
    })
}

#[cfg(not(target_os = "windows"))]
fn endpoint_matches(endpoint: &str, port: u16, kind: PortUsageKind) -> bool {
    let Some((host, endpoint_port)) = split_host_port_impl(endpoint) else {
        return false;
    };
    if endpoint_port != port {
        return false;
    }

    match kind {
        PortUsageKind::LoopbackOnly => host == "127.0.0.1" || host == "::1" || host == "localhost",
        PortUsageKind::WildcardOrSpecific => true,
    }
}

#[cfg(not(target_os = "windows"))]
fn endpoint_matches_from_lsof(token: &str, port: u16, kind: PortUsageKind) -> bool {
    let cleaned = token.trim_start_matches("TCP").trim();
    let endpoint = cleaned.split("->").next().unwrap_or(cleaned);
    endpoint_matches(endpoint, port, kind)
}

#[cfg(test)]
mod tests {
    use super::{resolve_port_probe_results, split_host_port, PortUsageKind};

    #[test]
    fn split_host_port_supports_ipv6_bracket_notation() {
        let parsed = split_host_port("[::1]:25565").expect("ipv6 endpoint should parse");

        assert_eq!(parsed, ("::1", 25565));
    }

    #[test]
    fn resolve_port_probe_results_returns_true_when_any_probe_matches() {
        let resolved = resolve_port_probe_results([
            ("ss", Err("missing".to_string())),
            ("netstat", Ok(true)),
            ("lsof", Ok(false)),
        ])
        .expect("matching probe should short-circuit to true");

        assert!(resolved);
    }

    #[test]
    fn resolve_port_probe_results_returns_false_when_any_probe_succeeds_without_match() {
        let resolved = resolve_port_probe_results([
            ("ss", Err("missing".to_string())),
            ("netstat", Ok(false)),
            ("lsof", Err("blocked".to_string())),
        ])
        .expect("one successful non-match probe should support false result");

        assert!(!resolved);
    }

    #[test]
    fn resolve_port_probe_results_errors_when_all_probes_fail() {
        let error = resolve_port_probe_results([
            ("ss", Err("missing".to_string())),
            ("netstat", Err("missing".to_string())),
            ("lsof", Err("blocked".to_string())),
        ])
        .expect_err("all failed probes should surface an explicit error");

        assert!(error.contains("端口占用探测全部失败"));
        assert!(error.contains("ss: missing"));
        assert!(error.contains("netstat: missing"));
        assert!(error.contains("lsof: blocked"));
    }

    #[test]
    fn loopback_only_kind_remains_constructible_for_tests() {
        let kind = PortUsageKind::LoopbackOnly;
        assert!(matches!(kind, PortUsageKind::LoopbackOnly));
    }
}
