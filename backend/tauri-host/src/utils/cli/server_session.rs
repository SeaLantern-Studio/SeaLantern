use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};

use crate::models::server::{LocalTerminalMode, ServerInstance, ServerStatus};
use crate::services::global;
use crate::services::server::terminal_transcript;
use crate::utils::server_status::{status_detail_field, status_detail_indicates_running};

use super::server_control::{
    request_stop_with_feedback, restart_server_with_wait, DEFAULT_RESTART_STOP_TIMEOUT_SECS,
};
use super::server_feedback::render_send_command_failure_hint_lines;
use super::server_manage_logs::read_recent_cli_logs;
use super::server_manage_render::{render_server_inspect_lines, render_server_status_lines};
use super::server_shared::{trace_cli_action, trace_cli_error};

pub(super) fn attach_server_cli(server: &ServerInstance) {
    if should_attach_interactive_terminal(server) {
        attach_server_terminal_cli(server);
        return;
    }

    trace_cli_action("cli_session_enter", &format!("server_id={}", server.id));
    println!("已进入服务器 CLI 会话: {}", server.name);
    println!(
        "输入任意内容发送到服务器控制台；输入 /status 查看状态，/inspect 查看详情，/logs 查看最新日志，/stop 停服，/restart 重启，/exit 退出会话。\n"
    );

    loop {
        print!("server:{}> ", server.name);
        if io::stdout().flush().is_err() {
            break;
        }

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if handle_cli_session_input(server, trimmed) {
            break;
        }
    }
}

fn should_attach_interactive_terminal(server: &ServerInstance) -> bool {
    let status = global::server_manager().get_server_status(&server.id);
    let detail = status.detail_message.as_deref();

    let status_looks_running = matches!(
        status.status,
        ServerStatus::Running | ServerStatus::Starting | ServerStatus::Stopping
    ) || status_detail_indicates_running(detail);

    if !status_looks_running {
        return false;
    }

    if let Some(terminal) = status.terminal.as_ref() {
        return terminal.attach_supported || terminal.interactive_supported;
    }

    if let Some(attach) = status_detail_field(detail, "attach") {
        return attach.eq_ignore_ascii_case("true");
    }

    if let Some(interactive) = status_detail_field(detail, "interactive") {
        return interactive.eq_ignore_ascii_case("true");
    }

    if let Some(backend) = status_detail_field(detail, "terminal_backend") {
        return backend.eq_ignore_ascii_case("pty");
    }

    server
        .local_runtime()
        .is_some_and(|runtime| runtime.terminal_mode == LocalTerminalMode::PtyManaged)
}

fn attach_server_terminal_cli(server: &ServerInstance) {
    trace_cli_action("cli_terminal_session_enter", &format!("server_id={}", server.id));
    println!("已连接 PTY 终端会话: {}", server.name);
    println!("按 Ctrl+] 退出当前 attach；不会停止服务端。\n");

    struct RawModeGuard;

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
        }
    }

    if let Err(error) = enable_raw_mode() {
        eprintln!("启用终端 raw 模式失败: {}", error);
        return;
    }
    let _raw_mode_guard = RawModeGuard;

    let mut cursor = 0_u64;
    let server_id = server.id.clone();

    if let Ok((cols, rows)) = size() {
        let _ = global::server_manager().resize_terminal(&server_id, cols, rows);
    }

    loop {
        match terminal_transcript::read_transcript_checked(&server_id, cursor, Some(128 * 1024)) {
            Ok(chunk) => {
                cursor = chunk.next_cursor;
                if !chunk.data.is_empty() {
                    print!("{}", chunk.data);
                    let _ = io::stdout().flush();
                }
            }
            Err(error) => {
                eprintln!("读取终端 transcript 失败: {}", error);
                break;
            }
        }

        match event::poll(Duration::from_millis(120)) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key_event)) => {
                    if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                        continue;
                    }

                    if should_exit_terminal_attach(key_event) {
                        break;
                    }

                    if let Some(input) = key_event_to_terminal_input(key_event) {
                        if let Err(error) =
                            global::server_manager().send_terminal_input(&server_id, &input)
                        {
                            eprintln!("发送终端输入失败: {}", error);
                            break;
                        }
                    }
                }
                Ok(Event::Paste(text)) => {
                    if let Err(error) = global::server_manager().send_terminal_input(&server_id, &text)
                    {
                        eprintln!("发送终端输入失败: {}", error);
                        break;
                    }
                }
                Ok(Event::Resize(cols, rows)) => {
                    let _ = global::server_manager().resize_terminal(&server_id, cols, rows);
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!("读取终端输入事件失败: {}", error);
                    break;
                }
            }
            Ok(false) => {}
            Err(error) => {
                eprintln!("轮询终端输入事件失败: {}", error);
                break;
            }
        }
    }

    println!();
    trace_cli_action("cli_terminal_session_exit", &format!("server_id={}", server.id));
}

fn should_exit_terminal_attach(key_event: KeyEvent) -> bool {
    key_event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key_event.code, KeyCode::Char(']'))
}

fn key_event_to_terminal_input(key_event: KeyEvent) -> Option<String> {
    let mut prefix = String::new();
    if key_event.modifiers.contains(KeyModifiers::ALT) {
        prefix.push('\u{1b}');
    }

    let suffix = match key_event.code {
        KeyCode::Backspace => "\u{7f}".to_string(),
        KeyCode::Enter => "\r".to_string(),
        KeyCode::Left => "\u{1b}[D".to_string(),
        KeyCode::Right => "\u{1b}[C".to_string(),
        KeyCode::Up => "\u{1b}[A".to_string(),
        KeyCode::Down => "\u{1b}[B".to_string(),
        KeyCode::Home => "\u{1b}[H".to_string(),
        KeyCode::End => "\u{1b}[F".to_string(),
        KeyCode::PageUp => "\u{1b}[5~".to_string(),
        KeyCode::PageDown => "\u{1b}[6~".to_string(),
        KeyCode::Tab => "\t".to_string(),
        KeyCode::BackTab => "\u{1b}[Z".to_string(),
        KeyCode::Delete => "\u{1b}[3~".to_string(),
        KeyCode::Insert => "\u{1b}[2~".to_string(),
        KeyCode::Esc => "\u{1b}".to_string(),
        KeyCode::Char(character) => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                let upper = character.to_ascii_uppercase();
                if upper.is_ascii() && (('@'..='_').contains(&upper)) {
                    char::from((upper as u8) & 0x1f_u8).to_string()
                } else if character == ' ' {
                    "\0".to_string()
                } else {
                    return None;
                }
            } else {
                character.to_string()
            }
        }
        _ => return None,
    };

    prefix.push_str(&suffix);
    Some(prefix)
}

fn handle_cli_session_input(server: &ServerInstance, trimmed: &str) -> bool {
    match trimmed {
        "/exit" | "/quit" => {
            trace_cli_action("cli_session_exit", &format!("server_id={}", server.id));
            true
        }
        "/status" => {
            trace_cli_action("cli_status", &format!("server_id={}", server.id));
            let status = global::server_manager().get_server_status(&server.id);
            for line in render_server_status_lines(server, &status) {
                println!("{}", line);
            }
            false
        }
        "/inspect" => {
            trace_cli_action("cli_inspect", &format!("server_id={}", server.id));
            let status = global::server_manager().get_server_status(&server.id);
            for line in render_server_inspect_lines(server, &status) {
                println!("{}", line);
            }
            false
        }
        "/stop" => {
            trace_cli_action("cli_stop", &format!("server_id={}", server.id));
            if let Err(err) = request_stop_with_feedback(server, "cli_stop", "已请求停止服务器...")
            {
                trace_cli_error("cli_stop_failed", &format!("server_id={}", server.id), &err);
                eprintln!("停止失败: {}", err);
            }
            false
        }
        "/restart" => {
            trace_cli_action("cli_restart", &format!("server_id={}", server.id));
            if let Err(err) = restart_server_interactive(server) {
                trace_cli_error("cli_restart_failed", &format!("server_id={}", server.id), &err);
                eprintln!("重启失败: {}", err);
            }
            false
        }
        "/logs" => {
            trace_cli_action("cli_logs", &format!("server_id={}", server.id));
            match read_recent_cli_logs(server) {
                Ok(lines) => {
                    for line in lines {
                        println!("{}", line);
                    }
                }
                Err(err) => {
                    trace_cli_error("cli_logs_failed", &format!("server_id={}", server.id), &err);
                    eprintln!("读取日志失败: {}", err);
                }
            }
            false
        }
        other => {
            trace_cli_action(
                "cli_send_command",
                &format!("server_id={} command={}", server.id, other),
            );
            if let Err(err) = global::server_manager().send_command(&server.id, other) {
                trace_cli_error(
                    "cli_send_command_failed",
                    &format!("server_id={} command={}", server.id, other),
                    &err,
                );
                eprintln!("发送命令失败: {}", err);
                for line in render_send_command_failure_hint_lines(server, other, &err) {
                    eprintln!("{}", line);
                }
            }
            false
        }
    }
}

fn restart_server_interactive(server: &ServerInstance) -> Result<(), String> {
    restart_server_with_wait(server, "cli_restart", DEFAULT_RESTART_STOP_TIMEOUT_SECS)
}

#[cfg(test)]
mod tests {
    use super::{handle_cli_session_input, key_event_to_terminal_input, should_exit_terminal_attach};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::models::server::{
        LocalRuntimeConfig, LocalTerminalMode, ServerInstance, ServerRuntimeConfig,
    };

    fn sample_server() -> ServerInstance {
        ServerInstance {
            id: "server-1".to_string(),
            name: "Transport Test".to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/transport-test".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/transport-test/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                terminal_mode: LocalTerminalMode::PipeManaged,
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn handle_cli_session_input_returns_true_for_exit_aliases() {
        let server = sample_server();
        assert!(handle_cli_session_input(&server, "/exit"));
        assert!(handle_cli_session_input(&server, "/quit"));
    }

    #[test]
    fn handle_cli_session_input_returns_false_for_inspect_command() {
        let server = sample_server();
        assert!(!handle_cli_session_input(&server, "/inspect"));
    }

    #[test]
    fn ctrl_right_bracket_exits_terminal_attach() {
        let key_event = KeyEvent::new(KeyCode::Char(']'), KeyModifiers::CONTROL);
        assert!(should_exit_terminal_attach(key_event));
    }

    #[test]
    fn ctrl_c_maps_to_etx_for_terminal_attach() {
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(key_event_to_terminal_input(key_event).as_deref(), Some("\u{3}"));
    }

    #[test]
    fn arrow_keys_map_to_ansi_sequences() {
        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(key_event_to_terminal_input(key_event).as_deref(), Some("\u{1b}[A"));
    }
}
