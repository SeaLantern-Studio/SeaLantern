use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_entry::{ServerCommandError, ServerCommandErrorKind};
use crate::utils::cli::server_setup::{
    preflight_runtime_requirements_detailed, resolve_runtime_kind,
};
use crate::utils::cli::server_shared::{
    resolve_cli_target_hint, trace_cli_action, trace_cli_error,
};

#[allow(clippy::result_large_err)]
pub(super) fn execute_server_command_for_entry<FExecute>(
    mut command: CliServerCommand,
    execute_server_command: FExecute,
) -> Result<(), ServerCommandError>
where
    FExecute: FnOnce(CliServerCommand) -> Result<(), String>,
{
    let runtime_kind = resolve_runtime_kind(&command).map_err(|message| ServerCommandError {
        kind: ServerCommandErrorKind::Execute,
        message,
        parsed_command: Some(command.clone()),
        preflight_error: None,
    })?;

    if let Err(preflight_error) =
        preflight_runtime_requirements_detailed(&mut command, runtime_kind)
    {
        trace_cli_error(
            "runtime_preflight_failed",
            &format!(
                "runtime={} stage={} target={}",
                preflight_error.runtime_kind.as_runtime_label(),
                preflight_error.stage.as_str(),
                resolve_cli_target_hint(&command)
            ),
            &preflight_error.message,
        );
        return Err(ServerCommandError {
            kind: ServerCommandErrorKind::Execute,
            message: preflight_error.to_string(),
            parsed_command: Some(command),
            preflight_error: Some(preflight_error),
        });
    }

    trace_cli_action(
        "runtime_preflight_ok",
        &format!(
            "runtime={} target={}",
            runtime_kind.as_runtime_label(),
            resolve_cli_target_hint(&command)
        ),
    );
    command.runtime_prevalidated = true;

    execute_server_command(command).map_err(|message| ServerCommandError {
        kind: ServerCommandErrorKind::Execute,
        message,
        parsed_command: None,
        preflight_error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::execute_server_command_for_entry;
    use crate::utils::cli::server_args::CliServerCommand;
    use crate::utils::cli::server_setup::RuntimePreflightStage;
    use crate::utils::cli::server_test_support::{lock_env, EnvGuard};

    #[test]
    fn execute_server_command_for_entry_surfaces_structured_local_preflight_error() {
        let command = CliServerCommand {
            positional_name: Some("fabric-1.20.1".to_string()),
            runtime: Some("local".to_string()),
            java_path: Some("Z:/definitely-missing-java/bin/java.exe".to_string()),
            ..Default::default()
        };

        let err = execute_server_command_for_entry(command, |_| Ok(()))
            .expect_err("expected local preflight failure for invalid explicit Java path");

        let preflight = err.preflight_error.expect("preflight error should exist");
        assert_eq!(preflight.stage, RuntimePreflightStage::LocalJava);
        assert!(preflight.message.contains("--java"));
    }

    #[test]
    fn execute_server_command_for_entry_surfaces_structured_docker_preflight_error() {
        let _env_lock = lock_env();
        let _headless_guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _host_guard = EnvGuard::remove("SEALANTERN_SERVERS_HOST_ROOT");
        let _container_guard = EnvGuard::remove("SEALANTERN_SERVERS_CONTAINER_ROOT");

        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("latest".to_string()),
            ..Default::default()
        };

        let err = execute_server_command_for_entry(command, |_| Ok(()))
            .expect_err("expected docker preflight failure for missing host/container mapping");

        let preflight = err.preflight_error.expect("preflight error should exist");
        assert_eq!(preflight.stage, RuntimePreflightStage::DockerDataDir);
        assert!(preflight
            .message
            .contains("SEALANTERN_SERVERS_CONTAINER_ROOT"));
        assert!(preflight.message.contains("--data-dir"));
    }

    #[test]
    fn execute_server_command_for_entry_rejects_incompatible_docker_runtime_image_before_create() {
        let _env_lock = lock_env();
        let command = CliServerCommand {
            positional_name: Some("paper-docker".to_string()),
            runtime: Some("docker".to_string()),
            image: Some("naloveyuki/liteyukibot-web".to_string()),
            image_tag: Some("latest".to_string()),
            data_dir: Some("E:/docker/paper-docker".to_string()),
            ..Default::default()
        };

        let err = execute_server_command_for_entry(command, |_| Ok(()))
            .expect_err("incompatible docker image should fail in preflight");

        let preflight = err.preflight_error.expect("preflight error should exist");
        assert_eq!(preflight.stage, RuntimePreflightStage::DockerImage);
        assert!(preflight.message.contains("liteyukibot-web"));
        assert!(preflight.message.contains("minecraft-server"));
    }
}
