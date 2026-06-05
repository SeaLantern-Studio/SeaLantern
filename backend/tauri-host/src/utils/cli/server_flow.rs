use std::path::Path;

use crate::utils::cli::server_args::{CliMode, CliServerCommand, WebMode};
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::{trace_cli_action, CliServerRuntimeKind};

pub(super) fn resolve_server_command_name(command: &CliServerCommand) -> Result<String, String> {
    command
        .name
        .clone()
        .or_else(|| command.positional_name.clone())
        .or_else(|| command.server_tag.clone())
        .or_else(|| infer_server_name_from_folder(command.folder.as_deref()))
        .ok_or_else(|| "缺少服务器名称，请使用位置参数、--name、--tag 或 --folder".to_string())
}

pub(super) fn infer_server_name_from_folder(folder: Option<&str>) -> Option<String> {
    let folder = folder?.trim();
    if folder.is_empty() {
        return None;
    }

    Path::new(folder)
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

pub(super) fn ensure_transport_defaults(command: &mut CliServerCommand) {
    if command.create_only || command.detach {
        command.web = WebMode::Disabled;
        command.cli = CliMode::Disabled;
        return;
    }

    if command.web == WebMode::Disabled && command.cli == CliMode::Disabled {
        command.cli = CliMode::Enabled;
    }
}

pub(super) fn validate_transport_mode(command: &CliServerCommand) -> Result<(), String> {
    if command.create_only && command.detach {
        return Err("--create-only 不能与 --detach 同时使用".to_string());
    }

    if command.create_only && (command.web == WebMode::Enabled || command.cli == CliMode::Enabled) {
        return Err("--create-only 不能与 --web 或 --cli 同时使用".to_string());
    }

    if command.detach && (command.web == WebMode::Enabled || command.cli == CliMode::Enabled) {
        return Err("--detach 不能与 --web 或 --cli 同时使用".to_string());
    }

    Ok(())
}

pub(super) fn prepare_server_ports<FPreparePorts>(
    command: &CliServerCommand,
    runtime_kind: CliServerRuntimeKind,
    prepare_ports_fn: FPreparePorts,
) -> Result<PreparedPorts, String>
where
    FPreparePorts: Fn(bool, Option<u16>, u16) -> Result<PreparedPorts, String>,
{
    if command.create_only {
        let ports = PreparedPorts {
            game_port: command.port.flatten().unwrap_or(25565),
            web_port: None,
        };

        trace_cli_action(
            "prepared_ports_create_only",
            &format!(
                "runtime={} game_port={} web_port=disabled",
                runtime_kind.as_runtime_label(),
                ports.game_port,
            ),
        );

        return Ok(ports);
    }

    let ports = prepare_ports_fn(
        command.web == WebMode::Enabled,
        command.web_port.flatten(),
        command.port.flatten().unwrap_or(25565),
    )?;

    trace_cli_action(
        "prepared_ports",
        &format!(
            "runtime={} game_port={} web_port={}",
            runtime_kind.as_runtime_label(),
            ports.game_port,
            ports
                .web_port
                .map(|value| value.to_string())
                .unwrap_or_else(|| "disabled".to_string())
        ),
    );

    Ok(ports)
}
