use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_feedback::{
    render_runtime_preflight_failure_hint_lines,
    render_runtime_preflight_failure_hint_lines_from_error,
};
use crate::utils::cli::server_help::print_server_help;
use crate::utils::cli::server_setup::{resolve_runtime_kind, RuntimePreflightError};
use crate::utils::cli::server_shared::{trace_cli_action, trace_cli_error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ServerCommandOutcome {
    ManagedHandled,
    CreateExecuted,
    PrintHelpRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ServerCommandErrorKind {
    Manage,
    Parse,
    Execute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServerCommandError {
    pub(super) kind: ServerCommandErrorKind,
    pub(super) message: String,
    pub(super) parsed_command: Option<CliServerCommand>,
    pub(super) preflight_error: Option<RuntimePreflightError>,
}

pub(super) fn handle_server_command<FManage, FParse, FExecute>(
    args: &[String],
    try_manage: FManage,
    parse: FParse,
    execute: FExecute,
) where
    FManage: Fn(&[String]) -> Result<bool, String>,
    FParse: Fn(&[String]) -> Result<CliServerCommand, String>,
    FExecute: Fn(CliServerCommand) -> Result<(), ServerCommandError>,
{
    match run_server_command_with(args, try_manage, parse, execute) {
        Ok(ServerCommandOutcome::ManagedHandled) => {
            std::process::exit(0);
        }
        Ok(ServerCommandOutcome::CreateExecuted) => {
            std::process::exit(0);
        }
        Ok(ServerCommandOutcome::PrintHelpRequested) => {
            print_server_help();
            std::process::exit(0);
        }
        Err(err) => match err.kind {
            ServerCommandErrorKind::Manage => {
                trace_cli_error("manage_execute_failed", "", &err.message);
                eprintln!("server 管理命令失败: {}", err.message);
                std::process::exit(2);
            }
            ServerCommandErrorKind::Parse => {
                trace_cli_error("parse_failed", "", &err.message);
                eprintln!("server 参数错误: {}", err.message);
                print_server_help();
                std::process::exit(2);
            }
            ServerCommandErrorKind::Execute => {
                trace_cli_error("execute_failed", "", &err.message);
                eprintln!("server 命令失败: {}", err.message);
                if let Some(command) = err.parsed_command.as_ref() {
                    if let Some(preflight_error) = err.preflight_error.as_ref() {
                        for line in render_runtime_preflight_failure_hint_lines_from_error(
                            command,
                            preflight_error,
                        ) {
                            eprintln!("{}", line);
                        }
                    } else if let Ok(runtime_kind) = resolve_runtime_kind(command) {
                        for line in render_runtime_preflight_failure_hint_lines(
                            command,
                            runtime_kind,
                            &err.message,
                        ) {
                            eprintln!("{}", line);
                        }
                    }
                }
                std::process::exit(2);
            }
        },
    }
}

#[allow(clippy::result_large_err)]
pub(super) fn run_server_command_with<FManage, FParse, FExecute>(
    args: &[String],
    try_manage: FManage,
    parse: FParse,
    execute: FExecute,
) -> Result<ServerCommandOutcome, ServerCommandError>
where
    FManage: Fn(&[String]) -> Result<bool, String>,
    FParse: Fn(&[String]) -> Result<CliServerCommand, String>,
    FExecute: Fn(CliServerCommand) -> Result<(), ServerCommandError>,
{
    match try_manage(args) {
        Ok(true) => return Ok(ServerCommandOutcome::ManagedHandled),
        Ok(false) => {}
        Err(err) => {
            return Err(ServerCommandError {
                kind: ServerCommandErrorKind::Manage,
                message: err,
                parsed_command: None,
                preflight_error: None,
            });
        }
    }

    trace_cli_action("invoke", &format!("args={}", args.join(" | ")));
    match parse(args) {
        Ok(command) => {
            execute(command)?;
            Ok(ServerCommandOutcome::CreateExecuted)
        }
        Err(err) if err == "__PRINT_HELP__" => Ok(ServerCommandOutcome::PrintHelpRequested),
        Err(err) => Err(ServerCommandError {
            kind: ServerCommandErrorKind::Parse,
            message: err,
            parsed_command: None,
            preflight_error: None,
        }),
    }
}
