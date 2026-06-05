use crate::models::server::ServerInstance;
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_create_flow::handle_created_server_flow;
use crate::utils::cli::server_flow::{
    ensure_transport_defaults, prepare_server_ports, resolve_server_command_name,
    validate_transport_mode,
};
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::{trace_cli_action, CliServerRuntimeKind};

pub(super) struct ServerCommandExecutionDeps<
    FResolveRuntime,
    FEnsureMemory,
    FPreflightRuntime,
    FPreparePorts,
    FCreateLocal,
    FCreateDocker,
    FEnsureStarted,
    FOrchestrate,
    FPrintCreated,
> {
    pub resolve_runtime: FResolveRuntime,
    pub ensure_memory: FEnsureMemory,
    pub preflight_runtime: FPreflightRuntime,
    pub prepare_ports_fn: FPreparePorts,
    pub create_local: FCreateLocal,
    pub create_docker: FCreateDocker,
    pub ensure_started: FEnsureStarted,
    pub orchestrate: FOrchestrate,
    pub print_created: FPrintCreated,
}

fn execute_server_command_with_deps<
    FResolveRuntime,
    FEnsureMemory,
    FPreflightRuntime,
    FPreparePorts,
    FCreateLocal,
    FCreateDocker,
    FEnsureStarted,
    FOrchestrate,
    FPrintCreated,
>(
    command: &mut CliServerCommand,
    deps: ServerCommandExecutionDeps<
        FResolveRuntime,
        FEnsureMemory,
        FPreflightRuntime,
        FPreparePorts,
        FCreateLocal,
        FCreateDocker,
        FEnsureStarted,
        FOrchestrate,
        FPrintCreated,
    >,
) -> Result<(), String>
where
    FResolveRuntime: Fn(&CliServerCommand) -> Result<CliServerRuntimeKind, String>,
    FEnsureMemory: Fn(&CliServerCommand) -> Result<(), String>,
    FPreflightRuntime: Fn(&mut CliServerCommand, CliServerRuntimeKind) -> Result<(), String>,
    FPreparePorts: Fn(bool, Option<u16>, u16) -> Result<PreparedPorts, String>,
    FCreateLocal: Fn(&CliServerCommand, &str, &PreparedPorts) -> Result<ServerInstance, String>,
    FCreateDocker: Fn(&CliServerCommand, &str, &PreparedPorts) -> Result<ServerInstance, String>,
    FEnsureStarted: Fn(&ServerInstance) -> Result<(), String>,
    FOrchestrate: Fn(&CliServerCommand, &ServerInstance, &PreparedPorts) -> Result<(), String>,
    FPrintCreated: Fn(
        &ServerInstance,
        u16,
        Option<u16>,
        CliServerRuntimeKind,
        &[String],
    ) -> Result<(), String>,
{
    let ServerCommandExecutionDeps {
        resolve_runtime,
        ensure_memory,
        preflight_runtime,
        prepare_ports_fn,
        create_local,
        create_docker,
        ensure_started,
        orchestrate,
        print_created,
    } = deps;

    let resolved_name = resolve_server_command_name(command)?;
    command.name = Some(resolved_name.clone());
    trace_cli_action(
        "resolved_name",
        &format!("name={} aliases={}", resolved_name, command.aliases.join(",")),
    );

    let runtime_kind = resolve_runtime(command)?;
    if !command.runtime_prevalidated {
        preflight_runtime(command, runtime_kind)?;
        command.runtime_prevalidated = true;
    }
    ensure_memory(command)?;
    validate_transport_mode(command)?;
    ensure_transport_defaults(command);
    let ports = prepare_server_ports(command, runtime_kind, prepare_ports_fn)?;

    let server = match runtime_kind {
        CliServerRuntimeKind::Local => create_local(command, &resolved_name, &ports)?,
        CliServerRuntimeKind::Docker => create_docker(command, &resolved_name, &ports)?,
    };

    print_created(&server, ports.game_port, ports.web_port, runtime_kind, &command.aliases)?;
    handle_created_server_flow(command, &server, &ports, runtime_kind, ensure_started, orchestrate)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_server_command_with<
    FResolveRuntime,
    FEnsureMemory,
    FPreflightRuntime,
    FPreparePorts,
    FCreateLocal,
    FCreateDocker,
    FEnsureStarted,
    FOrchestrate,
    FPrintCreated,
>(
    command: &mut CliServerCommand,
    resolve_runtime: FResolveRuntime,
    ensure_memory: FEnsureMemory,
    preflight_runtime: FPreflightRuntime,
    prepare_ports_fn: FPreparePorts,
    create_local: FCreateLocal,
    create_docker: FCreateDocker,
    ensure_started: FEnsureStarted,
    orchestrate: FOrchestrate,
    print_created: FPrintCreated,
) -> Result<(), String>
where
    FResolveRuntime: Fn(&CliServerCommand) -> Result<CliServerRuntimeKind, String>,
    FEnsureMemory: Fn(&CliServerCommand) -> Result<(), String>,
    FPreflightRuntime: Fn(&mut CliServerCommand, CliServerRuntimeKind) -> Result<(), String>,
    FPreparePorts: Fn(bool, Option<u16>, u16) -> Result<PreparedPorts, String>,
    FCreateLocal: Fn(&CliServerCommand, &str, &PreparedPorts) -> Result<ServerInstance, String>,
    FCreateDocker: Fn(&CliServerCommand, &str, &PreparedPorts) -> Result<ServerInstance, String>,
    FEnsureStarted: Fn(&ServerInstance) -> Result<(), String>,
    FOrchestrate: Fn(&CliServerCommand, &ServerInstance, &PreparedPorts) -> Result<(), String>,
    FPrintCreated: Fn(
        &ServerInstance,
        u16,
        Option<u16>,
        CliServerRuntimeKind,
        &[String],
    ) -> Result<(), String>,
{
    execute_server_command_with_deps(
        command,
        ServerCommandExecutionDeps {
            resolve_runtime,
            ensure_memory,
            preflight_runtime,
            prepare_ports_fn,
            create_local,
            create_docker,
            ensure_started,
            orchestrate,
            print_created,
        },
    )
}
