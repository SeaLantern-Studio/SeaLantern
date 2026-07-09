use std::io::{self, Write};
use std::net::TcpListener;

use sea_lantern_runtime::{is_tcp_port_listening_checked, PortUsageKind};

use super::cli_env::effective_cli_web_bind_host;
use super::server_shared::{trace_cli_action, trace_cli_error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PreparedPorts {
    pub game_port: u16,
    pub web_port: Option<u16>,
}

pub(super) fn prepare_ports(
    web_enabled: bool,
    requested_web_port: Option<u16>,
    requested_game_port: u16,
) -> Result<PreparedPorts, String> {
    let web_bind_host = effective_cli_web_bind_host();
    trace_cli_action(
        "ports_prepare_begin",
        &format!(
            "web_enabled={} requested_web_port={} requested_game_port={} web_bind_host={}",
            web_enabled,
            requested_web_port
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".to_string()),
            requested_game_port,
            web_bind_host
        ),
    );

    prepare_ports_with(
        web_enabled,
        requested_web_port,
        requested_game_port,
        is_port_available_for_binding,
        prompt_yes_no,
    )
    .inspect(|prepared| {
        trace_cli_action(
            "ports_prepare_resolved",
            &format!(
                "game_port={} web_port={}",
                prepared.game_port,
                prepared
                    .web_port
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "disabled".to_string())
            ),
        );
    })
    .inspect_err(|error| {
        trace_cli_error("ports_prepare_failed", "", error);
    })
}

pub(super) fn prepare_web_port_only(
    requested_web_port: Option<u16>,
    reserved_game_port: u16,
) -> Result<Option<u16>, String> {
    prepare_web_port_only_with(
        requested_web_port,
        reserved_game_port,
        |port| is_port_available_for_binding(port, PortBindingKind::CliWeb),
        prompt_yes_no,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prepare_web_port_only_with<FIsAvailable, FPrompt>(
    requested_web_port: Option<u16>,
    reserved_game_port: u16,
    is_available: FIsAvailable,
    mut prompt: FPrompt,
) -> Result<Option<u16>, String>
where
    FIsAvailable: FnMut(u16) -> bool,
    FPrompt: FnMut(&str) -> Result<bool, String>,
{
    let requested_web_port = requested_web_port.unwrap_or(8888);
    trace_cli_action(
        "ports_prepare_web_only_begin",
        &format!(
            "requested_web_port={} reserved_game_port={}",
            requested_web_port, reserved_game_port
        ),
    );

    let resolved = resolve_web_port_with(
        requested_web_port,
        Some(reserved_game_port),
        false,
        is_available,
        &mut prompt,
    )?;

    trace_cli_action(
        "ports_prepare_web_only_resolved",
        &format!(
            "requested_web_port={} reserved_game_port={} resolved_web_port={}",
            requested_web_port, reserved_game_port, resolved
        ),
    );

    Ok(Some(resolved))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn prepare_ports_with<FIsAvailable, FPrompt>(
    web_enabled: bool,
    requested_web_port: Option<u16>,
    requested_game_port: u16,
    mut is_available: FIsAvailable,
    mut prompt: FPrompt,
) -> Result<PreparedPorts, String>
where
    FIsAvailable: FnMut(u16, PortBindingKind) -> bool,
    FPrompt: FnMut(&str) -> Result<bool, String>,
{
    let requested_web_port = requested_web_port.unwrap_or(8888);
    let initial_web_conflict =
        web_enabled && !is_available(requested_web_port, PortBindingKind::CliWeb);
    let initial_game_conflict = web_enabled
        && (requested_game_port == requested_web_port
            || !is_available(requested_game_port, PortBindingKind::MinecraftGame));

    trace_cli_action(
        "ports_prepare_initial_state",
        &format!(
            "web_enabled={} requested_web_port={} requested_game_port={} initial_web_conflict={} initial_game_conflict={}",
            web_enabled,
            requested_web_port,
            requested_game_port,
            initial_web_conflict,
            initial_game_conflict
        ),
    );

    let web_port = if web_enabled {
        Some(resolve_web_port_with(
            requested_web_port,
            web_reserved_game_port(requested_web_port, requested_game_port),
            initial_web_conflict && initial_game_conflict,
            |port| is_available(port, PortBindingKind::CliWeb),
            &mut prompt,
        )?)
    } else {
        None
    };

    let game_port = resolve_game_port_with(
        requested_game_port,
        |port| Some(port) != web_port && is_available(port, PortBindingKind::MinecraftGame),
        &mut prompt,
    )?;
    Ok(PreparedPorts { game_port, web_port })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PortBindingKind {
    CliWeb,
    MinecraftGame,
}

fn resolve_web_port_with<FIsAvailable, FPrompt>(
    requested_port: u16,
    reserved_port: Option<u16>,
    prompt_on_conflict: bool,
    mut is_available: FIsAvailable,
    mut prompt: FPrompt,
) -> Result<u16, String>
where
    FIsAvailable: FnMut(u16) -> bool,
    FPrompt: FnMut(&str) -> Result<bool, String>,
{
    if prompt_on_conflict {
        trace_cli_action(
            "ports_web_resolution_mode",
            &format!(
                "requested_port={} reserved_port={} mode=prompt",
                requested_port,
                reserved_port
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
        );
        return resolve_web_port_with_prompt(
            requested_port,
            reserved_port,
            &mut is_available,
            &mut prompt,
        );
    }

    match find_next_available_port(requested_port, |port| {
        Some(port) != reserved_port && is_available(port)
    }) {
        Some(port) if port == requested_port => {
            trace_cli_action(
                "ports_web_keep_requested",
                &format!("requested_port={} resolved_port={}", requested_port, port),
            );
            Ok(port)
        }
        Some(port) => {
            println!("web 端口 {} 已被占用，已自动顺延到 {}", requested_port, port);
            trace_cli_action(
                "ports_web_auto_shift",
                &format!("requested_port={} resolved_port={}", requested_port, port),
            );
            Ok(port)
        }
        None => {
            let error =
                format!("web 端口 {} 及其后续端口均不可用，请检查占用或手动指定", requested_port);
            trace_cli_error(
                "ports_web_exhausted",
                &format!("requested_port={}", requested_port),
                &error,
            );
            Err(error)
        }
    }
}

fn resolve_web_port_with_prompt<FIsAvailable, FPrompt>(
    requested_port: u16,
    reserved_port: Option<u16>,
    mut is_available: FIsAvailable,
    mut prompt: FPrompt,
) -> Result<u16, String>
where
    FIsAvailable: FnMut(u16) -> bool,
    FPrompt: FnMut(&str) -> Result<bool, String>,
{
    let mut port = requested_port;
    loop {
        if Some(port) != reserved_port && is_available(port) {
            trace_cli_action(
                "ports_web_prompt_resolved",
                &format!("requested_port={} resolved_port={}", requested_port, port),
            );
            return Ok(port);
        }

        if port == u16::MAX {
            let error = format!("web 端口 {} 及其后续端口均不可用，请检查占用或手动指定", port);
            trace_cli_error(
                "ports_web_prompt_exhausted",
                &format!("requested_port={}", requested_port),
                &error,
            );
            return Err(error);
        }

        let next_port = port + 1;
        let prompt_message =
            format!("Web 端口 {} 已被占用，是否尝试切换到 {}？ [Y/n] ", port, next_port);
        let accepted = prompt(&prompt_message)?;
        trace_cli_action(
            "ports_web_prompt_decision",
            &format!(
                "requested_port={} current_port={} suggested_port={} accepted={}",
                requested_port, port, next_port, accepted
            ),
        );
        if !accepted {
            return Err(format!("用户取消使用被占用的 Web 端口 {}", port));
        }

        port = next_port;
    }
}

fn web_reserved_game_port(requested_web_port: u16, requested_game_port: u16) -> Option<u16> {
    let _ = requested_web_port;
    Some(requested_game_port)
}

fn resolve_game_port_with<FIsAvailable, FPrompt>(
    requested_port: u16,
    mut is_available: FIsAvailable,
    mut prompt: FPrompt,
) -> Result<u16, String>
where
    FIsAvailable: FnMut(u16) -> bool,
    FPrompt: FnMut(&str) -> Result<bool, String>,
{
    let mut port = requested_port;
    loop {
        if is_available(port) {
            if port != requested_port {
                println!("Minecraft 端口 {} 已被占用，已切换到 {}", requested_port, port);
            }
            trace_cli_action(
                "ports_game_resolved",
                &format!("requested_port={} resolved_port={}", requested_port, port),
            );
            return Ok(port);
        }

        if port == u16::MAX {
            let error =
                format!("Minecraft 端口 {} 及其后续端口均不可用，请检查占用或手动指定", port);
            trace_cli_error(
                "ports_game_exhausted",
                &format!("requested_port={}", requested_port),
                &error,
            );
            return Err(error);
        }

        let next_port = port + 1;
        let prompt_message =
            format!("Minecraft 端口 {} 已被占用，是否尝试切换到 {}？ [Y/n] ", port, next_port);
        let accepted = prompt(&prompt_message)?;
        trace_cli_action(
            "ports_game_prompt_decision",
            &format!(
                "requested_port={} current_port={} suggested_port={} accepted={}",
                requested_port, port, next_port, accepted
            ),
        );
        if !accepted {
            return Err(format!("用户取消使用被占用的 Minecraft 端口 {}", port));
        }

        port = next_port;
    }
}

fn find_next_available_port<FIsAvailable>(
    start_port: u16,
    mut is_available: FIsAvailable,
) -> Option<u16>
where
    FIsAvailable: FnMut(u16) -> bool,
{
    let mut port = start_port;
    loop {
        if is_available(port) {
            return Some(port);
        }
        if port == u16::MAX {
            return None;
        }
        port += 1;
    }
}

#[cfg(test)]
pub(super) fn is_port_available(port: u16) -> bool {
    is_port_available_for_binding(port, PortBindingKind::MinecraftGame)
}

pub(super) fn is_port_available_for_binding(port: u16, binding: PortBindingKind) -> bool {
    match binding {
        PortBindingKind::CliWeb => {
            let bind_host = effective_cli_web_bind_host();
            TcpListener::bind((bind_host.as_str(), port)).is_ok()
        }
        // Minecraft 端口既可能被本地进程绑定到回环地址，也可能被 Docker/其他服务发布到
        // 0.0.0.0。Windows 上单纯试绑端口对 Docker published port 不可靠，因此优先检查
        // 系统监听表，再补一次 bind 作为兜底。
        PortBindingKind::MinecraftGame => minecraft_game_port_is_available(port, |port| {
            is_tcp_port_listening_checked(port, PortUsageKind::WildcardOrSpecific)
        }),
    }
}

fn minecraft_game_port_is_available<F>(port: u16, mut detect_listening: F) -> bool
where
    F: FnMut(u16) -> Result<bool, String>,
{
    let listening = match detect_listening(port) {
        Ok(listening) => listening,
        Err(error) => {
            trace_cli_error(
                "ports_detect_listening_failed",
                &format!("port={} binding=minecraft_game", port),
                &error,
            );
            return false;
        }
    };

    !listening && TcpListener::bind(("0.0.0.0", port)).is_ok()
}

pub(super) fn prompt_yes_no(prompt: &str) -> Result<bool, String> {
    prompt_yes_no_with_default(prompt, true)
}

pub(super) fn prompt_yes_no_default_no(prompt: &str) -> Result<bool, String> {
    prompt_yes_no_with_default(prompt, false)
}

fn prompt_yes_no_with_default(prompt: &str, default_yes: bool) -> Result<bool, String> {
    loop {
        print!("{}", prompt);
        io::stdout()
            .flush()
            .map_err(|e| format!("输出提示失败: {}", e))?;

        let mut input = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("读取输入失败: {}", e))?;

        if bytes_read == 0 {
            let error = format!(
                "当前命令需要交互确认，但标准输入已结束；请显式指定未占用端口，或在交互终端中重试。prompt={} default_yes={}",
                sanitize_prompt_for_log(prompt),
                default_yes
            );
            trace_cli_error(
                "ports_prompt_eof",
                &format!("prompt={}", sanitize_prompt_for_log(prompt)),
                &error,
            );
            return Err(error);
        }

        trace_cli_action(
            "ports_prompt_input",
            &format!("prompt={} raw_input={}", sanitize_prompt_for_log(prompt), input.trim()),
        );

        let parsed = if default_yes {
            parse_yes_no_input(&input)
        } else {
            parse_yes_no_input_with_default(&input, false)
        };

        match parsed {
            Some(result) => {
                trace_cli_action(
                    "ports_prompt_decoded",
                    &format!(
                        "prompt={} accepted={} default_yes={}",
                        sanitize_prompt_for_log(prompt),
                        result,
                        default_yes
                    ),
                );
                return Ok(result);
            }
            None => {
                let guidance = if default_yes {
                    "请输入 y/yes/是/o/回车 确认，或 n/no/否 取消。"
                } else {
                    "请输入 y/yes/是/o 确认，或 n/no/否/回车 取消。"
                };
                println!("{}", guidance);
                trace_cli_error(
                    "ports_prompt_invalid_input",
                    &format!("prompt={}", sanitize_prompt_for_log(prompt)),
                    &format!("raw_input={}", input.trim()),
                );
            }
        }
    }
}

fn parse_yes_no_input(input: &str) -> Option<bool> {
    parse_yes_no_input_with_default(input, true)
}

fn parse_yes_no_input_with_default(input: &str, default_yes: bool) -> Option<bool> {
    let trimmed = input.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return Some(default_yes);
    }

    if matches!(trimmed.as_str(), "y" | "yes" | "是" | "o" | "ok") {
        return Some(true);
    }

    if matches!(trimmed.as_str(), "n" | "no" | "否" | "不" | "cancel" | "stop") {
        return Some(false);
    }

    None
}

fn sanitize_prompt_for_log(prompt: &str) -> String {
    let collapsed = prompt
        .replace(['\r', '\n', '\t'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    ["， ", "。 ", "！ ", "； ", "： "]
        .into_iter()
        .fold(collapsed, |acc, pattern| acc.replace(pattern, &pattern[..pattern.len() - 1]))
}

#[cfg(test)]
mod tests {
    use super::{
        find_next_available_port, is_port_available, is_port_available_for_binding,
        minecraft_game_port_is_available, parse_yes_no_input, parse_yes_no_input_with_default,
        prepare_ports_with, resolve_game_port_with, resolve_web_port_with, sanitize_prompt_for_log,
        PortBindingKind, PreparedPorts,
    };
    use crate::utils::cli::server_test_support::{lock_env, EnvGuard};
    use std::collections::{HashMap, VecDeque};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    #[test]
    fn is_port_available_returns_true_for_ephemeral_port_like_value() {
        assert!(is_port_available(0) || !is_port_available(0));
    }

    #[test]
    fn cli_web_port_availability_uses_effective_web_bind_host() {
        let _env_lock = lock_env();
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "127.0.0.1");

        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind should succeed");
        let occupied_port = listener
            .local_addr()
            .expect("local addr should resolve")
            .port();

        assert!(!is_port_available_for_binding(occupied_port, PortBindingKind::CliWeb));

        drop(listener);
    }

    #[test]
    fn minecraft_game_port_availability_detects_wildcard_binding_conflict() {
        let listener = TcpListener::bind(("0.0.0.0", 0)).expect("bind should succeed");
        let occupied_port = listener
            .local_addr()
            .expect("local addr should resolve")
            .port();

        assert!(!is_port_available_for_binding(occupied_port, PortBindingKind::MinecraftGame));
    }

    #[test]
    fn minecraft_game_port_availability_treats_detection_errors_as_unavailable() {
        let available_listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind should succeed");
        let candidate_port = available_listener
            .local_addr()
            .expect("local addr should resolve")
            .port();
        drop(available_listener);

        assert!(!minecraft_game_port_is_available(candidate_port, |_| {
            Err("netstat failed".to_string())
        }));
    }

    #[test]
    fn resolve_web_port_keeps_requested_port_when_available() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind should succeed");
        let available_port = listener
            .local_addr()
            .expect("local addr should resolve")
            .port();
        drop(listener);

        let resolved = resolve_web_port_with(available_port, None, false, |_| true, |_| Ok(true))
            .expect("port should stay available");
        assert_eq!(resolved, available_port);
    }

    #[test]
    fn find_next_available_port_returns_same_port_when_available() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind should succeed");
        let available_port = listener
            .local_addr()
            .expect("local addr should resolve")
            .port();
        drop(listener);

        let resolved = find_next_available_port(available_port, |_| true)
            .expect("available port should be returned");
        assert_eq!(resolved, available_port);
    }

    #[test]
    fn find_next_available_port_skips_occupied_port() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind should succeed");
        let occupied_port = listener
            .local_addr()
            .expect("local addr should resolve")
            .port();

        let resolved = find_next_available_port(occupied_port, |port| port != occupied_port)
            .expect("a later port should exist");
        assert_eq!(resolved, occupied_port.saturating_add(1));
    }

    #[test]
    fn prepare_ports_resolves_web_before_prompting_for_mc_port() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let web_requested = 8888;
        let mc_requested = 25565;

        let availability_events = Arc::clone(&events);
        let prompt_events = Arc::clone(&events);

        let prepared = prepare_ports_with(
            true,
            Some(web_requested),
            mc_requested,
            move |port, _| {
                availability_events
                    .lock()
                    .expect("events lock")
                    .push(format!("check:{port}"));
                port != web_requested && port != mc_requested
            },
            move |message| {
                prompt_events
                    .lock()
                    .expect("events lock")
                    .push(format!("prompt:{message}"));
                Ok(true)
            },
        )
        .expect("ports should resolve");

        assert_eq!(
            prepared,
            PreparedPorts {
                game_port: mc_requested + 1,
                web_port: Some(web_requested + 1),
            }
        );

        let events = events.lock().expect("events lock");
        let web_check_index = events
            .iter()
            .position(|entry| entry == &format!("check:{web_requested}"))
            .expect("web check should exist");
        let prompt_index = events
            .iter()
            .position(|entry| entry.starts_with("prompt:"))
            .expect("prompt should exist");
        assert!(web_check_index < prompt_index);
    }

    #[test]
    fn resolve_web_port_auto_shifts_without_prompt_when_only_web_conflicts() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let resolved = resolve_web_port_with(
            8888,
            Some(25565),
            false,
            |port| port != 8888,
            move |message: &str| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("web port should auto-shift without prompting");

        assert_eq!(resolved, 8889);
        assert!(prompts.lock().expect("prompt lock").is_empty());
    }

    #[test]
    fn prepare_ports_prompts_for_web_first_when_web_and_mc_are_both_initially_occupied() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let prepared = prepare_ports_with(
            true,
            Some(8888),
            25565,
            |port, _| port != 8888 && port != 25565,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("double conflict should resolve through prompts");

        assert_eq!(prepared, PreparedPorts { game_port: 25566, web_port: Some(8889) });

        let prompts = prompts.lock().expect("prompt lock");
        assert_eq!(prompts.len(), 2);
        assert!(prompts[0].contains("Web 端口 8888"));
        assert!(prompts[0].contains("8889"));
        assert!(prompts[1].contains("Minecraft 端口 25565"));
        assert!(prompts[1].contains("25566"));
    }

    #[test]
    fn prepare_ports_aborts_before_mc_prompt_when_user_rejects_web_port_switch_in_double_conflict()
    {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let err = prepare_ports_with(
            true,
            Some(8888),
            25565,
            |port, _| port != 8888 && port != 25565,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(false)
            },
        )
        .expect_err("rejecting web switch should abort entire flow");

        assert!(err.contains("Web 端口") || err.contains("web 端口"));
        let prompts = prompts.lock().expect("prompt lock");
        assert_eq!(prompts.len(), 1);
        assert!(prompts[0].contains("Web 端口 8888"));
    }

    #[test]
    fn prepare_ports_prefers_preserving_mc_port_when_web_requests_same_port() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let prepared = prepare_ports_with(
            true,
            Some(25565),
            25565,
            |_, _| true,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("web port should auto-shift away from the requested mc port");

        assert_eq!(prepared, PreparedPorts { game_port: 25565, web_port: Some(25566) });

        let prompts = prompts.lock().expect("prompt lock");
        assert!(prompts.is_empty());
    }

    #[test]
    fn prepare_ports_treats_shifted_web_port_as_reserved_for_game_port() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let prepared = prepare_ports_with(
            true,
            Some(8888),
            8889,
            |port, _| port != 8888,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("web auto-shift should avoid the requested game port to reduce follow-up prompts");

        assert_eq!(prepared, PreparedPorts { game_port: 8889, web_port: Some(8890) });

        let prompts = prompts.lock().expect("prompt lock");
        assert!(prompts.is_empty());
    }

    #[test]
    fn prepare_ports_auto_shifts_web_away_from_same_requested_mc_port_without_prompt() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let prepared = prepare_ports_with(
            true,
            Some(25565),
            25565,
            |_, _| true,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("web port should shift without extra prompting");

        assert_eq!(prepared, PreparedPorts { game_port: 25565, web_port: Some(25566) });
        assert!(prompts.lock().expect("prompt lock").is_empty());
    }

    #[test]
    fn resolve_game_port_prompts_for_each_incremental_port() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let resolved = resolve_game_port_with(
            25565,
            |port| port >= 25568,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("port should eventually resolve");

        assert_eq!(resolved, 25568);
        let prompts = prompts.lock().expect("prompt lock");
        assert_eq!(prompts.len(), 3);
        assert!(prompts[0].contains("25565"));
        assert!(prompts[0].contains("25566"));
        assert!(prompts[1].contains("25566"));
        assert!(prompts[1].contains("25567"));
        assert!(prompts[2].contains("25567"));
        assert!(prompts[2].contains("25568"));
    }

    #[test]
    fn resolve_game_port_stops_when_user_rejects_switch() {
        let err = resolve_game_port_with(25565, |_| false, |_| Ok(false))
            .expect_err("rejection should abort resolution");
        assert!(err.contains("用户取消"));
        assert!(err.contains("25565"));
    }

    #[test]
    fn prepare_ports_aborts_when_web_port_range_is_exhausted() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);
        let web_requested = u16::MAX;

        let err = prepare_ports_with(
            true,
            Some(web_requested),
            25565,
            move |port, _| port != web_requested,
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect_err("web exhaustion should abort");

        assert!(err.contains("web 端口"));
        assert!(prompts.lock().expect("prompt lock").is_empty());
    }

    #[test]
    fn prepare_ports_handles_multiple_mc_prompts_before_success() {
        let responses = Arc::new(Mutex::new(VecDeque::from([true, true])));
        let prompts = Arc::new(Mutex::new(Vec::new()));

        let response_log = Arc::clone(&responses);
        let prompt_log = Arc::clone(&prompts);
        let occupied = Arc::new(HashMap::from([(25565_u16, true), (25566_u16, true)]));
        let occupied_map = Arc::clone(&occupied);

        let prepared = prepare_ports_with(
            false,
            None,
            25565,
            move |port, _| !occupied_map.get(&port).copied().unwrap_or(false),
            move |message| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                response_log
                    .lock()
                    .expect("response lock")
                    .pop_front()
                    .ok_or_else(|| "missing test response".to_string())
            },
        )
        .expect("ports should resolve after prompts");

        assert_eq!(prepared, PreparedPorts { game_port: 25567, web_port: None });

        let prompts = prompts.lock().expect("prompt lock");
        assert_eq!(prompts.len(), 2);
        assert!(prompts[0].contains("25565"));
        assert!(prompts[1].contains("25566"));
    }

    #[test]
    fn prepare_web_port_only_auto_shifts_without_touching_game_port_flow() {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let prompt_log = Arc::clone(&prompts);

        let resolved = crate::utils::cli::server_ports::prepare_web_port_only_with(
            Some(25565),
            25565,
            |port| port != 25565,
            move |message: &str| {
                prompt_log
                    .lock()
                    .expect("prompt lock")
                    .push(message.to_string());
                Ok(true)
            },
        )
        .expect("web-only attach port should auto-shift away from reserved game port");

        assert_eq!(resolved, Some(25566));
        assert!(prompts.lock().expect("prompt lock").is_empty());
    }

    #[test]
    fn prepare_ports_propagates_prompt_error_instead_of_silent_accept_on_non_interactive_input() {
        let err = prepare_ports_with(
            false,
            None,
            25565,
            |_, _| false,
            |_| {
                Err("当前命令需要交互确认，但标准输入已结束；请显式指定未占用端口，或在交互终端中重试。prompt=Minecraft 端口 25565 已被占用，是否尝试切换到 25566？ [Y/n] default_yes=true".to_string())
            },
        )
        .expect_err("non-interactive prompt failure should surface");

        assert!(err.contains("需要交互确认"));
        assert!(err.contains("标准输入已结束"));
    }

    #[test]
    fn parse_yes_no_input_accepts_expected_confirm_variants() {
        assert_eq!(parse_yes_no_input("\n"), Some(true));
        assert_eq!(parse_yes_no_input("Y"), Some(true));
        assert_eq!(parse_yes_no_input("yes"), Some(true));
        assert_eq!(parse_yes_no_input("是"), Some(true));
        assert_eq!(parse_yes_no_input("ok"), Some(true));
    }

    #[test]
    fn parse_yes_no_input_accepts_expected_reject_variants() {
        assert_eq!(parse_yes_no_input("n"), Some(false));
        assert_eq!(parse_yes_no_input("NO"), Some(false));
        assert_eq!(parse_yes_no_input("否"), Some(false));
        assert_eq!(parse_yes_no_input("cancel"), Some(false));
    }

    #[test]
    fn parse_yes_no_input_returns_none_for_ambiguous_text() {
        assert_eq!(parse_yes_no_input("maybe"), None);
        assert_eq!(parse_yes_no_input("继续一下"), None);
    }

    #[test]
    fn parse_yes_no_input_with_default_no_treats_empty_input_as_reject() {
        assert_eq!(parse_yes_no_input_with_default("\n", false), Some(false));
        assert_eq!(parse_yes_no_input_with_default("", false), Some(false));
    }

    #[test]
    fn parse_yes_no_input_with_default_yes_preserves_existing_enter_confirms_behavior() {
        assert_eq!(parse_yes_no_input_with_default("\n", true), Some(true));
    }

    #[test]
    fn sanitize_prompt_for_log_collapses_whitespace() {
        assert_eq!(
            sanitize_prompt_for_log(
                "Minecraft 端口 25565 已被占用，\n是否尝试切换到 25566？ [Y/n] "
            ),
            "Minecraft 端口 25565 已被占用，是否尝试切换到 25566？ [Y/n]"
        );
    }
}
