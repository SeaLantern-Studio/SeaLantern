use crate::utils::cli::server_args::{parse_server_command, CliServerCommand};
use crate::utils::cli::server_create_entry::execute_server_command_for_entry as execute_for_entry;
use crate::utils::cli::server_create_flow::print_created_server_record;
use crate::utils::cli::server_entry::{
    handle_server_command as handle_server_command_entry, ServerCommandError,
};
use crate::utils::cli::server_execute::execute_server_command_with;
use crate::utils::cli::server_manage::try_handle_server_management_command;
use crate::utils::cli::server_ports::prepare_ports;
use crate::utils::cli::server_setup::{
    create_or_attach_docker_server, create_or_attach_local_server, ensure_memory_bounds,
    preflight_runtime_requirements, resolve_runtime_kind,
};
use crate::utils::cli::server_transport::{ensure_server_started, orchestrate_transports};

pub(super) fn handle_server_command(args: &[String]) {
    handle_server_command_entry(
        args,
        try_handle_server_management_command,
        parse_server_command,
        execute_server_command_for_entry,
    )
}

fn execute_server_command(mut command: CliServerCommand) -> Result<(), String> {
    execute_server_command_with(
        &mut command,
        resolve_runtime_kind,
        ensure_memory_bounds,
        preflight_runtime_requirements,
        |web_enabled, requested_web_port, requested_game_port| {
            prepare_ports(web_enabled, requested_web_port, requested_game_port)
        },
        |command, resolved_name, ports| {
            create_or_attach_local_server(command, resolved_name, ports)
        },
        |command, resolved_name, ports| {
            create_or_attach_docker_server(command, resolved_name, ports)
        },
        ensure_server_started,
        orchestrate_transports,
        print_created_server_record,
    )
}

#[allow(clippy::result_large_err)]
fn execute_server_command_for_entry(command: CliServerCommand) -> Result<(), ServerCommandError> {
    execute_for_entry(command, execute_server_command)
}

#[cfg(test)]
mod tests {
    use super::execute_server_command_with;
    use crate::utils::cli::server_args::{CliMode, CliServerCommand, WebMode};
    use crate::utils::cli::server_ports::PreparedPorts;
    use crate::utils::cli::server_shared::CliServerRuntimeKind;
    use crate::utils::cli::server_test_support::sample_server;
    use std::sync::{Arc, Mutex};

    #[test]
    fn execute_server_command_defaults_to_cli_and_local_runtime() {
        let mut command = CliServerCommand {
            positional_name: Some("fabric-1.20.1".to_string()),
            jar_path: Some("E:/srv/server.jar".to_string()),
            java_path: Some("C:/Java/bin/java.exe".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let prepare_events = Arc::clone(&events);
        let local_events = Arc::clone(&events);
        let orchestrate_events = Arc::clone(&events);
        let print_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Local),
            |_| Ok(()),
            |_, _| Ok(()),
            move |web_enabled, web_port, game_port| {
                prepare_events.lock().expect("events lock").push(format!(
                    "prepare:web_enabled={} web_port={:?} game_port={}",
                    web_enabled, web_port, game_port
                ));
                Ok(PreparedPorts { game_port, web_port })
            },
            move |command, resolved_name, ports| {
                local_events.lock().expect("events lock").push(format!(
                    "local:name={} cli={:?} game_port={}",
                    resolved_name, command.cli, ports.game_port
                ));
                Ok(sample_server())
            },
            move |_, _, _| Err("docker path should not execute".to_string()),
            move |_| Ok(()),
            move |command, _, ports| {
                orchestrate_events
                    .lock()
                    .expect("events lock")
                    .push(format!(
                        "orchestrate:cli={:?} web={:?} game_port={}",
                        command.cli, command.web, ports.game_port
                    ));
                Ok(())
            },
            move |server, game_port, web_port, runtime_kind, _| {
                print_events.lock().expect("events lock").push(format!(
                    "print:{}:{}:{:?}:{}",
                    server.id,
                    game_port,
                    web_port,
                    runtime_kind.as_runtime_label()
                ));
                Ok(())
            },
        )
        .expect("local default execution should succeed");

        assert_eq!(command.cli, CliMode::Enabled);
        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "prepare:web_enabled=false web_port=None game_port=25565",
                "local:name=fabric-1.20.1 cli=Enabled game_port=25565",
                "print:server-1:25565:None:local",
                "orchestrate:cli=Enabled web=Disabled game_port=25565"
            ]
        );
    }

    #[test]
    fn execute_server_command_prefers_docker_path_for_auto_runtime_with_image_hint() {
        let mut command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            web: WebMode::Enabled,
            web_port: Some(Some(8000)),
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let prepare_events = Arc::clone(&events);
        let docker_events = Arc::clone(&events);
        let orchestrate_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Docker),
            |_| Ok(()),
            |_, _| Ok(()),
            move |web_enabled, web_port, game_port| {
                prepare_events.lock().expect("events lock").push(format!(
                    "prepare:web_enabled={} web_port={:?} game_port={}",
                    web_enabled, web_port, game_port
                ));
                Ok(PreparedPorts { game_port, web_port })
            },
            move |_, _, _| Err("local path should not execute".to_string()),
            move |command, resolved_name, ports| {
                docker_events.lock().expect("events lock").push(format!(
                    "docker:name={} web={:?} image={:?} game_port={}",
                    resolved_name, command.web, command.image, ports.game_port
                ));
                Ok(sample_server())
            },
            move |_| Ok(()),
            move |command, _, ports| {
                orchestrate_events
                    .lock()
                    .expect("events lock")
                    .push(format!(
                        "orchestrate:cli={:?} web={:?} web_port={:?} game_port={}",
                        command.cli, command.web, ports.web_port, ports.game_port
                    ));
                Ok(())
            },
            move |_, _, _, _, _| Ok(()),
        )
        .expect("docker auto execution should succeed");

        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "prepare:web_enabled=true web_port=Some(8000) game_port=25565",
                "docker:name=paper-docker web=Enabled image=Some(\"itzg/minecraft-server\") game_port=25565",
                "orchestrate:cli=Disabled web=Enabled web_port=Some(8000) game_port=25565"
            ]
        );
    }

    #[test]
    fn execute_server_command_preserves_web_only_without_forcing_cli() {
        let mut command = CliServerCommand {
            positional_name: Some("web-only".to_string()),
            jar_path: Some("E:/srv/server.jar".to_string()),
            java_path: Some("C:/Java/bin/java.exe".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            web: WebMode::Enabled,
            web_port: Some(Some(8123)),
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let local_events = Arc::clone(&events);
        let orchestrate_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Local),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |command, _, _| {
                local_events
                    .lock()
                    .expect("events lock")
                    .push(format!("local:cli={:?} web={:?}", command.cli, command.web));
                Ok(sample_server())
            },
            move |_, _, _| Err("docker path should not execute".to_string()),
            move |_| Ok(()),
            move |command, _, ports| {
                orchestrate_events
                    .lock()
                    .expect("events lock")
                    .push(format!(
                        "orchestrate:cli={:?} web={:?} web_port={:?}",
                        command.cli, command.web, ports.web_port
                    ));
                Ok(())
            },
            move |_, _, _, _, _| Ok(()),
        )
        .expect("web-only execution should succeed");

        assert_eq!(command.cli, CliMode::Disabled);
        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "local:cli=Disabled web=Enabled",
                "orchestrate:cli=Disabled web=Enabled web_port=Some(8123)"
            ]
        );
    }

    #[test]
    fn execute_server_command_detach_starts_without_transport_orchestration() {
        let mut command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            detach: true,
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let docker_events = Arc::clone(&events);
        let print_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Docker),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |_, _, _| Err("local path should not execute".to_string()),
            move |command, resolved_name, ports| {
                docker_events.lock().expect("events lock").push(format!(
                    "docker:name={} detach={} cli={:?} web={:?} port={}",
                    resolved_name, command.detach, command.cli, command.web, ports.game_port
                ));
                Ok(sample_server())
            },
            move |_| Ok(()),
            move |_, _, _| Err("orchestrate should not run in detach mode".to_string()),
            move |_, game_port, web_port, runtime_kind, _| {
                print_events.lock().expect("events lock").push(format!(
                    "print:runtime={} game_port={} web_port={:?}",
                    runtime_kind.as_runtime_label(),
                    game_port,
                    web_port
                ));
                Ok(())
            },
        )
        .expect("detach execution should succeed");

        assert!(command.detach);
        assert_eq!(command.cli, CliMode::Disabled);
        assert_eq!(command.web, WebMode::Disabled);
        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "docker:name=paper-docker detach=true cli=Disabled web=Disabled port=25565",
                "print:runtime=docker_itzg game_port=25565 web_port=None"
            ]
        );
    }

    #[test]
    fn execute_server_command_detach_surfaces_start_failure_after_create() {
        let mut command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            detach: true,
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let docker_events = Arc::clone(&events);
        let print_events = Arc::clone(&events);
        let ensure_events = Arc::clone(&events);

        let err = execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Docker),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |_, _, _| Err("local path should not execute".to_string()),
            move |_, resolved_name, _| {
                docker_events
                    .lock()
                    .expect("events lock")
                    .push(format!("docker:name={}", resolved_name));
                Ok(sample_server())
            },
            move |_| {
                ensure_events
                    .lock()
                    .expect("events lock")
                    .push("ensure:start".to_string());
                Err("docker run failed".to_string())
            },
            move |_, _, _| Err("orchestrate should not run in detach mode".to_string()),
            move |_, game_port, web_port, runtime_kind, _| {
                print_events.lock().expect("events lock").push(format!(
                    "print:runtime={} game_port={} web_port={:?}",
                    runtime_kind.as_runtime_label(),
                    game_port,
                    web_port
                ));
                Ok(())
            },
        )
        .expect_err("detach start failure should surface");

        assert!(err.contains("docker run failed"));
        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "docker:name=paper-docker",
                "print:runtime=docker_itzg game_port=25565 web_port=None",
                "ensure:start"
            ]
        );
    }

    #[test]
    fn execute_server_command_create_only_skips_start_and_transport_orchestration() {
        let mut command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            create_only: true,
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let docker_events = Arc::clone(&events);
        let print_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Docker),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |_, _, _| Err("local path should not execute".to_string()),
            move |command, resolved_name, ports| {
                docker_events.lock().expect("events lock").push(format!(
                    "docker:name={} create_only={} detach={} cli={:?} web={:?} port={}",
                    resolved_name,
                    command.create_only,
                    command.detach,
                    command.cli,
                    command.web,
                    ports.game_port
                ));
                Ok(sample_server())
            },
            move |_| Err("ensure_started should not run in create-only mode".to_string()),
            move |_, _, _| Err("orchestrate should not run in create-only mode".to_string()),
            move |_, game_port, web_port, runtime_kind, _| {
                print_events.lock().expect("events lock").push(format!(
                    "print:runtime={} game_port={} web_port={:?}",
                    runtime_kind.as_runtime_label(),
                    game_port,
                    web_port
                ));
                Ok(())
            },
        )
        .expect("create-only execution should succeed");

        assert!(command.create_only);
        assert_eq!(command.cli, CliMode::Disabled);
        assert_eq!(command.web, WebMode::Disabled);
        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "docker:name=paper-docker create_only=true detach=false cli=Disabled web=Disabled port=25565",
                "print:runtime=docker_itzg game_port=25565 web_port=None"
            ]
        );
    }

    #[test]
    fn execute_server_command_allows_folder_only_name_inference() {
        let mut command = CliServerCommand {
            folder: Some("E:/servers/cache-server".to_string()),
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let local_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Local),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |_, resolved_name, _| {
                local_events
                    .lock()
                    .expect("events lock")
                    .push(format!("local:name={}", resolved_name));
                Ok(sample_server())
            },
            move |_, _, _| Err("docker path should not execute".to_string()),
            move |_| Ok(()),
            move |_, _, _| Ok(()),
            move |_, _, _, _, _| Ok(()),
        )
        .expect("folder-only flow should infer server name");

        assert_eq!(command.name.as_deref(), Some("cache-server"));
        let events = events.lock().expect("events lock");
        assert_eq!(events.as_slice(), ["local:name=cache-server"]);
    }

    #[test]
    fn execute_server_command_preserves_goal_style_local_inputs_through_create_flow() {
        let mut command = CliServerCommand {
            positional_name: Some("fabric-1.20.1".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            jar_path: Some("E:/srv/server.jar".to_string()),
            java_path: Some("C:/Java/bin/java.exe".to_string()),
            min_memory_mb: Some(2048),
            max_memory_mb: Some(4096),
            web: WebMode::Enabled,
            web_port: Some(Some(8000)),
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let local_events = Arc::clone(&events);
        let print_events = Arc::clone(&events);
        let orchestrate_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Local),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |command, resolved_name, ports| {
                local_events.lock().expect("events lock").push(format!(
                    "local:name={} jar={:?} java={:?} min={:?} max={:?} aliases={} port={}",
                    resolved_name,
                    command.jar_path,
                    command.java_path,
                    command.min_memory_mb,
                    command.max_memory_mb,
                    command.aliases.len(),
                    ports.game_port
                ));
                Ok(sample_server())
            },
            move |_, _, _| Err("docker path should not execute".to_string()),
            move |_| Ok(()),
            move |command, _, ports| {
                orchestrate_events
                    .lock()
                    .expect("events lock")
                    .push(format!(
                        "orchestrate:web={:?} cli={:?} web_port={:?} game_port={}",
                        command.web, command.cli, ports.web_port, ports.game_port
                    ));
                Ok(())
            },
            move |_, game_port, web_port, runtime_kind, aliases| {
                print_events.lock().expect("events lock").push(format!(
                    "print:runtime={} game_port={} web_port={:?} aliases={}",
                    runtime_kind.as_runtime_label(),
                    game_port,
                    web_port,
                    aliases.len()
                ));
                Ok(())
            },
        )
        .expect("goal-style local execution should succeed");

        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "local:name=fabric-1.20.1 jar=Some(\"E:/srv/server.jar\") java=Some(\"C:/Java/bin/java.exe\") min=Some(2048) max=Some(4096) aliases=0 port=25565",
                "print:runtime=local game_port=25565 web_port=Some(8000) aliases=0",
                "orchestrate:web=Enabled cli=Disabled web_port=Some(8000) game_port=25565"
            ]
        );
    }

    #[test]
    fn execute_server_command_preserves_docker_aliases_and_cli_web_combination() {
        let mut command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("latest".to_string()),
            aliases: vec!["cache_server".to_string(), "test_server".to_string()],
            web: WebMode::Enabled,
            web_port: Some(Some(8000)),
            cli: CliMode::Enabled,
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let docker_events = Arc::clone(&events);
        let print_events = Arc::clone(&events);
        let orchestrate_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Docker),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |_, _, _| Err("local path should not execute".to_string()),
            move |command, resolved_name, ports| {
                docker_events.lock().expect("events lock").push(format!(
                    "docker:name={} runtime={:?} image={:?}:{:?} aliases={} cli={:?} web={:?} port={}",
                    resolved_name,
                    command.runtime,
                    command.image,
                    command.image_tag,
                    command.aliases.len(),
                    command.cli,
                    command.web,
                    ports.game_port
                ));
                Ok(sample_server())
            },
            move |_| Ok(()),
            move |command, _, ports| {
                orchestrate_events
                    .lock()
                    .expect("events lock")
                    .push(format!(
                        "orchestrate:web={:?} cli={:?} web_port={:?} game_port={}",
                        command.web, command.cli, ports.web_port, ports.game_port
                    ));
                Ok(())
            },
            move |_, game_port, web_port, runtime_kind, aliases| {
                print_events.lock().expect("events lock").push(format!(
                    "print:runtime={} game_port={} web_port={:?} aliases={}",
                    runtime_kind.as_runtime_label(),
                    game_port,
                    web_port,
                    aliases.len()
                ));
                Ok(())
            },
        )
        .expect("docker execution with aliases should succeed");

        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "docker:name=paper-docker runtime=Some(\"docker\") image=Some(\"itzg/minecraft-server\"):Some(\"latest\") aliases=2 cli=Enabled web=Enabled port=25565",
                "print:runtime=docker_itzg game_port=25565 web_port=Some(8000) aliases=2",
                "orchestrate:web=Enabled cli=Enabled web_port=Some(8000) game_port=25565"
            ]
        );
    }

    #[test]
    fn execute_server_command_preserves_custom_entry_with_env_only_java_mode() {
        let mut command = CliServerCommand {
            positional_name: Some("custom-local".to_string()),
            mc_version: Some("1.20.1".to_string()),
            core_type: Some("fabric".to_string()),
            entry: Some("java -Xmx4G -Xms4G -jar server.jar nogui".to_string()),
            java_from_env_only: true,
            cli: CliMode::Enabled,
            ..Default::default()
        };
        let events = Arc::new(Mutex::new(Vec::new()));

        let local_events = Arc::clone(&events);
        let orchestrate_events = Arc::clone(&events);

        execute_server_command_with(
            &mut command,
            |_| Ok(CliServerRuntimeKind::Local),
            |_| Ok(()),
            |_, _| Ok(()),
            |web_enabled, web_port, game_port| {
                Ok(PreparedPorts {
                    game_port,
                    web_port: web_port.filter(|_| web_enabled),
                })
            },
            move |command, resolved_name, ports| {
                local_events.lock().expect("events lock").push(format!(
                    "local:name={} entry={:?} java_env_only={} cli={:?} port={}",
                    resolved_name,
                    command.entry,
                    command.java_from_env_only,
                    command.cli,
                    ports.game_port
                ));
                Ok(sample_server())
            },
            move |_, _, _| Err("docker path should not execute".to_string()),
            move |_| Ok(()),
            move |command, _, _| {
                orchestrate_events
                    .lock()
                    .expect("events lock")
                    .push(format!("orchestrate:entry={:?} cli={:?}", command.entry, command.cli));
                Ok(())
            },
            move |_, _, _, _, _| Ok(()),
        )
        .expect("custom entry local execution should succeed");

        let events = events.lock().expect("events lock");
        assert_eq!(
            events.as_slice(),
            [
                "local:name=custom-local entry=Some(\"java -Xmx4G -Xms4G -jar server.jar nogui\") java_env_only=true cli=Enabled port=25565",
                "orchestrate:entry=Some(\"java -Xmx4G -Xms4G -jar server.jar nogui\") cli=Enabled"
            ]
        );
    }
}
