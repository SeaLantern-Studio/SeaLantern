#[path = "docker_mounts.rs"]
mod docker_mounts;
#[path = "docker_paths.rs"]
mod docker_paths;
#[path = "docker_preflight.rs"]
mod docker_preflight;
#[path = "docker_request_builder.rs"]
mod docker_request_builder;

pub(super) use docker_paths::resolve_docker_data_dir;
pub(super) use docker_preflight::{
    ensure_docker_environment, preflight_docker_command_mode_support,
    preflight_docker_image_reference, validate_docker_itzg_image_compatibility,
};
pub(super) use docker_request_builder::{
    build_docker_create_request, parse_command_mode, resolve_requested_docker_image,
};

#[cfg(test)]
use crate::utils::cli::server_args::CliServerCommand;
#[cfg(test)]
use crate::utils::cli::server_ports::PreparedPorts;
#[cfg(test)]
use docker_mounts::{parse_docker_volume_mount, parse_published_port};
#[cfg(test)]
use docker_paths::map_container_visible_path_to_docker_host_path;
#[cfg(test)]
use docker_preflight::{
    preflight_docker_command_mode_support_from_outputs_for_tests,
    preflight_docker_image_reference_from_outputs_for_tests,
};
#[cfg(test)]
use docker_request_builder::{format_memory_env_value, parse_docker_backend};

#[derive(Debug, Clone, Copy)]
pub(super) struct DockerCreateDefaults {
    pub default_max_memory_mb: u32,
    pub default_min_memory_mb: u32,
}

const DEFAULT_DOCKER_IMAGE: &str = "itzg/minecraft-server";
pub(super) const DEFAULT_DOCKER_IMAGE_TAG: &str = "latest";

#[cfg(test)]
fn build_docker_request_after_preflight_for_tests(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    defaults: DockerCreateDefaults,
    local_output: &std::process::Output,
    manifest_output: &std::process::Output,
) -> Result<crate::models::server::CreateDockerItzgServerRequest, String> {
    build_docker_request_after_facade_preflight_with_stdio_probe_for_tests(
        command,
        resolved_name,
        ports,
        defaults,
        local_output,
        manifest_output,
        |_image_ref| Ok(()),
    )
}

#[cfg(test)]
fn build_docker_request_after_facade_preflight_with_stdio_probe_for_tests<F>(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    defaults: DockerCreateDefaults,
    local_output: &std::process::Output,
    manifest_output: &std::process::Output,
    ensure_stdio_support: F,
) -> Result<crate::models::server::CreateDockerItzgServerRequest, String>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    let (image, image_tag) = resolve_requested_docker_image(command)?;
    validate_docker_itzg_image_compatibility(&image)?;
    preflight_docker_image_reference_from_outputs_for_tests(
        &image,
        &image_tag,
        local_output,
        manifest_output,
    )?;

    let command_mode = parse_command_mode(command.command_mode.as_deref())?;
    preflight_docker_command_mode_support_from_outputs_for_tests(
        &image,
        &image_tag,
        &command_mode,
        local_output,
        manifest_output,
        ensure_stdio_support,
    )?;

    build_docker_create_request(command, resolved_name, ports, defaults)
}

#[cfg(test)]
fn preflight_docker_mode_from_outputs_for_tests<F>(
    command: &CliServerCommand,
    local_output: &std::process::Output,
    manifest_output: &std::process::Output,
    ensure_stdio_support: F,
) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    let (image, image_tag) = resolve_requested_docker_image(command)?;
    let command_mode = parse_command_mode(command.command_mode.as_deref())?;
    preflight_docker_command_mode_support_from_outputs_for_tests(
        &image,
        &image_tag,
        &command_mode,
        local_output,
        manifest_output,
        ensure_stdio_support,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_docker_create_request,
        build_docker_request_after_facade_preflight_with_stdio_probe_for_tests,
        build_docker_request_after_preflight_for_tests, format_memory_env_value,
        map_container_visible_path_to_docker_host_path, parse_command_mode, parse_docker_backend,
        parse_docker_volume_mount, parse_published_port,
        preflight_docker_mode_from_outputs_for_tests, resolve_docker_data_dir,
        resolve_requested_docker_image, validate_docker_itzg_image_compatibility,
        DockerCreateDefaults, DEFAULT_DOCKER_IMAGE_TAG,
    };
    use crate::models::server::{DockerBackendKind, DockerCommandMode};
    use crate::utils::cli::server_args::CliServerCommand;
    use crate::utils::cli::server_ports::PreparedPorts;
    use once_cell::sync::Lazy;
    use std::cell::Cell;
    use std::path::Path;
    use std::process::Output;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    struct EnvVarGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    fn sample_defaults() -> DockerCreateDefaults {
        DockerCreateDefaults {
            default_max_memory_mb: 4096,
            default_min_memory_mb: 2048,
        }
    }

    fn output_with_status(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: super::exit_status_from_raw(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    fn local_success_output() -> Output {
        output_with_status(0, "[{\"Id\":\"sha256:abc\"}]", "")
    }

    fn manifest_success_output() -> Output {
        output_with_status(0, "{\"schemaVersion\":2}", "")
    }

    fn failed_output(stderr: &str) -> Output {
        output_with_status(1, "", stderr)
    }

    fn failed_output_with_stdout(stdout: &str) -> Output {
        output_with_status(1, stdout, "")
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, original }
        }

        fn remove(key: &'static str) -> Self {
            let original = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn parse_server_command_defaults_docker_command_mode_to_rcon() {
        let command = CliServerCommand::default();
        assert_eq!(
            parse_command_mode(command.command_mode.as_deref()).unwrap(),
            DockerCommandMode::Rcon
        );
    }

    #[test]
    fn parse_docker_backend_accepts_engine_api_aliases() {
        assert_eq!(parse_docker_backend(Some("engine_api")).unwrap(), DockerBackendKind::EngineApi);
        assert_eq!(parse_docker_backend(Some("engine-api")).unwrap(), DockerBackendKind::EngineApi);
    }

    #[test]
    fn parse_docker_backend_rejects_unknown_value() {
        let err = parse_docker_backend(Some("podman")).expect_err("unknown backend should fail");
        assert!(err.contains("docker backend"));
        assert!(err.contains("podman"));
    }

    #[test]
    fn parse_command_mode_accepts_stdio_aliases() {
        assert_eq!(parse_command_mode(Some("stdio")).unwrap(), DockerCommandMode::DockerStdio);
        assert_eq!(
            parse_command_mode(Some("docker-stdio")).unwrap(),
            DockerCommandMode::DockerStdio
        );
    }

    #[test]
    fn parse_command_mode_rejects_unknown_value() {
        let err = parse_command_mode(Some("pty")).expect_err("unknown command mode should fail");

        assert!(err.contains("docker command mode"));
        assert!(err.contains("pty"));
    }

    #[test]
    fn resolve_requested_docker_image_uses_default_image_and_tag_by_default() {
        let command = CliServerCommand::default();

        let (image, tag) =
            resolve_requested_docker_image(&command).expect("default image should resolve");

        assert_eq!(image, "itzg/minecraft-server");
        assert_eq!(tag, DEFAULT_DOCKER_IMAGE_TAG);
    }

    #[test]
    fn build_docker_create_request_defaults_to_rcon_transport() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("docker request should build");

        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert!(request.runtime.rcon.is_some());
        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("2G"));
        assert_eq!(request.runtime.env.get("ENABLE_RCON").map(String::as_str), Some("true"));
        assert_eq!(request.runtime.env.get("RCON_PORT").map(String::as_str), Some("25575"));
        assert!(request.runtime.env.get("RCON_PASSWORD").is_some());
        assert_eq!(request.runtime.extra_ports.len(), 1);
        assert_eq!(request.runtime.extra_ports[0].container_port, 25575);
    }

    #[test]
    fn build_docker_create_request_avoids_game_and_web_ports_for_default_rcon_port() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-rcon-avoid".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25575, web_port: Some(25576) };

        let request =
            build_docker_create_request(&command, "paper-rcon-avoid", &ports, sample_defaults())
                .expect("docker request should avoid reserved game/web ports for rcon");

        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert_eq!(request.runtime.extra_ports.len(), 1);
        assert!(request.runtime.extra_ports[0].host_port >= 25577);
        assert_eq!(
            request.runtime.rcon.as_ref().map(|rcon| rcon.port),
            Some(request.runtime.extra_ports[0].host_port)
        );
    }

    #[test]
    fn build_docker_create_request_can_infer_core_and_mc_from_folder_name() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");

        let command = CliServerCommand {
            runtime: Some("docker".to_string()),
            folder: Some(folder.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("docker request should infer metadata from folder");

        assert_eq!(request.core_type, "paper");
        assert_eq!(request.mc_version, "1.21.1");
        assert_eq!(request.runtime.version, "1.21.1");
        assert_eq!(request.runtime.data_dir_mount, folder.to_string_lossy().to_string());
    }

    #[test]
    fn build_docker_create_request_prefers_explicit_data_dir_over_folder_mount_source() {
        let command = CliServerCommand {
            runtime: Some("docker".to_string()),
            folder: Some("E:/servers/paper-prod-1.21.1".to_string()),
            data_dir: Some("E:/docker/explicit".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("explicit data dir should still win");

        assert_eq!(request.runtime.data_dir_mount, "E:/docker/explicit");
    }

    #[test]
    fn build_docker_create_request_builds_rcon_ready_runtime_shape() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("java21".to_string()),
            data_dir: Some("E:/docker/paper".to_string()),
            container_name: Some("Sea Lantern Docker".to_string()),
            command_mode: Some("rcon".to_string()),
            aliases: vec!["paper_prod".to_string()],
            min_memory_mb: Some(2048),
            max_memory_mb: Some(4096),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("docker request should build");

        assert_eq!(request.runtime.image_tag, "java21");
        assert_eq!(request.runtime.data_dir_mount, "E:/docker/paper");
        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("2G"));
        assert!(request.runtime.rcon.is_some());
    }

    #[test]
    fn build_docker_create_request_locks_facade_runtime_shape() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/facade-lock".to_string()),
            aliases: vec!["paper_prod".to_string(), "main".to_string()],
            docker_env: vec![("ENABLE_AUTOPAUSE".to_string(), "FALSE".to_string())],
            docker_mounts: vec!["E:/plugins:/data/plugins:ro".to_string()],
            docker_publishes: vec!["8123:8123/tcp".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("facade request should build");

        assert_eq!(request.name, "paper-prod");
        assert_eq!(request.aliases, vec!["paper_prod", "main"]);
        assert_eq!(request.core_type, "paper");
        assert_eq!(request.mc_version, "1.21.1");
        assert_eq!(request.port, 25565);

        assert_eq!(request.runtime.image, "itzg/minecraft-server");
        assert_eq!(request.runtime.image_tag, "latest");
        assert_eq!(request.runtime.container_name, "sealantern-paper-prod");
        assert_eq!(request.runtime.type_value, "PAPER");
        assert_eq!(request.runtime.version, "1.21.1");
        assert_eq!(request.runtime.data_dir_mount, "E:/docker/facade-lock");
        assert_eq!(request.runtime.published_game_port, 25565);
        assert_eq!(request.runtime.docker_backend_kind, DockerBackendKind::Cli);
        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);

        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(
            request
                .runtime
                .env
                .get("ENABLE_AUTOPAUSE")
                .map(String::as_str),
            Some("FALSE")
        );

        assert_eq!(request.runtime.volume_mounts.len(), 1);
        assert_eq!(request.runtime.volume_mounts[0].source, "E:/plugins");
        assert_eq!(request.runtime.volume_mounts[0].target, "/data/plugins");
        assert!(request.runtime.volume_mounts[0].read_only);

        assert!(request.runtime.extra_ports.iter().any(|port| {
            port.host_port == 8123 && port.container_port == 8123 && port.protocol == "tcp"
        }));
        assert!(request.runtime.rcon.is_some());
    }

    #[test]
    fn build_docker_create_request_skips_rcon_side_effects_for_stdio_mode() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/paper-stdio".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-stdio", &ports, sample_defaults())
                .expect("docker stdio request should build");

        assert_eq!(request.runtime.command_mode, DockerCommandMode::DockerStdio);
        assert!(request.runtime.extra_ports.is_empty());
        assert!(request.runtime.rcon.is_none());
        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("2G"));
        assert!(request.runtime.env.get("ENABLE_RCON").is_none());
        assert!(request.runtime.env.get("RCON_PASSWORD").is_none());
        assert_eq!(
            request
                .runtime
                .env
                .get("CREATE_CONSOLE_IN_PIPE")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn facade_chain_preserves_local_success_and_builds_runtime_request() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/facade-local-success".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request = build_docker_request_after_preflight_for_tests(
            &command,
            "paper-local-success",
            &ports,
            sample_defaults(),
            &local_success_output(),
            &failed_output("manifest unknown: manifest unknown"),
        )
        .expect("local success should pass preflight and build facade request");

        assert_eq!(request.runtime.image, "itzg/minecraft-server");
        assert_eq!(request.runtime.image_tag, "latest");
        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert!(request.runtime.rcon.is_some());
    }

    #[test]
    fn facade_chain_preserves_remote_resolvable_and_builds_runtime_request() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/facade-remote-success".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request = build_docker_request_after_preflight_for_tests(
            &command,
            "paper-remote-success",
            &ports,
            sample_defaults(),
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &manifest_success_output(),
        )
        .expect("remote resolvable should pass preflight and build facade request");

        assert_eq!(request.runtime.image, "itzg/minecraft-server");
        assert_eq!(request.runtime.image_tag, "latest");
        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert!(request.runtime.rcon.is_some());
    }

    #[test]
    fn facade_chain_preserves_soft_failure_and_still_builds_runtime_request() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/facade-soft-failure".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request = build_docker_request_after_preflight_for_tests(
            &command,
            "paper-soft-failure",
            &ports,
            sample_defaults(),
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
        )
        .expect("soft failure should stay non-blocking through facade request build");

        assert_eq!(request.runtime.image, "itzg/minecraft-server");
        assert_eq!(request.runtime.image_tag, "latest");
        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert!(request.runtime.rcon.is_some());
    }

    #[test]
    fn facade_chain_preserves_hard_failure_and_blocks_request_build() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image_tag: Some("missing".to_string()),
            data_dir: Some("E:/docker/facade-hard-failure".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err = build_docker_request_after_preflight_for_tests(
            &command,
            "paper-hard-failure",
            &ports,
            sample_defaults(),
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:missing",
            ),
            &failed_output_with_stdout("manifest unknown: manifest unknown"),
        )
        .expect_err("hard failure should block facade request build");

        assert!(err.contains("镜像或标签不存在"));
        assert!(err.contains("manifest unknown"));
    }

    #[test]
    fn facade_chain_preserves_remote_resolvable_stdio_skip_behavior() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/facade-remote-stdio".to_string()),
            ..Default::default()
        };
        let probe_calls = Cell::new(0);

        preflight_docker_mode_from_outputs_for_tests(
            &command,
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &manifest_success_output(),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("remote resolvable docker_stdio should skip probe at facade level");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn facade_chain_preserves_soft_failure_stdio_skip_behavior() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/facade-soft-stdio".to_string()),
            ..Default::default()
        };
        let probe_calls = Cell::new(0);

        preflight_docker_mode_from_outputs_for_tests(
            &command,
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("soft failure docker_stdio should skip probe at facade level");

        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn facade_chain_preserves_local_success_stdio_probe_requirement() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/facade-local-stdio".to_string()),
            ..Default::default()
        };
        let probe_calls = Cell::new(0);

        preflight_docker_mode_from_outputs_for_tests(
            &command,
            &local_success_output(),
            &failed_output("manifest unknown: manifest unknown"),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Ok(())
            },
        )
        .expect("local success docker_stdio should still require probe at facade level");

        assert_eq!(probe_calls.get(), 1);
    }

    #[test]
    fn facade_chain_preserves_hard_failure_stdio_blocking_behavior() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            image_tag: Some("missing".to_string()),
            data_dir: Some("E:/docker/facade-hard-stdio".to_string()),
            ..Default::default()
        };
        let probe_calls = Cell::new(0);

        let err = preflight_docker_mode_from_outputs_for_tests(
            &command,
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:missing",
            ),
            &failed_output_with_stdout("manifest unknown: manifest unknown"),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Ok(())
            },
        )
        .expect_err("hard failure docker_stdio should block before probe at facade level");

        assert!(err.contains("镜像或标签不存在"));
        assert_eq!(probe_calls.get(), 0);
    }

    #[test]
    fn facade_chain_preserves_remote_resolvable_stdio_and_builds_runtime_request_without_probe() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/facade-remote-stdio-build".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };
        let probe_calls = Cell::new(0);

        let request = build_docker_request_after_facade_preflight_with_stdio_probe_for_tests(
            &command,
            "paper-remote-stdio-build",
            &ports,
            sample_defaults(),
            &failed_output(
                "Error response from daemon: No such image: itzg/minecraft-server:latest",
            ),
            &manifest_success_output(),
            |_image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                Err("probe should have been skipped".to_string())
            },
        )
        .expect("remote resolvable docker_stdio facade chain should skip probe and still build");

        assert_eq!(probe_calls.get(), 0);
        assert_eq!(request.runtime.command_mode, DockerCommandMode::DockerStdio);
        assert!(request.runtime.rcon.is_none());
        assert!(request.runtime.extra_ports.is_empty());
        assert_eq!(
            request
                .runtime
                .env
                .get("CREATE_CONSOLE_IN_PIPE")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn facade_chain_preserves_local_success_stdio_probe_and_builds_runtime_request() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/facade-local-stdio-build".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };
        let probe_calls = Cell::new(0);

        let request = build_docker_request_after_facade_preflight_with_stdio_probe_for_tests(
            &command,
            "paper-local-stdio-build",
            &ports,
            sample_defaults(),
            &local_success_output(),
            &failed_output("manifest unknown: manifest unknown"),
            |image_ref| {
                probe_calls.set(probe_calls.get() + 1);
                assert_eq!(image_ref, "itzg/minecraft-server:latest");
                Ok(())
            },
        )
        .expect("local success docker_stdio facade chain should probe first and then build");

        assert_eq!(probe_calls.get(), 1);
        assert_eq!(request.runtime.command_mode, DockerCommandMode::DockerStdio);
        assert!(request.runtime.rcon.is_none());
        assert!(request.runtime.extra_ports.is_empty());
        assert_eq!(
            request
                .runtime
                .env
                .get("CREATE_CONSOLE_IN_PIPE")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn build_docker_create_request_applies_custom_env_and_mounts() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-custom".to_string()),
            docker_env: vec![
                ("STOP_DURATION".to_string(), "180".to_string()),
                ("DISABLE_HEALTHCHECK".to_string(), "true".to_string()),
            ],
            docker_mounts: vec!["E:/plugins:/data/plugins:ro".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-custom", &ports, sample_defaults())
                .expect("custom docker request should build");

        assert_eq!(request.runtime.env.get("STOP_DURATION").map(String::as_str), Some("180"));
        assert_eq!(
            request
                .runtime
                .env
                .get("DISABLE_HEALTHCHECK")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(request.runtime.volume_mounts.len(), 1);
        assert_eq!(request.runtime.volume_mounts[0].source, "E:/plugins");
        assert_eq!(request.runtime.volume_mounts[0].target, "/data/plugins");
        assert!(request.runtime.volume_mounts[0].read_only);
    }

    #[test]
    fn build_docker_create_request_supports_extra_publish_ports() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-publish".to_string()),
            docker_publishes: vec!["24454:24454/udp".to_string(), "8123:8123".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-publish", &ports, sample_defaults())
                .expect("custom publish ports should build");

        assert!(request.runtime.extra_ports.iter().any(|port| {
            port.host_port == 24454 && port.container_port == 24454 && port.protocol == "udp"
        }));
        assert!(request.runtime.extra_ports.iter().any(|port| {
            port.host_port == 8123 && port.container_port == 8123 && port.protocol == "tcp"
        }));
    }

    #[test]
    fn build_docker_create_request_rejects_publish_port_conflicting_with_game_port() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-publish-conflict".to_string()),
            docker_publishes: vec!["25565:24454/udp".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err = build_docker_create_request(
            &command,
            "paper-publish-conflict",
            &ports,
            sample_defaults(),
        )
        .expect_err("game port conflict should fail");

        assert!(err.contains("宿主端口 25565"));
        assert!(err.contains("冲突"));
    }

    #[test]
    fn build_docker_create_request_rejects_publish_port_conflicting_with_rcon_port() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-publish-rcon-conflict".to_string()),
            docker_publishes: vec!["25575:24454/tcp".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err = build_docker_create_request(
            &command,
            "paper-publish-rcon-conflict",
            &ports,
            sample_defaults(),
        )
        .expect_err("rcon port conflict should fail");

        assert!(err.contains("宿主端口 25575"));
    }

    #[test]
    fn build_docker_create_request_rejects_rcon_password_file_without_explicit_password() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-rcon-secret".to_string()),
            docker_env: vec![(
                "RCON_PASSWORD_FILE".to_string(),
                "/run/secrets/rcon_pass".to_string(),
            )],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err =
            build_docker_create_request(&command, "paper-rcon-secret", &ports, sample_defaults())
                .expect_err("rcon password file without explicit password should fail");

        assert!(err.contains("RCON_PASSWORD_FILE"));
        assert!(err.contains("RCON_PASSWORD"));
    }

    #[test]
    fn build_docker_create_request_respects_explicit_rcon_env_values() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-rcon-custom".to_string()),
            docker_env: vec![
                ("RCON_PORT".to_string(), "28016".to_string()),
                ("RCON_PASSWORD".to_string(), "top-secret".to_string()),
            ],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-rcon-custom", &ports, sample_defaults())
                .expect("custom rcon env should build");

        assert_eq!(request.runtime.extra_ports[0].container_port, 28016);
        assert_eq!(request.runtime.env.get("RCON_PORT").map(String::as_str), Some("28016"));
        assert_eq!(
            request
                .runtime
                .rcon
                .as_ref()
                .map(|rcon| rcon.password.as_str()),
            Some("top-secret")
        );
    }

    #[test]
    fn parse_docker_volume_mount_supports_windows_host_paths() {
        let mount = parse_docker_volume_mount("E:/plugins:/data/plugins:ro")
            .expect("windows-style docker mount should parse");

        assert_eq!(mount.source, "E:/plugins");
        assert_eq!(mount.target, "/data/plugins");
        assert!(mount.read_only);
    }

    #[test]
    fn parse_published_port_supports_tcp_and_udp() {
        let udp = parse_published_port("24454:24454/udp").expect("udp publish should parse");
        let tcp = parse_published_port("8123:8123").expect("tcp publish should parse");

        assert_eq!(udp.host_port, 24454);
        assert_eq!(udp.container_port, 24454);
        assert_eq!(udp.protocol, "udp");
        assert_eq!(tcp.protocol, "tcp");
    }

    #[test]
    fn resolve_requested_docker_image_supports_embedded_tag() {
        let command = CliServerCommand {
            image: Some("itzg/minecraft-server:java21".to_string()),
            ..Default::default()
        };

        let (image, tag) =
            resolve_requested_docker_image(&command).expect("embedded tag should resolve");

        assert_eq!(image, "itzg/minecraft-server");
        assert_eq!(tag, "java21");
    }

    #[test]
    fn build_docker_create_request_supports_full_image_reference_without_separate_image_tag() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("registry.local:5000/itzg/minecraft-server:java21".to_string()),
            data_dir: Some("E:/docker/full-image-ref".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-full-image", &ports, sample_defaults())
                .expect("full image ref should resolve for docker request");

        assert_eq!(request.runtime.image, "registry.local:5000/itzg/minecraft-server");
        assert_eq!(request.runtime.image_tag, "java21");
    }

    #[test]
    fn validate_docker_itzg_image_compatibility_accepts_default_and_private_mirror_shapes() {
        validate_docker_itzg_image_compatibility("itzg/minecraft-server")
            .expect("official image should be accepted");
        validate_docker_itzg_image_compatibility("registry.local:5000/itzg/minecraft-server")
            .expect("private mirror with minecraft-server suffix should be accepted");
        validate_docker_itzg_image_compatibility("minecraft-server")
            .expect("bare minecraft-server image name should be accepted");
    }

    #[test]
    fn validate_docker_itzg_image_compatibility_rejects_non_minecraft_runtime_images() {
        let err = validate_docker_itzg_image_compatibility("naloveyuki/liteyukibot-web")
            .expect_err("non minecraft runtime image should be rejected");

        assert!(err.contains("liteyukibot-web"));
        assert!(err.contains("minecraft-server"));
    }

    #[test]
    fn resolve_docker_data_dir_prefers_explicit_data_dir() {
        let command = CliServerCommand {
            data_dir: Some("E:/docker/explicit".to_string()),
            ..Default::default()
        };

        let resolved = resolve_docker_data_dir(&command, "paper-docker")
            .expect("explicit data dir should win");
        assert_eq!(resolved, "E:/docker/explicit");
    }

    #[test]
    fn resolve_docker_data_dir_falls_back_to_folder_mount_source() {
        let command = CliServerCommand {
            folder: Some("E:/servers/paper-prod".to_string()),
            ..Default::default()
        };

        let resolved =
            resolve_docker_data_dir(&command, "paper-docker").expect("folder should be reused");

        assert_eq!(resolved, "E:/servers/paper-prod");
    }

    #[test]
    fn resolve_docker_data_dir_errors_for_container_like_default_without_mapping() {
        let _env_lock = ENV_LOCK.lock().expect("env lock should acquire");
        let _headless_guard = EnvVarGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _host_guard = EnvVarGuard::remove("SEALANTERN_SERVERS_HOST_ROOT");
        let _container_guard = EnvVarGuard::remove("SEALANTERN_SERVERS_CONTAINER_ROOT");

        let command = CliServerCommand::default();
        let err = resolve_docker_data_dir(&command, "paper-docker")
            .expect_err("container-like env without mapping should fail fast");

        assert!(err.contains("SEALANTERN_SERVERS_CONTAINER_ROOT"));
        assert!(err.contains("--data-dir"));
        assert!(!err.contains("0.0.0.0"));
    }

    #[test]
    fn resolve_docker_data_dir_uses_mapped_default_when_container_mapping_exists() {
        let _env_lock = ENV_LOCK.lock().expect("env lock should acquire");
        let _headless_guard = EnvVarGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _host_guard =
            EnvVarGuard::set("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        let _container_guard =
            EnvVarGuard::set("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");

        let command = CliServerCommand::default();
        let resolved = resolve_docker_data_dir(&command, "paper-docker")
            .expect("mapped container env should resolve host path");

        assert!(resolved.replace('\\', "/").ends_with("/paper-docker"));
        assert!(resolved
            .replace('\\', "/")
            .starts_with("E:/srv/sealantern/servers/"));
    }

    #[test]
    fn default_docker_image_tag_matches_latest() {
        assert_eq!(DEFAULT_DOCKER_IMAGE_TAG, "latest");
    }

    #[test]
    fn map_container_visible_path_to_docker_host_path_translates_when_roots_match() {
        std::env::set_var("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        std::env::set_var("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");

        let mapped = map_container_visible_path_to_docker_host_path(Path::new(
            "/app/data/servers/paper-docker",
        ))
        .expect("path should translate when roots match");

        assert_eq!(mapped.replace('\\', "/"), "E:/srv/sealantern/servers/paper-docker");

        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");
    }

    #[test]
    fn map_container_visible_path_to_docker_host_path_returns_none_without_mapping() {
        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");

        assert!(map_container_visible_path_to_docker_host_path(Path::new(
            "/app/data/servers/paper"
        ))
        .is_none());
    }

    #[test]
    fn format_memory_env_value_prefers_gigabytes_when_evenly_divisible() {
        assert_eq!(format_memory_env_value(2048), "2G");
        assert_eq!(format_memory_env_value(1536), "1536M");
    }

    #[test]
    fn build_docker_create_request_applies_default_memory_envs_when_cli_omits_memory_flags() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/default-memory".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request = build_docker_create_request(
            &command,
            "paper-default-memory",
            &ports,
            DockerCreateDefaults {
                default_max_memory_mb: 3072,
                default_min_memory_mb: 1536,
            },
        )
        .expect("default docker memory envs should be applied");

        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("3G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("3G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("1536M"));
    }
}

#[cfg(test)]
fn exit_status_from_raw(code: i32) -> std::process::ExitStatus {
    #[cfg(windows)]
    {
        std::os::windows::process::ExitStatusExt::from_raw(code as u32)
    }
    #[cfg(unix)]
    {
        std::os::unix::process::ExitStatusExt::from_raw(code)
    }
}
