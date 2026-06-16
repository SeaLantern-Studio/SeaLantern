use crate::models::server::ServerInstance;
use crate::services::global;

use super::server_args::{CliMode, WebMode};
use super::server_endpoint::{render_cli_web_binding_hint, render_cli_web_browser_url};
use super::server_ports::{prepare_ports, prepare_web_port_only, PreparedPorts};
use super::server_shared::trace_cli_action;
use super::server_transport::orchestrate_transports_with;
use crate::utils::server_status::status_blocks_start;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct ManageStartOptions {
    pub(super) cli: CliMode,
    pub(super) web: WebMode,
    pub(super) requested_web_port: Option<Option<u16>>,
}

impl ManageStartOptions {
    pub(super) fn has_transport_request(&self) -> bool {
        self.cli == CliMode::Enabled || self.web == WebMode::Enabled
    }

    pub(super) fn requested_web_port_value(&self) -> Option<u16> {
        self.requested_web_port.flatten()
    }
}

pub(super) fn parse_manage_start_options(args: &[String]) -> Result<ManageStartOptions, String> {
    let mut options = ManageStartOptions::default();
    let mut index = 0;
    while index < args.len() {
        let current = args[index].as_str();

        if current == "--cli" {
            options.cli = CliMode::Enabled;
            index += 1;
            continue;
        }
        if current == "--web" {
            options.web = WebMode::Enabled;
            if let Some(next) = args.get(index + 1).filter(|value| !value.starts_with('-')) {
                let port = next
                    .parse::<u16>()
                    .map_err(|_| format!("--web 需要有效端口号: {}", next))?;
                options.requested_web_port = Some(Some(port));
                index += 2;
            } else {
                options.requested_web_port = Some(None);
                index += 1;
            }
            continue;
        }
        if let Some(value) = current.strip_prefix("--web=") {
            let port = value
                .parse::<u16>()
                .map_err(|_| format!("--web 需要有效端口号: {}", value))?;
            options.web = WebMode::Enabled;
            options.requested_web_port = Some(Some(port));
            index += 1;
            continue;
        }
        if let Some(value) = current.strip_prefix("--web:") {
            let port = value
                .parse::<u16>()
                .map_err(|_| format!("--web 需要有效端口号: {}", value))?;
            options.web = WebMode::Enabled;
            options.requested_web_port = Some(Some(port));
            index += 1;
            continue;
        }
        if let Some(value) = current.strip_prefix("-web:") {
            let port = value
                .parse::<u16>()
                .map_err(|_| format!("-web 需要有效端口号: {}", value))?;
            options.web = WebMode::Enabled;
            options.requested_web_port = Some(Some(port));
            index += 1;
            continue;
        }

        return Err(format!("start 不支持的参数: {}", current));
    }

    Ok(options)
}

pub(super) fn args_are_transport_only(args: &[String]) -> bool {
    !args.is_empty()
        && args.iter().all(|arg| {
            matches!(arg.as_str(), "--cli" | "--web")
                || arg.starts_with("--web=")
                || arg.starts_with("--web:")
                || arg.starts_with("-web:")
                || arg.parse::<u16>().is_ok()
        })
}

pub(super) fn start_existing_server_with_optional_transports(
    server: &ServerInstance,
    options: &ManageStartOptions,
) -> Result<(), String> {
    if !options.has_transport_request() {
        super::server_control::start_server_with_feedback(
            server,
            "manage_start",
            "服务器正在启动...",
        )?;
        return Ok(());
    }

    let prepared_ports = prepare_manage_start_ports(server, options)?;
    trace_cli_action(
        "manage_start_with_transport",
        &format!(
            "server_id={} cli={:?} web={:?} web_port={:?}",
            server.id, options.cli, options.web, prepared_ports.web_port
        ),
    );

    orchestrate_transports_with(
        &build_manage_transport_command(options),
        server,
        &prepared_ports,
        |server| {
            super::server_control::start_server_with_feedback(
                server,
                "manage_start",
                "服务器正在启动...",
            )?;
            Ok(())
        },
        |port, server| {
            let url = render_cli_web_browser_url(port, &server.id);
            trace_cli_action(
                "manage_start_web_attach",
                &format!("server_id={} port={} url={}", server.id, port, url),
            );
            if let Some(hint) = render_cli_web_binding_hint(port) {
                println!("web transport 绑定提示: {}", hint);
            }
            super::server_transport::start_web_transport(port, server)
        },
        |server| {
            trace_cli_action("manage_start_cli_attach", &format!("server_id={}", server.id));
            super::server_session::attach_server_cli(server);
            Ok(())
        },
        |web_handle| {
            println!("web transport 正在运行: {}\n按 Ctrl+C 可结束当前命令进程。", web_handle.url);
            let _ = web_handle.join_handle.join();
            Ok(())
        },
    )
}

fn prepare_manage_start_ports(
    server: &ServerInstance,
    options: &ManageStartOptions,
) -> Result<PreparedPorts, String> {
    let current_status = global::server_manager().get_server_status(&server.id);
    let runtime_is_active = status_blocks_start(&current_status);

    prepare_manage_start_ports_with(
        server,
        options,
        runtime_is_active,
        prepare_ports,
        prepare_web_port_only,
    )
}

fn prepare_manage_start_ports_with<FPreparePorts, FPrepareWebOnly>(
    server: &ServerInstance,
    options: &ManageStartOptions,
    runtime_is_active: bool,
    mut prepare_ports_fn: FPreparePorts,
    mut prepare_web_port_only_fn: FPrepareWebOnly,
) -> Result<PreparedPorts, String>
where
    FPreparePorts: FnMut(bool, Option<u16>, u16) -> Result<PreparedPorts, String>,
    FPrepareWebOnly: FnMut(Option<u16>, u16) -> Result<Option<u16>, String>,
{
    let web_enabled = options.web == WebMode::Enabled;

    if runtime_is_active {
        let web_port = if web_enabled {
            prepare_web_port_only_fn(options.requested_web_port_value(), server.port).map_err(
                |err| {
                    format!(
                        "为已有服务器附加 web transport 失败 (server_id={}): {}",
                        server.id, err
                    )
                },
            )?
        } else {
            None
        };

        let ports = PreparedPorts {
            game_port: server.port,
            web_port,
        };

        trace_cli_action(
            "manage_start_ports_existing_runtime",
            &format!(
                "server_id={} runtime_is_active={} game_port={} web_port={:?}",
                server.id, runtime_is_active, ports.game_port, ports.web_port
            ),
        );

        return Ok(ports);
    }

    prepare_ports_fn(web_enabled, options.requested_web_port_value(), server.port).inspect(
        |ports| {
            trace_cli_action(
                "manage_start_ports",
                &format!(
                    "server_id={} game_port={} web_port={:?}",
                    server.id, ports.game_port, ports.web_port
                ),
            );
        },
    )
}

fn build_manage_transport_command(
    options: &ManageStartOptions,
) -> super::server_args::CliServerCommand {
    super::server_args::CliServerCommand {
        cli: options.cli,
        web: options.web,
        web_port: options.requested_web_port,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        args_are_transport_only, parse_manage_start_options, prepare_manage_start_ports_with,
        ManageStartOptions,
    };
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, LocalTerminalMode, ServerInstance,
        ServerRuntimeConfig,
    };
    use crate::utils::cli::server_args::{CliMode, WebMode};
    use crate::utils::cli::server_ports::PreparedPorts;

    fn sample_server() -> ServerInstance {
        ServerInstance {
            id: "server-1".to_string(),
            name: "sample".to_string(),
            aliases: vec![],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: "E:/servers/sample".to_string(),
            port: 25579,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/sample/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                terminal_mode: LocalTerminalMode::PipeManaged,
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn manage_start_options_detect_transport_request() {
        assert!(!ManageStartOptions::default().has_transport_request());
        assert!(ManageStartOptions {
            cli: CliMode::Enabled,
            ..Default::default()
        }
        .has_transport_request());
        assert!(ManageStartOptions {
            web: WebMode::Enabled,
            ..Default::default()
        }
        .has_transport_request());
    }

    #[test]
    fn manage_start_options_extract_requested_web_port() {
        let options = ManageStartOptions {
            web: WebMode::Enabled,
            requested_web_port: Some(Some(8899)),
            ..Default::default()
        };

        assert_eq!(options.requested_web_port_value(), Some(8899));
    }

    #[test]
    fn parse_manage_start_options_supports_web_and_cli_shapes() {
        let parsed = parse_manage_start_options(&["--cli".to_string(), "--web:8899".to_string()])
            .expect("manage start options should parse");

        assert_eq!(parsed.cli, CliMode::Enabled);
        assert_eq!(parsed.web, WebMode::Enabled);
        assert_eq!(parsed.requested_web_port, Some(Some(8899)));
    }

    #[test]
    fn args_are_transport_only_rejects_create_flow_flags() {
        assert!(args_are_transport_only(&["--cli".to_string(), "--web:8899".to_string()]));
        assert!(!args_are_transport_only(&["--runtime".to_string(), "docker".to_string()]));
        assert!(!args_are_transport_only(&["--jar".to_string(), "server.jar".to_string()]));
    }

    #[test]
    fn args_are_transport_only_accepts_space_style_web_port_for_manage_start() {
        assert!(args_are_transport_only(&[
            "--web".to_string(),
            "8899".to_string(),
            "--cli".to_string()
        ]));
    }

    #[test]
    fn existing_runtime_attach_reuses_recorded_game_port_without_prompting() {
        let server = sample_server();
        let options = ManageStartOptions {
            web: WebMode::Enabled,
            requested_web_port: Some(Some(8899)),
            ..Default::default()
        };

        let mut normal_calls = Vec::new();
        let mut web_only_calls = Vec::new();
        let ports = prepare_manage_start_ports_with(
            &server,
            &options,
            true,
            |web_enabled, requested_web_port, game_port| {
                normal_calls.push((web_enabled, requested_web_port, game_port));
                Ok(PreparedPorts { game_port, web_port: requested_web_port })
            },
            |requested_web_port, reserved_game_port| {
                web_only_calls.push((requested_web_port, reserved_game_port));
                Ok(requested_web_port)
            },
        )
        .expect("existing runtime attach should resolve ports");

        assert_eq!(ports.game_port, 25579);
        assert_eq!(ports.web_port, Some(8899));
        assert!(normal_calls.is_empty());
        assert_eq!(web_only_calls, vec![(Some(8899), 25579)]);
    }

    #[test]
    fn inactive_runtime_attach_uses_normal_prepare_ports_flow() {
        let server = sample_server();
        let options = ManageStartOptions {
            web: WebMode::Enabled,
            requested_web_port: Some(Some(8899)),
            ..Default::default()
        };

        let mut calls = Vec::new();
        let ports = prepare_manage_start_ports_with(
            &server,
            &options,
            false,
            |web_enabled, requested_web_port, game_port| {
                calls.push((web_enabled, requested_web_port, game_port));
                Ok(PreparedPorts {
                    game_port: 25580,
                    web_port: requested_web_port,
                })
            },
            |_, _| panic!("inactive runtime should not use web-only prepare path"),
        )
        .expect("inactive runtime attach should use normal port prep");

        assert_eq!(ports.game_port, 25580);
        assert_eq!(ports.web_port, Some(8899));
        assert_eq!(calls, vec![(true, Some(8899), 25579)]);
    }
}
