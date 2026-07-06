use std::io::{self, Write};

use crate::models::server::ServerInstance;
use crate::services::global;

use super::server_control::{
    request_stop_with_feedback, restart_server_with_wait, DEFAULT_RESTART_STOP_TIMEOUT_SECS,
};
use super::server_feedback::render_send_command_failure_hint_lines;
use super::server_manage_logs::read_recent_cli_logs;
use super::server_manage_render::{render_server_inspect_lines, render_server_status_lines};
use super::server_shared::{trace_cli_action, trace_cli_error};

pub(super) fn attach_server_cli(server: &ServerInstance) {
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
    use super::handle_cli_session_input;
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};

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
}
