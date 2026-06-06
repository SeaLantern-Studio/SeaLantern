use crate::models::server::{ServerInstance, ServerStatusInfo};
use crate::services::global;

use super::server_args::CliMode;
use super::server_control::{
    restart_server_with_wait, stop_server_with_feedback, DEFAULT_RESTART_STOP_TIMEOUT_SECS,
};
use super::server_feedback::render_send_command_failure_hint_lines;
use super::server_manage_logs::{follow_server_logs, read_recent_server_logs};
use super::server_manage_render::{
    render_server_dedupe_report_lines, render_server_inspect_lines, render_server_list_lines,
    render_server_management_help, render_server_status_lines,
};
use super::server_manage_start::{
    args_are_transport_only, parse_manage_start_options,
    start_existing_server_with_optional_transports, ManageStartOptions,
};
use super::server_ports::prompt_yes_no_default_no;
use super::server_ref::{resolve_server_reference, resolve_server_reference_from_servers};
use super::server_shared::{describe_server_instance, trace_cli_action};

const DEFAULT_LOG_LINES: usize = 20;
const DEFAULT_LOG_FOLLOW_INTERVAL_MS: u64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServerManageCommand {
    Help,
    List,
    Inspect {
        target: String,
    },
    Status {
        target: String,
    },
    Logs {
        target: String,
        lines: usize,
        follow: bool,
        interval_ms: u64,
    },
    Send {
        target: String,
        command: String,
    },
    Start {
        target: String,
        options: ManageStartOptions,
    },
    Stop {
        target: String,
    },
    ForceStop {
        target: String,
    },
    Restart {
        target: String,
    },
    DedupeAudit,
    DedupeApply,
}

pub(super) fn try_handle_server_management_command(args: &[String]) -> Result<bool, String> {
    let command = if let Some(command) = parse_server_manage_command(args)? {
        command
    } else {
        let servers = global::server_manager().get_server_list();
        let Some(command) = parse_implicit_existing_start_command(args, &servers)? else {
            return Ok(false);
        };
        command
    };

    trace_cli_action("manage_invoke", &format!("args={}", args.join(" | ")));
    execute_server_manage_command(command)
}

fn execute_server_manage_command(command: ServerManageCommand) -> Result<bool, String> {
    match command {
        ServerManageCommand::Help => {
            print_server_management_help();
        }
        ServerManageCommand::List => {
            print_server_list();
        }
        ServerManageCommand::Inspect { target } => {
            let server = resolve_server_reference(&target)?;
            trace_cli_action("manage_inspect_target", &format!("server_id={}", server.id));
            let status = global::server_manager().get_server_status(&server.id);
            print_server_inspect(&server, &status);
        }
        ServerManageCommand::Status { target } => {
            let server = resolve_server_reference(&target)?;
            trace_cli_action("manage_status", &format!("server_id={}", server.id));
            let status = global::server_manager().get_server_status(&server.id);
            print_server_status(&server, &status);
        }
        ServerManageCommand::Logs { target, lines, follow, interval_ms } => {
            let server = resolve_server_reference(&target)?;
            print_server_logs(&server, lines, follow, interval_ms)?;
        }
        ServerManageCommand::Send { target, command } => {
            let server = resolve_server_reference(&target)?;
            trace_cli_action(
                "manage_send",
                &format!("server_id={} command={}", server.id, command),
            );
            if let Err(err) = global::server_manager().send_command(&server.id, &command) {
                for line in render_send_command_failure_hint_lines(&server, &command, &err) {
                    eprintln!("{}", line);
                }
                return Err(err);
            }
            println!("已发送命令到服务器 {}: {}", server.name, command);
        }
        ServerManageCommand::Start { target, options } => {
            let server = resolve_server_reference(&target)?;
            trace_cli_action("manage_start", &format!("server_id={}", server.id));
            start_existing_server_with_optional_transports(&server, &options)?;
        }
        ServerManageCommand::Stop { target } => {
            let server = resolve_server_reference(&target)?;
            trace_cli_action("manage_stop", &format!("server_id={}", server.id));
            stop_server_with_feedback(&server, "manage_stop", "正在停止服务器...")?;
        }
        ServerManageCommand::ForceStop { target } => {
            let server = resolve_server_reference(&target)?;
            force_stop_server(&server)?;
        }
        ServerManageCommand::Restart { target } => {
            let server = resolve_server_reference(&target)?;
            restart_server(&server)?;
        }
        ServerManageCommand::DedupeAudit => {
            trace_cli_action("manage_dedupe_audit", "");
            let report = global::server_manager().audit_duplicate_server_records()?;
            for line in render_server_dedupe_report_lines(&report, false) {
                println!("{}", line);
            }
        }
        ServerManageCommand::DedupeApply => {
            trace_cli_action("manage_dedupe_apply", "");
            let report = global::server_manager().dedupe_duplicate_server_records()?;
            for line in render_server_dedupe_report_lines(&report, true) {
                println!("{}", line);
            }
        }
    }

    Ok(true)
}

fn parse_server_manage_command(args: &[String]) -> Result<Option<ServerManageCommand>, String> {
    let Some(first) = args.first() else {
        return Ok(None);
    };

    let subcommand = first.trim().to_ascii_lowercase();
    if subcommand.is_empty() {
        return Ok(None);
    }

    if matches!(subcommand.as_str(), "manage-help" | "subcommands") {
        return Ok(Some(ServerManageCommand::Help));
    }

    if subcommand != "start" && looks_like_server_create_flow(args) {
        return Ok(None);
    }

    if subcommand == "start" && looks_like_reserved_name_create_flow(args) {
        return Ok(None);
    }

    match subcommand.as_str() {
        "list" => ensure_no_extra_args(args, 1).map(|_| Some(ServerManageCommand::List)),
        "inspect" => parse_target_only(args, "inspect")
            .map(|target| Some(ServerManageCommand::Inspect { target })),
        "status" => parse_target_only(args, "status")
            .map(|target| Some(ServerManageCommand::Status { target })),
        "start" => parse_start_command(args).map(Some),
        "stop" => {
            parse_target_only(args, "stop").map(|target| Some(ServerManageCommand::Stop { target }))
        }
        "force-stop" | "forcestop" => parse_target_only(args, "force-stop")
            .map(|target| Some(ServerManageCommand::ForceStop { target })),
        "restart" => parse_target_only(args, "restart")
            .map(|target| Some(ServerManageCommand::Restart { target })),
        "dedupe-audit" => {
            ensure_no_extra_args(args, 1).map(|_| Some(ServerManageCommand::DedupeAudit))
        }
        "dedupe-apply" => {
            ensure_no_extra_args(args, 1).map(|_| Some(ServerManageCommand::DedupeApply))
        }
        "logs" => parse_logs_command(args).map(Some),
        "send" => parse_send_command(args).map(Some),
        _ => Ok(None),
    }
}

fn looks_like_server_create_flow(args: &[String]) -> bool {
    args.iter().skip(1).any(|arg| {
        matches!(
            arg.as_str(),
            "--name"
                | "--n"
                | "--folder"
                | "--f"
                | "--fd"
                | "--runtime"
                | "--r"
                | "--mc"
                | "--mc-version"
                | "--core"
                | "--jar"
                | "--java"
                | "--j"
                | "--J"
                | "--web"
                | "--cli"
                | "--detach"
                | "--create-only"
                | "--no-start"
                | "--image"
                | "--image-tag"
                | "--data-dir"
                | "--container-name"
                | "--docker-backend"
                | "--command-mode"
                | "--entry"
                | "--startup"
                | "--min"
                | "--max"
                | "--tag"
                | "--t"
                | "--alias"
        ) || arg.starts_with("-p")
            || arg.starts_with("--port")
            || arg.starts_with("-web:")
            || arg.starts_with("--web:")
            || arg.starts_with("--web=")
            || arg.starts_with("-min:")
            || arg.starts_with("-max:")
    })
}

fn ensure_no_extra_args(args: &[String], expected_len: usize) -> Result<(), String> {
    if args.len() == expected_len {
        Ok(())
    } else {
        Err(format!("参数过多: {}", args[expected_len..].join(" ")))
    }
}

fn parse_target_only(args: &[String], subcommand: &str) -> Result<String, String> {
    let target = args
        .get(1)
        .cloned()
        .ok_or_else(|| format!("{} 需要服务器 ID / 名称 / 别名", subcommand))?;
    ensure_no_extra_args(args, 2)?;
    Ok(target)
}

fn parse_start_command(args: &[String]) -> Result<ServerManageCommand, String> {
    let target = args
        .get(1)
        .cloned()
        .ok_or_else(|| "start 需要服务器 ID / 名称 / 别名".to_string())?;
    let options = parse_manage_start_options(&args[2..])?;

    Ok(ServerManageCommand::Start { target, options })
}

fn looks_like_reserved_name_create_flow(args: &[String]) -> bool {
    args.get(1).is_some_and(|value| value.starts_with('-')) && looks_like_server_create_flow(args)
}

fn parse_implicit_existing_start_command(
    args: &[String],
    servers: &[ServerInstance],
) -> Result<Option<ServerManageCommand>, String> {
    let Some(target) = args
        .first()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    if target.starts_with('-') || looks_like_reserved_manage_subcommand(target) {
        return Ok(None);
    }

    let resolved = match resolve_server_reference_from_servers(servers, target) {
        Ok(server) => server,
        Err(err) if err.starts_with("未找到服务器:") => return Ok(None),
        Err(err) => return Err(err),
    };

    if args.len() == 1 {
        return Ok(Some(ServerManageCommand::Start {
            target: resolved.id,
            options: ManageStartOptions {
                cli: CliMode::Enabled,
                ..Default::default()
            },
        }));
    }

    if !args_are_transport_only(&args[1..]) {
        return Ok(None);
    }

    let options = parse_manage_start_options(&args[1..])?;
    if !options.has_transport_request() {
        return Ok(None);
    }

    Ok(Some(ServerManageCommand::Start { target: resolved.id, options }))
}

fn looks_like_reserved_manage_subcommand(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "manage-help"
            | "subcommands"
            | "list"
            | "inspect"
            | "status"
            | "start"
            | "stop"
            | "force-stop"
            | "forcestop"
            | "restart"
            | "logs"
            | "send"
            | "dedupe-audit"
            | "dedupe-apply"
    )
}

fn parse_logs_command(args: &[String]) -> Result<ServerManageCommand, String> {
    let target = args
        .get(1)
        .cloned()
        .ok_or_else(|| "logs 需要服务器 ID / 名称 / 别名".to_string())?;
    let mut lines = DEFAULT_LOG_LINES;
    let mut follow = false;
    let mut interval_ms = DEFAULT_LOG_FOLLOW_INTERVAL_MS;
    let mut index = 2;
    while index < args.len() {
        let current = args[index].as_str();
        if let Some(value) = current.strip_prefix("--lines=") {
            lines = value
                .parse::<usize>()
                .map_err(|_| format!("无效的日志行数: {}", value))?;
            index += 1;
            continue;
        }
        if let Some(value) = current.strip_prefix("--interval=") {
            interval_ms = value
                .parse::<u64>()
                .map_err(|_| format!("无效的轮询间隔毫秒数: {}", value))?;
            index += 1;
            continue;
        }

        match current {
            "--lines" | "-n" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("{} 缺少值", args[index]))?;
                lines = value
                    .parse::<usize>()
                    .map_err(|_| format!("无效的日志行数: {}", value))?;
                index += 2;
            }
            "--follow" | "-f" => {
                follow = true;
                index += 1;
            }
            "--interval" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("{} 缺少值", args[index]))?;
                interval_ms = value
                    .parse::<u64>()
                    .map_err(|_| format!("无效的轮询间隔毫秒数: {}", value))?;
                index += 2;
            }
            other => return Err(format!("logs 不支持的参数: {}", other)),
        }
    }
    Ok(ServerManageCommand::Logs { target, lines, follow, interval_ms })
}

fn parse_send_command(args: &[String]) -> Result<ServerManageCommand, String> {
    let target = args
        .get(1)
        .cloned()
        .ok_or_else(|| "send 需要服务器 ID / 名称 / 别名".to_string())?;
    let command = args
        .get(2..)
        .filter(|items| !items.is_empty())
        .map(|items| items.join(" "))
        .ok_or_else(|| "send 需要控制台命令内容".to_string())?;
    Ok(ServerManageCommand::Send { target, command })
}

fn print_server_management_help() {
    println!("{}", render_server_management_help());
}

fn print_server_list() {
    let manager = global::server_manager();
    let servers = manager.get_server_list();
    let snapshots = servers
        .into_iter()
        .map(|server| {
            let status = manager.get_server_status(&server.id);
            (server, status)
        })
        .collect::<Vec<_>>();
    trace_cli_action("manage_list", &format!("count={}", snapshots.len()));
    for line in render_server_list_lines(&snapshots) {
        println!("{}", line);
    }
}

fn print_server_status(server: &ServerInstance, status: &ServerStatusInfo) {
    for line in render_server_status_lines(server, status) {
        println!("{}", line);
    }
}

fn print_server_logs(
    server: &ServerInstance,
    lines: usize,
    follow: bool,
    interval_ms: u64,
) -> Result<(), String> {
    trace_cli_action(
        "manage_logs",
        &format!(
            "server_id={} lines={} follow={} interval_ms={}",
            server.id, lines, follow, interval_ms
        ),
    );
    let logs = read_recent_server_logs(server, lines)?;
    println!("最近 {} 行日志: {}", logs.len(), server.name);
    for line in logs {
        println!("{}", line);
    }

    if follow {
        follow_server_logs(server, interval_ms.max(100))?;
    }

    Ok(())
}

fn restart_server(server: &ServerInstance) -> Result<(), String> {
    trace_cli_action("manage_restart", &format!("server_id={}", server.id));
    restart_server_with_wait(server, "manage_restart", DEFAULT_RESTART_STOP_TIMEOUT_SECS)
}

fn force_stop_server(server: &ServerInstance) -> Result<(), String> {
    trace_cli_action("manage_force_stop_prepare", &format!("server_id={}", server.id));
    let preparation = global::server_manager().prepare_force_stop_server(&server.id)?;

    println!("即将强制终止服务器 {}。这会直接中断运行时，不再等待优雅停服。", server.name);
    println!(
        "确认窗口约 15 秒，过期后需要重新执行 `sealantern server force-stop {}`。",
        server.id
    );

    let confirmed = prompt_yes_no_default_no("确认执行强制关停？ [y/N] ")?;
    if !confirmed {
        println!("已取消强制关停: {}", server.name);
        return Ok(());
    }

    trace_cli_action("manage_force_stop_confirm", &format!("server_id={}", server.id));
    global::server_manager().force_stop_server(&server.id, &preparation.token)?;
    println!("已执行强制关停: {}", server.name);
    Ok(())
}

fn print_server_inspect(server: &ServerInstance, status: &ServerStatusInfo) {
    trace_cli_action("manage_inspect", &describe_server_instance(server));
    for line in render_server_inspect_lines(server, status) {
        println!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        looks_like_reserved_manage_subcommand, parse_implicit_existing_start_command,
        parse_server_manage_command, ServerManageCommand, DEFAULT_LOG_FOLLOW_INTERVAL_MS,
        DEFAULT_LOG_LINES,
    };
    use crate::models::server::{
        CpuPolicyConfig, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
        JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::utils::cli::server_args::{CliMode, WebMode};
    use crate::utils::cli::server_manage_render::redact_secret;
    use crate::utils::cli::server_manage_start::ManageStartOptions;
    use crate::utils::cli::server_ref::resolve_server_reference_from_servers;
    use std::collections::BTreeMap;

    fn sample_local_server(name: &str, alias: &str) -> ServerInstance {
        ServerInstance {
            id: format!("{}-id", name),
            name: name.to_string(),
            aliases: vec![alias.to_string()],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: format!("E:/servers/{}", name),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/test/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn parse_logs_command_supports_lines_flag() {
        let args = vec![
            "logs".to_string(),
            "fabric".to_string(),
            "--lines".to_string(),
            "50".to_string(),
        ];

        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(
            command,
            Some(ServerManageCommand::Logs {
                target: "fabric".to_string(),
                lines: 50,
                follow: false,
                interval_ms: DEFAULT_LOG_FOLLOW_INTERVAL_MS,
            })
        );
    }

    #[test]
    fn parse_logs_command_supports_follow_and_interval() {
        let args = vec![
            "logs".to_string(),
            "fabric".to_string(),
            "--follow".to_string(),
            "--interval".to_string(),
            "250".to_string(),
        ];

        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(
            command,
            Some(ServerManageCommand::Logs {
                target: "fabric".to_string(),
                lines: DEFAULT_LOG_LINES,
                follow: true,
                interval_ms: 250,
            })
        );
    }

    #[test]
    fn parse_send_command_joins_remaining_tokens() {
        let args = vec![
            "send".to_string(),
            "paper".to_string(),
            "say".to_string(),
            "hello".to_string(),
            "world".to_string(),
        ];

        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(
            command,
            Some(ServerManageCommand::Send {
                target: "paper".to_string(),
                command: "say hello world".to_string(),
            })
        );
    }

    #[test]
    fn management_parser_preserves_create_flow_for_reserved_name() {
        let args = vec![
            "start".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--jar".to_string(),
            "E:/srv/server.jar".to_string(),
        ];

        assert!(parse_server_manage_command(&args).unwrap().is_none());
    }

    #[test]
    fn start_subcommand_shape_is_still_management_when_target_is_present() {
        let args = vec!["start".to_string(), "paper".to_string(), "--cli".to_string()];

        assert_eq!(
            parse_server_manage_command(&args).unwrap(),
            Some(ServerManageCommand::Start {
                target: "paper".to_string(),
                options: ManageStartOptions {
                    cli: CliMode::Enabled,
                    ..Default::default()
                },
            })
        );
    }

    #[test]
    fn parse_restart_command_uses_target_only_shape() {
        let args = vec!["restart".to_string(), "paper".to_string()];
        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(command, Some(ServerManageCommand::Restart { target: "paper".to_string() }));
    }

    #[test]
    fn parse_start_command_supports_optional_web_transport() {
        let args = vec![
            "start".to_string(),
            "paper".to_string(),
            "--web".to_string(),
            "8899".to_string(),
        ];

        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(
            command,
            Some(ServerManageCommand::Start {
                target: "paper".to_string(),
                options: ManageStartOptions {
                    cli: CliMode::Disabled,
                    web: WebMode::Enabled,
                    requested_web_port: Some(Some(8899)),
                },
            })
        );
    }

    #[test]
    fn parse_start_command_supports_cli_attach_only() {
        let args = vec!["start".to_string(), "paper".to_string(), "--cli".to_string()];

        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(
            command,
            Some(ServerManageCommand::Start {
                target: "paper".to_string(),
                options: ManageStartOptions {
                    cli: CliMode::Enabled,
                    web: WebMode::Disabled,
                    requested_web_port: None,
                },
            })
        );
    }

    #[test]
    fn parse_start_command_rejects_unknown_management_flags() {
        let args = vec![
            "start".to_string(),
            "paper".to_string(),
            "--runtime".to_string(),
            "docker".to_string(),
        ];

        let err = parse_server_manage_command(&args)
            .expect_err("unsupported management start flags should be rejected");
        assert!(err.contains("start 不支持的参数"));
        assert!(err.contains("--runtime"));
    }

    #[test]
    fn implicit_existing_server_start_with_web_transport_prefers_management_path() {
        let servers = vec![sample_local_server("fabric-main", "fabric")];
        let args = vec!["fabric-main".to_string(), "--web".to_string(), "8899".to_string()];

        let command = parse_implicit_existing_start_command(&args, &servers)
            .expect("implicit existing start parse should succeed");

        assert_eq!(
            command,
            Some(ServerManageCommand::Start {
                target: "fabric-main-id".to_string(),
                options: ManageStartOptions {
                    cli: CliMode::Disabled,
                    web: WebMode::Enabled,
                    requested_web_port: Some(Some(8899)),
                },
            })
        );
    }

    #[test]
    fn implicit_existing_server_start_without_transport_stays_out_of_management_path() {
        let servers = vec![sample_local_server("fabric-main", "fabric")];
        let args = vec!["fabric-main".to_string()];

        let command = parse_implicit_existing_start_command(&args, &servers)
            .expect("plain target should resolve to default CLI attach management path");

        assert_eq!(
            command,
            Some(ServerManageCommand::Start {
                target: "fabric-main-id".to_string(),
                options: ManageStartOptions {
                    cli: CliMode::Enabled,
                    ..Default::default()
                },
            })
        );
    }

    #[test]
    fn reserved_manage_subcommand_names_do_not_fall_into_implicit_existing_start_path() {
        assert!(looks_like_reserved_manage_subcommand("start"));
        assert!(looks_like_reserved_manage_subcommand("logs"));
        assert!(!looks_like_reserved_manage_subcommand("fabric-main"));
    }

    #[test]
    fn implicit_existing_server_start_does_not_swallow_create_flow_with_same_name() {
        let servers = vec![sample_local_server("fabric-main", "fabric")];
        let args = vec![
            "fabric-main".to_string(),
            "--jar".to_string(),
            "E:/srv/server.jar".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
        ];

        let command = parse_implicit_existing_start_command(&args, &servers)
            .expect("same-name create flow should not error");

        assert!(command.is_none());
    }

    #[test]
    fn parse_force_stop_command_uses_target_only_shape() {
        let args = vec!["force-stop".to_string(), "paper".to_string()];
        let command = parse_server_manage_command(&args).unwrap();
        assert_eq!(command, Some(ServerManageCommand::ForceStop { target: "paper".to_string() }));
    }

    #[test]
    fn parse_dedupe_commands_use_flag_only_shape() {
        assert_eq!(
            parse_server_manage_command(&["dedupe-audit".to_string()]).unwrap(),
            Some(ServerManageCommand::DedupeAudit)
        );
        assert_eq!(
            parse_server_manage_command(&["dedupe-apply".to_string()]).unwrap(),
            Some(ServerManageCommand::DedupeApply)
        );
    }

    #[test]
    fn resolve_server_reference_matches_alias() {
        let servers = vec![sample_local_server("fabric-main", "fabric")];
        let server = resolve_server_reference_from_servers(&servers, "fabric").unwrap();
        assert_eq!(server.name, "fabric-main");
    }

    #[test]
    fn resolve_server_reference_rejects_ambiguous_alias() {
        let servers = vec![
            sample_local_server("fabric-a", "shared"),
            sample_local_server("fabric-b", "shared"),
        ];
        let err = resolve_server_reference_from_servers(&servers, "shared").unwrap_err();
        assert!(err.contains("不唯一"));
    }

    #[test]
    fn redact_secret_hides_password_length_only() {
        assert_eq!(redact_secret("secret-pass"), "<redacted:11 chars>");
    }

    #[test]
    fn inspect_related_helpers_support_docker_runtime_shape() {
        let server = ServerInstance {
            id: "docker-id".to_string(),
            name: "paper-docker".to_string(),
            aliases: vec!["paper".to_string()],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/docker/paper".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sealantern-paper".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/paper".to_string(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: vec![],
                volume_mounts: vec![],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: vec![],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        };

        let resolved = resolve_server_reference_from_servers(std::slice::from_ref(&server), "paper-docker")
            .expect("docker runtime shape should still resolve by name");
        assert_eq!(resolved.runtime_kind, "docker_itzg");
    }
}
