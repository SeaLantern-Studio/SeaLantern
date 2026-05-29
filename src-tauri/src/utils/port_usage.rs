use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortUsageKind {
    #[cfg_attr(not(test), allow(dead_code))]
    LoopbackOnly,
    WildcardOrSpecific,
}

pub fn is_tcp_port_listening(port: u16, kind: PortUsageKind) -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_is_tcp_port_listening(port, kind).unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        return unix_is_tcp_port_listening(port, kind).unwrap_or(false);
    }
}

#[cfg(not(target_os = "windows"))]
fn unix_is_tcp_port_listening(port: u16, kind: PortUsageKind) -> Result<bool, String> {
    if let Ok(output) = run_command("ss", &["-ltn"]) {
        if parse_unix_ss_for_tcp_port(&output, port, kind) {
            return Ok(true);
        }
    }

    if let Ok(output) = run_command("netstat", &["-ltn"]) {
        if parse_unix_netstat_for_tcp_port(&output, port, kind) {
            return Ok(true);
        }
    }

    if let Ok(output) = run_command("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"]) {
        if parse_unix_lsof_for_tcp_port(&output, port, kind) {
            return Ok(true);
        }
    }

    Ok(false)
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

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
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
            .find(|token| token.contains("TCP"))
            .or_else(|| trimmed.split_whitespace().find(|token| token.contains(':')))
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
    use super::{split_host_port_impl, PortUsageKind};

    #[cfg(not(target_os = "windows"))]
    use super::{
        parse_unix_lsof_for_tcp_port, parse_unix_netstat_for_tcp_port, parse_unix_ss_for_tcp_port,
    };

    #[cfg(target_os = "windows")]
    use super::parse_windows_netstat_for_tcp_port;

    #[test]
    fn split_host_port_parses_ipv4_and_ipv6() {
        assert_eq!(split_host_port_impl("0.0.0.0:25565"), Some(("0.0.0.0", 25565)));
        assert_eq!(split_host_port_impl("[::]:25565"), Some(("::", 25565)));
        assert_eq!(split_host_port_impl("[::1]:25565"), Some(("::1", 25565)));
    }

    #[test]
    fn parse_windows_netstat_for_tcp_port_detects_listening_port() {
        let sample = "\
  TCP    0.0.0.0:25565          0.0.0.0:0              LISTENING       17088\n\
  TCP    [::]:25565             [::]:0                 LISTENING       17088\n\
  TCP    [::1]:25575            [::]:0                 LISTENING       5132\n";

        assert!(parse_windows_netstat_for_tcp_port(
            sample,
            25565,
            PortUsageKind::WildcardOrSpecific
        ));
        assert!(!parse_windows_netstat_for_tcp_port(sample, 25565, PortUsageKind::LoopbackOnly));
        assert!(parse_windows_netstat_for_tcp_port(sample, 25575, PortUsageKind::LoopbackOnly));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn parse_unix_ss_for_tcp_port_detects_wildcard_and_loopback() {
        let sample = "\
State  Recv-Q Send-Q Local Address:Port  Peer Address:PortProcess\n\
LISTEN 0      4096         0.0.0.0:25565      0.0.0.0:*\n\
LISTEN 0      4096           [::]:25565         [::]:*\n\
LISTEN 0      4096       127.0.0.1:25575      0.0.0.0:*\n";

        assert!(parse_unix_ss_for_tcp_port(sample, 25565, PortUsageKind::WildcardOrSpecific));
        assert!(!parse_unix_ss_for_tcp_port(sample, 25565, PortUsageKind::LoopbackOnly));
        assert!(parse_unix_ss_for_tcp_port(sample, 25575, PortUsageKind::LoopbackOnly));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn parse_unix_netstat_for_tcp_port_detects_listening_port() {
        let sample = "\
Active Internet connections (only servers)\n\
Proto Recv-Q Send-Q Local Address           Foreign Address         State\n\
tcp        0      0 0.0.0.0:25565           0.0.0.0:*               LISTEN\n\
tcp6       0      0 ::1:25575               :::*                    LISTEN\n";

        assert!(parse_unix_netstat_for_tcp_port(
            sample,
            25565,
            PortUsageKind::WildcardOrSpecific
        ));
        assert!(parse_unix_netstat_for_tcp_port(sample, 25575, PortUsageKind::LoopbackOnly));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn parse_unix_lsof_for_tcp_port_detects_listening_port() {
        let sample = "\
COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\n\
java     1234 root   15u  IPv4  12345      0t0  TCP *:25565 (LISTEN)\n\
java     1235 root   16u  IPv6  12346      0t0  TCP localhost:25575 (LISTEN)\n";

        assert!(parse_unix_lsof_for_tcp_port(sample, 25565, PortUsageKind::WildcardOrSpecific));
        assert!(parse_unix_lsof_for_tcp_port(sample, 25575, PortUsageKind::LoopbackOnly));
    }
}
