mod docker_support;
mod java_support;
#[cfg(test)]
mod local_folder_inspection;
mod local_request_support;
#[cfg(test)]
mod local_startup_support;
#[cfg(test)]
mod metadata_support;

use std::path::Path;

use crate::models::server::ServerInstance;
use crate::services::global;
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::{trace_cli_action, CliServerRuntimeKind};
use sea_lantern_server_local_setup_core::{
    inspect_local_folder, inspect_local_folder_checked, LocalFolderInspection,
};

use self::docker_support::{
    build_docker_create_request, ensure_docker_environment, preflight_docker_command_mode_support,
    preflight_docker_image_reference, resolve_docker_data_dir, resolve_requested_docker_image,
    validate_docker_itzg_image_compatibility, DockerCreateDefaults,
};
use self::java_support::resolve_java_path;
use self::local_request_support::{
    build_local_attach_request, build_local_create_request, trace_local_attach_request,
    trace_local_create_request, LocalDefaults,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RuntimePreflightStage {
    LocalJava,
    DockerEnvironment,
    DockerBackend,
    DockerDataDir,
    DockerImage,
}

impl RuntimePreflightStage {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::LocalJava => "local_java",
            Self::DockerEnvironment => "docker_environment",
            Self::DockerBackend => "docker_backend",
            Self::DockerDataDir => "docker_data_dir",
            Self::DockerImage => "docker_image",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimePreflightError {
    pub(super) runtime_kind: CliServerRuntimeKind,
    pub(super) stage: RuntimePreflightStage,
    pub(super) message: String,
    pub(super) detail: Option<String>,
}

impl RuntimePreflightError {
    pub(super) fn new(
        runtime_kind: CliServerRuntimeKind,
        stage: RuntimePreflightStage,
        message: String,
        detail: Option<String>,
    ) -> Self {
        Self { runtime_kind, stage, message, detail }
    }
}

impl std::fmt::Display for RuntimePreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(detail) = self
            .detail
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            write!(
                f,
                "runtime preflight failed: runtime={} stage={} message={} detail={}",
                self.runtime_kind.as_runtime_label(),
                self.stage.as_str(),
                self.message,
                detail
            )
        } else {
            write!(
                f,
                "runtime preflight failed: runtime={} stage={} message={}",
                self.runtime_kind.as_runtime_label(),
                self.stage.as_str(),
                self.message
            )
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RuntimeIntentAnalysis {
    local_hints: Vec<String>,
    docker_hints: Vec<String>,
}

impl RuntimeIntentAnalysis {
    fn has_local_hints(&self) -> bool {
        !self.local_hints.is_empty()
    }

    fn has_docker_hints(&self) -> bool {
        !self.docker_hints.is_empty()
    }

    fn describe(&self) -> String {
        format!(
            "local_hints=[{}] docker_hints=[{}]",
            self.local_hints.join(","),
            self.docker_hints.join(",")
        )
    }
}

pub(super) fn resolve_runtime_kind(
    command: &CliServerCommand,
) -> Result<CliServerRuntimeKind, String> {
    let analysis = analyze_runtime_intent(command);
    let runtime = command
        .runtime
        .as_deref()
        .unwrap_or("auto")
        .trim()
        .to_ascii_lowercase();
    match runtime.as_str() {
        "auto" => {
            trace_cli_action("runtime_auto_intent", &analysis.describe());
            if analysis.has_local_hints() && analysis.has_docker_hints() {
                Err(render_runtime_hint_conflict("auto", &analysis))
            } else if analysis.has_docker_hints() {
                Ok(CliServerRuntimeKind::Docker)
            } else {
                Ok(CliServerRuntimeKind::Local)
            }
        }
        "local" => {
            if analysis.has_docker_hints() {
                Err(render_runtime_hint_conflict("local", &analysis))
            } else {
                Ok(CliServerRuntimeKind::Local)
            }
        }
        "docker" | "docker_itzg" => {
            let hard_local_hints = collect_hard_local_runtime_hints(command);
            if !hard_local_hints.is_empty() {
                Err(format!(
                    "--runtime docker 与本地启动参数冲突: {}。请移除这些参数，或改用 --runtime local",
                    hard_local_hints.join(", ")
                ))
            } else {
                Ok(CliServerRuntimeKind::Docker)
            }
        }
        other => Err(format!("不支持的 runtime: {}", other)),
    }
}

fn analyze_runtime_intent(command: &CliServerCommand) -> RuntimeIntentAnalysis {
    let mut analysis = RuntimeIntentAnalysis::default();

    push_hint_if_present(&mut analysis.local_hints, command.jar_path.as_deref(), "--jar");
    push_hint_if_present(&mut analysis.local_hints, command.java_path.as_deref(), "--java");
    if command.java_from_env_only {
        analysis.local_hints.push("--J".to_string());
    }
    push_hint_if_present(&mut analysis.local_hints, command.entry.as_deref(), "--entry");
    push_hint_if_present(&mut analysis.local_hints, command.startup_mode.as_deref(), "--startup");

    push_hint_if_present(&mut analysis.docker_hints, command.image.as_deref(), "--image");
    push_hint_if_present(&mut analysis.docker_hints, command.image_tag.as_deref(), "--image-tag");
    push_hint_if_present(&mut analysis.docker_hints, command.data_dir.as_deref(), "--data-dir");
    push_hint_if_present(
        &mut analysis.docker_hints,
        command.container_name.as_deref(),
        "--container-name",
    );
    push_hint_if_present(
        &mut analysis.docker_hints,
        command.docker_backend.as_deref(),
        "--docker-backend",
    );
    push_hint_if_present(
        &mut analysis.docker_hints,
        command.command_mode.as_deref(),
        "--command-mode",
    );
    if !command.docker_env.is_empty() {
        analysis.docker_hints.push("--env".to_string());
    }
    if !command.docker_mounts.is_empty() {
        analysis.docker_hints.push("--mount".to_string());
    }
    if !command.docker_publishes.is_empty() {
        analysis.docker_hints.push("--publish".to_string());
    }

    if let Some(folder) = command.folder.as_deref() {
        let folder_path = Path::new(folder);
        if folder_path.exists() && folder_path.is_dir() {
            let inspection = inspect_local_folder(folder_path);
            if inspection.is_attachable() {
                analysis
                    .local_hints
                    .push("--folder(existing-local-server)".to_string());
            }
        }
    }

    analysis
}

fn collect_hard_local_runtime_hints(command: &CliServerCommand) -> Vec<String> {
    let mut hints = Vec::new();
    push_hint_if_present(&mut hints, command.jar_path.as_deref(), "--jar");
    push_hint_if_present(&mut hints, command.java_path.as_deref(), "--java");
    if command.java_from_env_only {
        hints.push("--J".to_string());
    }
    push_hint_if_present(&mut hints, command.entry.as_deref(), "--entry");
    push_hint_if_present(&mut hints, command.startup_mode.as_deref(), "--startup");
    hints
}

fn push_hint_if_present(target: &mut Vec<String>, value: Option<&str>, label: &str) {
    if value.map(str::trim).is_some_and(|value| !value.is_empty()) {
        target.push(label.to_string());
    }
}

fn render_runtime_hint_conflict(runtime: &str, analysis: &RuntimeIntentAnalysis) -> String {
    format!(
        "runtime={} 检测到 local 与 docker 参数同时存在: local=[{}], docker=[{}]。请显式指定 --runtime local|docker，或移除冲突参数",
        runtime,
        analysis.local_hints.join(", "),
        analysis.docker_hints.join(", ")
    )
}

pub(super) fn ensure_memory_bounds(command: &CliServerCommand) -> Result<(), String> {
    let settings = global::settings_manager().get();
    let min_memory_mb = command.min_memory_mb.unwrap_or(settings.default_min_memory);
    let max_memory_mb = command.max_memory_mb.unwrap_or(settings.default_max_memory);

    let mut system = sysinfo::System::new_all();
    system.refresh_memory();
    let physical_memory_mb = (system.total_memory() / 1024 / 1024) as u32;

    validate_memory_bounds(min_memory_mb, max_memory_mb, physical_memory_mb)?;

    Ok(())
}

pub(super) fn preflight_runtime_requirements(
    command: &mut CliServerCommand,
    runtime_kind: CliServerRuntimeKind,
) -> Result<(), String> {
    preflight_runtime_requirements_detailed(command, runtime_kind).map_err(|err| err.to_string())
}

pub(super) fn preflight_runtime_requirements_detailed(
    command: &mut CliServerCommand,
    runtime_kind: CliServerRuntimeKind,
) -> Result<(), RuntimePreflightError> {
    let settings = global::settings_manager().get();

    match runtime_kind {
        CliServerRuntimeKind::Local => {
            let resolved_java =
                resolve_java_path(command, &settings.default_java_path).map_err(|message| {
                    RuntimePreflightError::new(
                        CliServerRuntimeKind::Local,
                        RuntimePreflightStage::LocalJava,
                        message,
                        None,
                    )
                })?;
            trace_cli_action(
                "runtime_preflight_local_java_cache",
                &format!("java_path={}", resolved_java),
            );
            command.java_path = Some(resolved_java);
            command.java_path_prevalidated = true;
            Ok(())
        }
        CliServerRuntimeKind::Docker => {
            ensure_supported_docker_backend(command).map_err(|message| {
                RuntimePreflightError::new(
                    CliServerRuntimeKind::Docker,
                    RuntimePreflightStage::DockerBackend,
                    message,
                    command
                        .docker_backend
                        .as_ref()
                        .map(|value| format!("requested_backend={}", value.trim())),
                )
            })?;

            let target_name = command
                .name
                .as_deref()
                .or(command.positional_name.as_deref())
                .or(command.server_tag.as_deref())
                .unwrap_or("server");
            let resolved_data_dir =
                resolve_docker_data_dir(command, target_name).map_err(|message| {
                    RuntimePreflightError::new(
                        CliServerRuntimeKind::Docker,
                        RuntimePreflightStage::DockerDataDir,
                        message,
                        Some(format!("target_name={}", target_name)),
                    )
                })?;
            trace_cli_action(
                "runtime_preflight_docker_data_dir",
                &format!("target_name={} data_dir={}", target_name, resolved_data_dir),
            );

            let (image, image_tag) =
                resolve_requested_docker_image(command).map_err(|message| {
                    RuntimePreflightError::new(
                        CliServerRuntimeKind::Docker,
                        RuntimePreflightStage::DockerImage,
                        message,
                        command
                            .image
                            .as_ref()
                            .map(|value| format!("requested_image={}", value)),
                    )
                })?;
            validate_docker_itzg_image_compatibility(&image).map_err(|message| {
                RuntimePreflightError::new(
                    CliServerRuntimeKind::Docker,
                    RuntimePreflightStage::DockerImage,
                    message,
                    Some(format!("image={}:{}", image, image_tag)),
                )
            })?;
            ensure_docker_environment().map_err(|message| {
                RuntimePreflightError::new(
                    CliServerRuntimeKind::Docker,
                    RuntimePreflightStage::DockerEnvironment,
                    message,
                    None,
                )
            })?;
            preflight_docker_image_reference(&image, &image_tag).map_err(|message| {
                RuntimePreflightError::new(
                    CliServerRuntimeKind::Docker,
                    RuntimePreflightStage::DockerImage,
                    message,
                    Some(format!("image={}:{}", image, image_tag)),
                )
            })?;
            let command_mode = self::docker_support::parse_command_mode(
                command.command_mode.as_deref(),
            )
            .map_err(|message| {
                RuntimePreflightError::new(
                    CliServerRuntimeKind::Docker,
                    RuntimePreflightStage::DockerImage,
                    message,
                    Some(format!("image={}:{}", image, image_tag)),
                )
            })?;
            preflight_docker_command_mode_support(&image, &image_tag, &command_mode).map_err(
                |message| {
                    RuntimePreflightError::new(
                        CliServerRuntimeKind::Docker,
                        RuntimePreflightStage::DockerImage,
                        message,
                        Some(format!(
                            "image={}:{} command_mode={}",
                            image,
                            image_tag,
                            command_mode.as_str()
                        )),
                    )
                },
            )?;
            Ok(())
        }
    }
}

fn ensure_supported_docker_backend(command: &CliServerCommand) -> Result<(), String> {
    let requested_backend = command
        .docker_backend
        .as_deref()
        .unwrap_or("cli")
        .trim()
        .to_ascii_lowercase();

    if matches!(requested_backend.as_str(), "" | "cli") {
        return Ok(());
    }

    Err(format!(
        "当前仅支持 --docker-backend cli；{} 尚未接入真实运行链路，请移除该参数或改为 --docker-backend cli",
        requested_backend
    ))
}

fn validate_memory_bounds(
    min_memory_mb: u32,
    max_memory_mb: u32,
    physical_memory_mb: u32,
) -> Result<(), String> {
    if min_memory_mb == 0 || max_memory_mb == 0 {
        return Err("--min/--max 不能为 0".to_string());
    }

    if min_memory_mb > max_memory_mb {
        return Err("--min 不能大于 --max".to_string());
    }

    if physical_memory_mb > 0 && max_memory_mb > physical_memory_mb {
        return Err(format!(
            "--max={}MB 超过当前物理内存约 {}MB",
            max_memory_mb, physical_memory_mb
        ));
    }

    Ok(())
}

pub(super) fn create_or_attach_local_server(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
) -> Result<ServerInstance, String> {
    let settings = global::settings_manager().get();
    let defaults = LocalDefaults {
        default_java_path: &settings.default_java_path,
        default_max_memory_mb: settings.default_max_memory,
        default_min_memory_mb: settings.default_min_memory,
    };

    if let Some(folder) = command.folder.as_deref() {
        let folder_path = Path::new(folder);
        if folder_path.exists() {
            if !folder_path.is_dir() {
                return Err(format!("--folder 指定目录不存在或不是文件夹: {}", folder));
            }

            let inspection = inspect_local_folder_checked(folder_path)
                .map_err(|error| format!("无法检查本地服务器目录 {}: {}", folder, error))?;
            trace_cli_action(
                "local_folder_inspection",
                &format!("name={} folder={} {}", resolved_name, folder, inspection.describe()),
            );

            if inspection.is_attachable() {
                trace_cli_action(
                    "local_folder_resolution",
                    &format!("name={} mode=attach folder={}", resolved_name, folder),
                );
                return attach_existing_local_server(
                    command,
                    resolved_name,
                    ports,
                    folder,
                    &defaults,
                    &inspection,
                );
            }
        }

        trace_cli_action(
            "local_folder_resolution",
            &format!("name={} mode=create folder={}", resolved_name, folder),
        );
    }

    let req = build_local_create_request(command, resolved_name, ports, &defaults)?;
    trace_local_create_request(resolved_name, &req);

    global::server_manager().create_server(req)
}

pub(super) fn create_or_attach_docker_server(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
) -> Result<ServerInstance, String> {
    let settings = global::settings_manager().get();
    let req = build_docker_create_request(
        command,
        resolved_name,
        ports,
        DockerCreateDefaults {
            default_max_memory_mb: settings.default_max_memory,
            default_min_memory_mb: settings.default_min_memory,
        },
    )?;

    global::server_manager().create_docker_itzg_server(req)
}

fn attach_existing_local_server(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    folder: &str,
    defaults: &LocalDefaults<'_>,
    inspection: &LocalFolderInspection,
) -> Result<ServerInstance, String> {
    let request =
        build_local_attach_request(command, resolved_name, ports, folder, defaults, inspection)?;
    trace_local_attach_request(resolved_name, &request, ports);

    global::server_manager().add_existing_server(request)
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_memory_bounds, preflight_runtime_requirements, resolve_docker_data_dir,
        resolve_runtime_kind, validate_memory_bounds,
    };
    use crate::utils::cli::server_args::{parse_server_command, CliServerCommand};
    use crate::utils::cli::server_shared::CliServerRuntimeKind;
    use crate::utils::cli::server_test_support::{lock_env, EnvGuard};
    use tempfile::tempdir;

    #[test]
    fn resolve_runtime_kind_prefers_docker_when_image_present() {
        let args = vec![
            "paper-docker".to_string(),
            "--mc".to_string(),
            "1.21.1".to_string(),
            "--core".to_string(),
            "paper".to_string(),
            "--image".to_string(),
            "itzg/minecraft-server".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(resolve_runtime_kind(&parsed).unwrap(), CliServerRuntimeKind::Docker);
    }

    #[test]
    fn resolve_runtime_kind_prefers_docker_when_image_tag_present() {
        let args = vec![
            "paper-docker".to_string(),
            "--mc".to_string(),
            "1.21.1".to_string(),
            "--core".to_string(),
            "paper".to_string(),
            "--image-tag".to_string(),
            "latest".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(resolve_runtime_kind(&parsed).unwrap(), CliServerRuntimeKind::Docker);
    }

    #[test]
    fn resolve_runtime_kind_prefers_docker_when_command_mode_present() {
        let args = vec![
            "paper-docker".to_string(),
            "--mc".to_string(),
            "1.21.1".to_string(),
            "--core".to_string(),
            "paper".to_string(),
            "--command-mode".to_string(),
            "rcon".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(resolve_runtime_kind(&parsed).unwrap(), CliServerRuntimeKind::Docker);
    }

    #[test]
    fn resolve_runtime_kind_prefers_docker_when_container_name_present() {
        let args = vec![
            "paper-docker".to_string(),
            "--mc".to_string(),
            "1.21.1".to_string(),
            "--core".to_string(),
            "paper".to_string(),
            "--container-name".to_string(),
            "sealantern-paper".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(resolve_runtime_kind(&parsed).unwrap(), CliServerRuntimeKind::Docker);
    }

    #[test]
    fn resolve_runtime_kind_prefers_docker_when_env_mount_or_publish_present() {
        let env_command = CliServerCommand {
            docker_env: vec![("STOP_DURATION".to_string(), "180".to_string())],
            ..Default::default()
        };
        let mount_command = CliServerCommand {
            docker_mounts: vec!["E:/plugins:/data/plugins:ro".to_string()],
            ..Default::default()
        };
        let publish_command = CliServerCommand {
            docker_publishes: vec!["24454:24454/udp".to_string()],
            ..Default::default()
        };

        assert_eq!(resolve_runtime_kind(&env_command).unwrap(), CliServerRuntimeKind::Docker);
        assert_eq!(resolve_runtime_kind(&mount_command).unwrap(), CliServerRuntimeKind::Docker);
        assert_eq!(resolve_runtime_kind(&publish_command).unwrap(), CliServerRuntimeKind::Docker);
    }

    #[test]
    fn resolve_runtime_kind_defaults_to_local_without_docker_hints() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(resolve_runtime_kind(&parsed).unwrap(), CliServerRuntimeKind::Local);
    }

    #[test]
    fn resolve_runtime_kind_prefers_local_when_jar_hint_is_present() {
        let args = vec![
            "fabric-1.20.1".to_string(),
            "--mc".to_string(),
            "1.20.1".to_string(),
            "--core".to_string(),
            "fabric".to_string(),
            "--jar".to_string(),
            "E:/srv/server.jar".to_string(),
        ];

        let parsed = parse_server_command(&args).unwrap();
        assert_eq!(resolve_runtime_kind(&parsed).unwrap(), CliServerRuntimeKind::Local);
    }

    #[test]
    fn resolve_runtime_kind_prefers_local_for_attachable_folder_without_docker_hints() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let command = CliServerCommand {
            folder: Some(folder.to_string_lossy().to_string()),
            ..Default::default()
        };

        assert_eq!(resolve_runtime_kind(&command).unwrap(), CliServerRuntimeKind::Local);
    }

    #[test]
    fn resolve_runtime_kind_rejects_auto_when_local_and_docker_hints_are_mixed() {
        let command = CliServerCommand {
            jar_path: Some("E:/srv/server.jar".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            ..Default::default()
        };

        let err = resolve_runtime_kind(&command).expect_err("mixed auto hints should fail");
        assert!(err.contains("local 与 docker 参数同时存在"));
        assert!(err.contains("--jar"));
        assert!(err.contains("--image"));
    }

    #[test]
    fn resolve_runtime_kind_rejects_explicit_local_when_docker_hints_are_present() {
        let command = CliServerCommand {
            runtime: Some("local".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            ..Default::default()
        };

        let err =
            resolve_runtime_kind(&command).expect_err("local runtime should reject docker hints");
        assert!(err.contains("runtime=local"));
        assert!(err.contains("--image"));
    }

    #[test]
    fn resolve_runtime_kind_rejects_explicit_local_when_publish_hint_is_present() {
        let command = CliServerCommand {
            runtime: Some("local".to_string()),
            docker_publishes: vec!["24454:24454/udp".to_string()],
            ..Default::default()
        };

        let err = resolve_runtime_kind(&command)
            .expect_err("local runtime should reject docker publish hints");
        assert!(err.contains("runtime=local"));
        assert!(err.contains("--publish"));
    }

    #[test]
    fn resolve_runtime_kind_rejects_explicit_docker_when_local_hints_are_present() {
        let command = CliServerCommand {
            runtime: Some("docker".to_string()),
            java_path: Some("C:/Java/bin/java.exe".to_string()),
            ..Default::default()
        };

        let err =
            resolve_runtime_kind(&command).expect_err("docker runtime should reject local hints");
        assert!(err.contains("--runtime docker"));
        assert!(err.contains("--java"));
    }

    #[test]
    fn preflight_runtime_requirements_keeps_explicit_local_java_path() {
        let mut command = CliServerCommand {
            java_path: Some("%env:Path%".to_string()),
            ..Default::default()
        };

        preflight_runtime_requirements(&mut command, CliServerRuntimeKind::Local)
            .expect("local preflight should validate java path");

        let resolved = command
            .java_path
            .as_deref()
            .expect("java path should be cached");
        assert!(resolved.to_ascii_lowercase().contains("java"));
        assert!(command.java_path_prevalidated);
    }

    #[test]
    fn preflight_runtime_requirements_rejects_engine_api_backend_before_create() {
        let mut command = CliServerCommand {
            runtime: Some("docker".to_string()),
            docker_backend: Some("engine_api".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("latest".to_string()),
            data_dir: Some("E:/servers/paper-docker".to_string()),
            ..Default::default()
        };

        let err = preflight_runtime_requirements(&mut command, CliServerRuntimeKind::Docker)
            .expect_err("engine_api backend should be rejected before create");

        assert!(err.contains("--docker-backend cli"));
        assert!(err.contains("engine_api"));
    }

    #[test]
    fn resolve_docker_data_dir_can_fail_before_build_request_in_container_like_env() {
        let _env_lock = lock_env();
        let _headless_guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _host_guard = EnvGuard::remove("SEALANTERN_SERVERS_HOST_ROOT");
        let _container_guard = EnvGuard::remove("SEALANTERN_SERVERS_CONTAINER_ROOT");

        let command = CliServerCommand::default();
        let err = resolve_docker_data_dir(&command, "paper-docker")
            .expect_err("docker data dir preflight should fail without mapping");

        assert!(err.contains("SEALANTERN_SERVERS_CONTAINER_ROOT"));
        assert!(err.contains("--data-dir"));
    }

    #[test]
    fn ensure_memory_bounds_rejects_min_greater_than_max() {
        let command = CliServerCommand {
            min_memory_mb: Some(4096),
            max_memory_mb: Some(2048),
            ..Default::default()
        };

        let err = ensure_memory_bounds(&command).expect_err("min > max should fail");
        assert!(err.contains("--min"));
    }

    #[test]
    fn validate_memory_bounds_rejects_zero_when_only_max_is_provided() {
        let err = validate_memory_bounds(1024, 0, 8192).expect_err("zero max should be rejected");
        assert!(err.contains("不能为 0"));
    }

    #[test]
    fn validate_memory_bounds_rejects_max_above_physical_memory_when_only_max_is_provided() {
        let err = validate_memory_bounds(1024, 4096, 2048)
            .expect_err("max above physical memory should fail");
        assert!(err.contains("物理内存"));
        assert!(err.contains("4096"));
    }

    #[test]
    fn validate_memory_bounds_allows_min_without_explicit_max() {
        validate_memory_bounds(1024, 2048, 2048).expect("valid bounds should pass");
    }

    #[test]
    fn validate_memory_bounds_rejects_effective_min_above_effective_max() {
        let err = validate_memory_bounds(4096, 2048, 8192)
            .expect_err("effective min above max should fail");
        assert!(err.contains("--min"));
    }
}
