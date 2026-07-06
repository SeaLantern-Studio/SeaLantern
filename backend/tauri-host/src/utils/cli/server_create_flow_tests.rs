#![allow(clippy::result_large_err)]

use crate::utils::cli::server_args::{parse_server_command, CliServerCommand, WebMode};
use crate::utils::cli::server_entry::{
    run_server_command_with, ServerCommandError, ServerCommandErrorKind, ServerCommandOutcome,
};
use crate::utils::cli::server_execute::execute_server_command_with;
use crate::utils::cli::server_help::build_server_help_text;
use crate::utils::cli::server_ports::prepare_ports_with;
use crate::utils::cli::server_shared::CliServerRuntimeKind;
use crate::utils::cli::server_test_support::sample_server;
use std::sync::{Arc, Mutex};

#[test]
fn server_help_text_mentions_docker_command_mode_and_env_overrides() {
    let help = build_server_help_text();

    assert!(help.contains("--command-mode <rcon|docker_stdio>"));
    assert!(help.contains("--detach"));
    assert!(help.contains("--create-only"));
    assert!(help.contains("SEALANTERN_DOCKER_RCON_HOST=<host>"));
    assert!(help.contains("桌面、Headless 和容器环境"));
    assert!(help.contains("custom-local"));
    assert!(help.contains("sealantern server inspect <target>"));
    assert!(help.contains("sealantern server send paper-docker"));
    assert!(help.contains("sealantern compose generate paper-docker"));
    assert!(help.contains("sealantern server restart <target>"));
    assert!(help.contains("--follow --interval 1000"));
    assert!(help.contains("会立即返回"));
    assert!(help.contains("--folder 目录名"));
    assert!(help.contains("/inspect 查看服务器详情与运行时配置"));
    assert!(help.contains("/restart 请求重启当前服务器"));
}

#[test]
fn run_server_command_routes_manage_subcommand_before_create_flow() {
    let args = vec!["list".to_string()];
    let events = Arc::new(Mutex::new(Vec::new()));

    let manage_events = Arc::clone(&events);
    let parse_events = Arc::clone(&events);
    let execute_events = Arc::clone(&events);

    let result = run_server_command_with(
        &args,
        move |incoming| {
            manage_events
                .lock()
                .expect("events lock")
                .push(format!("manage:{:?}", incoming));
            Ok(true)
        },
        move |_| {
            parse_events
                .lock()
                .expect("events lock")
                .push("parse".to_string());
            Err("parse should not run".to_string())
        },
        move |_| {
            execute_events
                .lock()
                .expect("events lock")
                .push("execute".to_string());
            Err(ServerCommandError {
                kind: ServerCommandErrorKind::Execute,
                message: "execute should not run".to_string(),
                parsed_command: None,
                preflight_error: None,
            })
        },
    )
    .expect("manage flow should succeed");

    assert_eq!(result, ServerCommandOutcome::ManagedHandled);
    let events = events.lock().expect("events lock");
    assert_eq!(events.as_slice(), ["manage:[\"list\"]"]);
}

#[test]
fn run_server_command_returns_help_outcome_for_help_request() {
    let args = vec!["help".to_string()];
    let events = Arc::new(Mutex::new(Vec::new()));

    let parse_events = Arc::clone(&events);
    let execute_events = Arc::clone(&events);

    let result = run_server_command_with(
        &args,
        |_| Ok(false),
        move |incoming| {
            parse_events
                .lock()
                .expect("events lock")
                .push(format!("parse:{:?}", incoming));
            Err("__PRINT_HELP__".to_string())
        },
        move |_| {
            execute_events
                .lock()
                .expect("events lock")
                .push("execute".to_string());
            Err(ServerCommandError {
                kind: ServerCommandErrorKind::Execute,
                message: "execute should not run".to_string(),
                parsed_command: None,
                preflight_error: None,
            })
        },
    )
    .expect("help request should not fail");

    assert_eq!(result, ServerCommandOutcome::PrintHelpRequested);
    let events = events.lock().expect("events lock");
    assert_eq!(events.as_slice(), ["parse:[\"help\"]"]);
}

#[test]
fn run_server_command_executes_goal_style_create_flow_from_raw_args() {
    let args = vec![
        "fabric-1.20.1".to_string(),
        "--mc".to_string(),
        "1.20.1".to_string(),
        "--core".to_string(),
        "fabric".to_string(),
        "--jar".to_string(),
        "E:/srv/server.jar".to_string(),
        "--java".to_string(),
        "C:/Java/bin/java.exe".to_string(),
        "-p:25565".to_string(),
        "-min:2G".to_string(),
        "-max:4G".to_string(),
        "-web:8000".to_string(),
    ];
    let events = Arc::new(Mutex::new(Vec::new()));

    let parse_events = Arc::clone(&events);
    let execute_events = Arc::clone(&events);

    let result = run_server_command_with(
        &args,
        |_| Ok(false),
        move |incoming| {
            parse_events
                .lock()
                .expect("events lock")
                .push(format!("parse:{:?}", incoming));
            parse_server_command(incoming)
        },
        move |command| {
            execute_events.lock().expect("events lock").push(format!(
                "execute:name={:?} mc={:?} core={:?} jar={:?} java={:?} port={:?} min={:?} max={:?} web={:?} web_port={:?}",
                command.positional_name,
                command.mc_version,
                command.core_type,
                command.jar_path,
                command.java_path,
                command.port,
                command.min_memory_mb,
                command.max_memory_mb,
                command.web,
                command.web_port
            ));
            Ok(())
        },
    )
    .expect("goal-style raw args should execute create flow");

    assert_eq!(result, ServerCommandOutcome::CreateExecuted);
    let events = events.lock().expect("events lock");
    assert_eq!(
        events.as_slice(),
        [
            "parse:[\"fabric-1.20.1\", \"--mc\", \"1.20.1\", \"--core\", \"fabric\", \"--jar\", \"E:/srv/server.jar\", \"--java\", \"C:/Java/bin/java.exe\", \"-p:25565\", \"-min:2G\", \"-max:4G\", \"-web:8000\"]",
            "execute:name=Some(\"fabric-1.20.1\") mc=Some(\"1.20.1\") core=Some(\"fabric\") jar=Some(\"E:/srv/server.jar\") java=Some(\"C:/Java/bin/java.exe\") port=Some(Some(25565)) min=Some(2048) max=Some(4096) web=Enabled web_port=Some(Some(8000))"
        ]
    );
}

#[test]
fn run_server_command_reports_manage_errors_with_manage_kind() {
    let args = vec!["list".to_string()];

    let err = run_server_command_with(
        &args,
        |_| Err("manage failed".to_string()),
        |_| Err("parse should not run".to_string()),
        |_| {
            Err(ServerCommandError {
                kind: ServerCommandErrorKind::Execute,
                message: "execute should not run".to_string(),
                parsed_command: None,
                preflight_error: None,
            })
        },
    )
    .expect_err("manage failure should surface");

    assert_eq!(err.kind, ServerCommandErrorKind::Manage);
    assert_eq!(err.message, "manage failed");
}

#[test]
fn execute_server_command_uses_auto_shifted_web_port_in_create_flow() {
    let mut command = CliServerCommand {
        positional_name: Some("fabric-1.20.1".to_string()),
        jar_path: Some("E:/srv/server.jar".to_string()),
        java_path: Some("C:/Java/bin/java.exe".to_string()),
        mc_version: Some("1.20.1".to_string()),
        core_type: Some("fabric".to_string()),
        web: WebMode::Enabled,
        web_port: Some(Some(8888)),
        ..Default::default()
    };
    let events = Arc::new(Mutex::new(Vec::new()));

    let local_events = Arc::clone(&events);
    let print_events = Arc::clone(&events);

    execute_server_command_with(
        &mut command,
        |_| Ok(CliServerRuntimeKind::Local),
        |_| Ok(()),
        |_, _| Ok(()),
        |web_enabled, requested_web_port, requested_game_port| {
            prepare_ports_with(
                web_enabled,
                requested_web_port,
                requested_game_port,
                |port, _| port != 8888,
                |_| Ok(true),
            )
        },
        move |_, _, ports| {
            local_events
                .lock()
                .expect("events lock")
                .push(format!("local:game_port={} web_port={:?}", ports.game_port, ports.web_port));
            Ok(sample_server())
        },
        move |_, _, _| Err("docker path should not execute".to_string()),
        move |_| Ok(()),
        move |_, _, _| Ok(()),
        move |_, game_port, web_port, _, _| {
            print_events
                .lock()
                .expect("events lock")
                .push(format!("print:game_port={} web_port={:?}", game_port, web_port));
            Ok(())
        },
    )
    .expect("web auto-shift create flow should succeed");

    let events = events.lock().expect("events lock");
    assert_eq!(
        events.as_slice(),
        [
            "local:game_port=25565 web_port=Some(8889)",
            "print:game_port=25565 web_port=Some(8889)"
        ]
    );
}

#[test]
fn execute_server_command_resolves_double_conflict_in_web_then_mc_order() {
    let mut command = CliServerCommand {
        positional_name: Some("fabric-1.20.1".to_string()),
        jar_path: Some("E:/srv/server.jar".to_string()),
        java_path: Some("C:/Java/bin/java.exe".to_string()),
        mc_version: Some("1.20.1".to_string()),
        core_type: Some("fabric".to_string()),
        web: WebMode::Enabled,
        web_port: Some(Some(8888)),
        ..Default::default()
    };
    let events = Arc::new(Mutex::new(Vec::new()));

    let prompt_events = Arc::clone(&events);
    let local_events = Arc::clone(&events);

    execute_server_command_with(
        &mut command,
        |_| Ok(CliServerRuntimeKind::Local),
        |_| Ok(()),
        |_, _| Ok(()),
        move |web_enabled, requested_web_port, requested_game_port| {
            let prompt_events = Arc::clone(&prompt_events);
            prepare_ports_with(
                web_enabled,
                requested_web_port,
                requested_game_port,
                |port, _| port != 8888 && port != 25565,
                move |message| {
                    prompt_events
                        .lock()
                        .expect("events lock")
                        .push(format!("prompt:{}", message));
                    Ok(true)
                },
            )
        },
        move |_, _, ports| {
            local_events
                .lock()
                .expect("events lock")
                .push(format!("local:game_port={} web_port={:?}", ports.game_port, ports.web_port));
            Ok(sample_server())
        },
        move |_, _, _| Err("docker path should not execute".to_string()),
        move |_| Ok(()),
        move |_, _, _| Ok(()),
        move |_, _, _, _, _| Ok(()),
    )
    .expect("double conflict create flow should succeed");

    let events = events.lock().expect("events lock");
    assert_eq!(events.len(), 3);
    assert!(events[0].contains("prompt:Web 端口 8888"));
    assert!(events[1].contains("prompt:Minecraft 端口 25565"));
    assert_eq!(events[2], "local:game_port=25566 web_port=Some(8889)");
}

#[test]
fn execute_server_command_aborts_create_flow_when_user_rejects_web_switch_in_double_conflict() {
    let mut command = CliServerCommand {
        positional_name: Some("fabric-1.20.1".to_string()),
        jar_path: Some("E:/srv/server.jar".to_string()),
        java_path: Some("C:/Java/bin/java.exe".to_string()),
        mc_version: Some("1.20.1".to_string()),
        core_type: Some("fabric".to_string()),
        web: WebMode::Enabled,
        web_port: Some(Some(8888)),
        ..Default::default()
    };
    let events = Arc::new(Mutex::new(Vec::new()));

    let prompt_events = Arc::clone(&events);
    let local_events = Arc::clone(&events);

    let err = execute_server_command_with(
        &mut command,
        |_| Ok(CliServerRuntimeKind::Local),
        |_| Ok(()),
        |_, _| Ok(()),
        move |web_enabled, requested_web_port, requested_game_port| {
            let prompt_events = Arc::clone(&prompt_events);
            prepare_ports_with(
                web_enabled,
                requested_web_port,
                requested_game_port,
                |port, _| port != 8888 && port != 25565,
                move |message| {
                    prompt_events
                        .lock()
                        .expect("events lock")
                        .push(format!("prompt:{}", message));
                    Ok(false)
                },
            )
        },
        move |_, _, _| {
            local_events
                .lock()
                .expect("events lock")
                .push("local".to_string());
            Ok(sample_server())
        },
        move |_, _, _| Err("docker path should not execute".to_string()),
        move |_| Ok(()),
        move |_, _, _| Ok(()),
        move |_, _, _, _, _| Ok(()),
    )
    .expect_err("rejecting web switch should abort create flow");

    assert!(err.contains("Web 端口") || err.contains("web 端口"));
    let events = events.lock().expect("events lock");
    assert_eq!(events.len(), 1);
    assert!(events[0].contains("prompt:Web 端口 8888"));
}

#[test]
fn help_text_mentions_existing_server_default_cli_behavior() {
    let help = build_server_help_text();

    assert!(help.contains("默认等价于 `sealantern server start <target> --cli`"));
    assert!(help.contains("sealantern server paper-docker"));
}
