use super::{DockerContainerState, RuntimeStatusSnapshot, DOCKER_ITZG_RUNTIME_KIND};
use crate::models::server::{DockerItzgRuntimeConfig, ServerInstance, ServerStatus};
use crate::services::server::manager::startup_support::{
    resolve_effective_startup_config_checked, EffectiveStartupConfig,
};
use crate::utils::docker_cli::docker_error_indicates_missing_container;
use sea_lantern_docker_core::{
    build_docker_launch_detail as build_shared_docker_launch_detail,
    build_docker_launch_spec as build_shared_docker_launch_spec, DockerEffectiveLaunchConfig,
};
pub(crate) use sea_lantern_docker_core::{
    build_docker_run_args, DockerLaunchDetail, DockerLaunchSpec,
};
use std::path::Path;
use std::process::Output;

pub(crate) fn resolve_docker_launch_spec(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
    settings: &crate::models::settings::AppSettings,
) -> Result<DockerLaunchSpec, String> {
    let effective = resolve_effective_startup_config_checked(server, settings)?;
    build_shared_docker_launch_spec(runtime, &adapt_effective_launch_config(&effective))
}

pub(crate) fn build_docker_launch_detail(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
    settings: &crate::models::settings::AppSettings,
) -> Result<DockerLaunchDetail, String> {
    let launch_spec = resolve_docker_launch_spec(server, runtime, settings)?;
    Ok(build_shared_docker_launch_detail(
        DOCKER_ITZG_RUNTIME_KIND,
        runtime,
        &launch_spec,
    ))
}

fn adapt_effective_launch_config(
    effective: &EffectiveStartupConfig,
) -> DockerEffectiveLaunchConfig {
    DockerEffectiveLaunchConfig {
        max_memory: effective.max_memory,
        min_memory: effective.min_memory,
        jvm_args: effective.jvm_args.clone(),
        cpu_policy: effective.cpu_policy.clone(),
        jvm_preset: effective.jvm_preset.clone(),
    }
}

pub(super) fn ensure_runtime_path_ready(server: &ServerInstance) -> Result<(), String> {
    let path = Path::new(&server.path);
    std::fs::create_dir_all(path)
        .map_err(|e| format!("创建 Docker 数据目录失败 ({}): {}", path.display(), e))
}

pub(super) fn container_should_clear_starting(state: &DockerContainerState) -> bool {
    state.running
        && state
            .health_status
            .as_deref()
            .map(|status| !status.eq_ignore_ascii_case("starting"))
            .unwrap_or(true)
}

pub(super) fn resolve_managed_status(
    runtime_status: ServerStatus,
    is_starting: bool,
    is_stopping: bool,
) -> ServerStatus {
    if is_stopping {
        ServerStatus::Stopping
    } else if runtime_status == ServerStatus::Running && is_starting {
        ServerStatus::Starting
    } else {
        runtime_status
    }
}

pub(super) fn docker_status_is_not_running(snapshot: &RuntimeStatusSnapshot) -> bool {
    matches!(snapshot.status, ServerStatus::Stopped)
        || (matches!(snapshot.status, ServerStatus::Error)
            && !runtime_detail_indicates_running(snapshot.detail_message.as_deref()))
}

fn runtime_detail_indicates_running(detail: Option<&str>) -> bool {
    detail
        .map(|value| {
            value
                .split_whitespace()
                .any(|part| part.eq_ignore_ascii_case("running=true"))
        })
        .unwrap_or(false)
}

pub(super) fn render_container_error(
    runtime: &DockerItzgRuntimeConfig,
    state: &DockerContainerState,
) -> Option<String> {
    state.error_message.clone().or_else(|| {
        if !state.running && !exit_code_should_be_treated_as_stopped(state.exit_code) {
            Some(format!(
                "Docker 容器已退出: container={}, status={}, exit_code={}",
                runtime.container_name,
                state.status,
                state.exit_code.unwrap_or_default()
            ))
        } else if state.running {
            state
                .health_status
                .as_deref()
                .filter(|status| status.eq_ignore_ascii_case("unhealthy"))
                .map(|_| {
                    format!(
                        "Docker 容器健康检查失败: container={}, health=unhealthy",
                        runtime.container_name
                    )
                })
        } else {
            None
        }
    })
}

pub(super) fn render_container_detail(
    runtime: &DockerItzgRuntimeConfig,
    state: &DockerContainerState,
) -> String {
    let health = state.health_status.as_deref().unwrap_or("none");
    let exit_code = state
        .exit_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!(
        "runtime={} container={} state={} running={} health={} exit_code={} backend={} command_mode={}",
        DOCKER_ITZG_RUNTIME_KIND,
        runtime.container_name,
        state.status,
        state.running,
        health,
        exit_code,
        runtime.docker_backend_kind.as_str(),
        runtime.command_mode.as_str()
    )
}

fn render_container_runtime_state(state: &DockerContainerState) -> String {
    let health = state.health_status.as_deref().unwrap_or("none");
    let exit_code = state
        .exit_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!(
        "state={}, running={}, health={}, exit_code={}",
        state.status, state.running, health, exit_code
    )
}

pub(super) fn render_send_command_precondition_missing(
    runtime: &DockerItzgRuntimeConfig,
) -> String {
    format!(
        "Docker 容器不存在: {}。请先启动该服务器，或检查 container_name / docker runtime 配置。",
        runtime.container_name
    )
}

pub(super) fn render_send_command_precondition_stopped(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
    state: &DockerContainerState,
) -> String {
    format!(
        "Docker 容器当前不可接收命令: container={} {}。请先执行 `sealantern server status {}` 或 `sealantern server logs {} --lines 50` 检查容器状态，并在容器进入 running/healthy 后重试。",
        runtime.container_name,
        render_container_runtime_state(state),
        server.id,
        server.id
    )
}

pub(super) fn render_send_command_precondition_not_ready(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
    state: &DockerContainerState,
) -> Option<String> {
    let health_status = state.health_status.as_deref()?;

    if health_status.eq_ignore_ascii_case("starting") {
        return Some(format!(
            "Docker 容器当前不可接收命令: container={} {}。容器已启动但健康检查仍处于 starting，请先执行 `sealantern server status {}` 或 `sealantern server logs {} --lines 50` 观察启动过程；如目标是 itzg/minecraft-server，请等待 health=healthy 后再重试。",
            runtime.container_name,
            render_container_runtime_state(state),
            server.id,
            server.id
        ));
    }

    if health_status.eq_ignore_ascii_case("unhealthy") {
        return Some(format!(
            "Docker 容器当前不可接收命令: container={} {}。容器当前 health=unhealthy，请先执行 `sealantern server status {}` 或 `sealantern server logs {} --lines 50` 排查启动失败、配置错误或端口/RCON 问题，再决定是否重启或强制停止。",
            runtime.container_name,
            render_container_runtime_state(state),
            server.id,
            server.id
        ));
    }

    None
}

pub(super) fn render_rcon_connect_error(
    runtime: &DockerItzgRuntimeConfig,
    address: &str,
    error: &str,
) -> String {
    format!(
        "通过 RCON 连接 Docker 容器失败: container={} endpoint={} command_mode=rcon error={}。请确认容器已完成启动、RCON 端口可达、密码正确，或切换到 --command-mode docker_stdio。",
        runtime.container_name, address, error
    )
}

pub(super) fn render_rcon_command_error(
    runtime: &DockerItzgRuntimeConfig,
    address: &str,
    error: &str,
) -> String {
    format!(
        "通过 RCON 发送 Docker 命令失败: container={} endpoint={} command_mode=rcon error={}。请检查容器运行状态、RCON 配置与网络连通性。",
        runtime.container_name, address, error
    )
}

pub(super) fn map_container_status(state: &DockerContainerState) -> ServerStatus {
    if state.running {
        if let Some(health_status) = state.health_status.as_deref() {
            if health_status.eq_ignore_ascii_case("healthy") {
                return ServerStatus::Running;
            }
            if health_status.eq_ignore_ascii_case("starting") {
                return ServerStatus::Starting;
            }
            if health_status.eq_ignore_ascii_case("unhealthy") {
                return ServerStatus::Error;
            }
        }
    }

    match state.status.as_str() {
        "running" => ServerStatus::Running,
        "created" | "restarting" => ServerStatus::Starting,
        "removing" | "paused" | "dead" => ServerStatus::Stopping,
        "exited" => {
            if exit_code_should_be_treated_as_stopped(state.exit_code) {
                ServerStatus::Stopped
            } else {
                ServerStatus::Error
            }
        }
        _ => ServerStatus::Stopped,
    }
}

pub(super) fn exit_code_should_be_treated_as_stopped(exit_code: Option<i64>) -> bool {
    matches!(exit_code.unwrap_or_default(), 0 | 130 | 143)
}

pub(super) fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

pub(super) fn docker_output_indicates_missing_container(output: &Output) -> bool {
    docker_error_indicates_missing_container(&stderr_text(output))
        || docker_error_indicates_missing_container(String::from_utf8_lossy(&output.stdout).trim())
}

pub(super) fn is_container_not_found(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("no such object") || lower.contains("no such container")
}

pub(super) fn docker_exec_missing_mc_send_to_console(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("mc-send-to-console")
        && (lower.contains("not found")
            || lower.contains("no such file")
            || lower.contains("executable file not found"))
}

pub(super) fn docker_exec_requires_console_pipe(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("create_console_in_pipe") || lower.contains("console pipe needs to be enabled")
}

pub(super) fn docker_exec_named_pipe_missing(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("named pipe") && lower.contains("missing")
}

pub(super) fn docker_exec_requires_uid(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("exec needs to be run with user id")
}

#[cfg(test)]
mod tests {
    use super::{build_docker_launch_detail, resolve_docker_launch_spec};
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, DockerBackendKind, DockerCommandMode,
        DockerItzgRuntimeConfig, JvmPresetConfig, JvmPresetId, RconConfig, ServerInstance,
        ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use crate::services::server::manager::startup_support::resolve_effective_startup_config;
    use sea_lantern_docker_core::{
        build_docker_effective_env, resolve_docker_active_processor_count, resolve_docker_cpuset,
        ActiveProcessorCountDecision,
    };
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    fn docker_runtime() -> DockerItzgRuntimeConfig {
        DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.20.6".to_string(),
            data_dir_mount: "/data".to_string(),
            published_game_port: 25565,
            env: BTreeMap::new(),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::DockerStdio,
            rcon: Some(RconConfig {
                host: "127.0.0.1".to_string(),
                port: 25575,
                password: "secret".to_string(),
            }),
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        }
    }

    fn docker_server(path: String, runtime: DockerItzgRuntimeConfig) -> ServerInstance {
        ServerInstance {
            id: "docker-server".to_string(),
            name: "Docker Server".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.20.6".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(runtime),
        }
    }

    fn test_settings() -> AppSettings {
        AppSettings::default()
    }

    #[test]
    fn resolve_runtime_cpuset_supports_count_mode() {
        let mut runtime = docker_runtime();
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(4),
            explicit_set: None,
            sync_active_processor_count: true,
        };

        assert_eq!(resolve_docker_cpuset(&runtime.cpu_policy).unwrap(), Some("0-3".to_string()));
        assert_eq!(resolve_docker_active_processor_count(&runtime.cpu_policy).unwrap(), Some(4));
    }

    #[test]
    fn resolve_runtime_cpuset_supports_explicit_mode() {
        let mut runtime = docker_runtime();
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some("0-3,6,7".to_string()),
            sync_active_processor_count: true,
        };

        assert_eq!(
            resolve_docker_cpuset(&runtime.cpu_policy).unwrap(),
            Some("0-3,6-7".to_string())
        );
        assert_eq!(resolve_docker_active_processor_count(&runtime.cpu_policy).unwrap(), Some(6));
    }

    #[test]
    fn resolve_runtime_cpuset_skips_off_mode() {
        let runtime = docker_runtime();
        assert_eq!(resolve_docker_cpuset(&runtime.cpu_policy).unwrap(), None);
        assert_eq!(resolve_docker_active_processor_count(&runtime.cpu_policy).unwrap(), None);
    }

    #[test]
    fn resolve_runtime_cpuset_rejects_invalid_values() {
        let mut count_zero = docker_runtime();
        count_zero.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(0),
            explicit_set: None,
            sync_active_processor_count: true,
        };
        assert!(resolve_docker_cpuset(&count_zero.cpu_policy).is_err());

        let mut explicit_empty = docker_runtime();
        explicit_empty.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some(" ".to_string()),
            sync_active_processor_count: true,
        };
        assert!(resolve_docker_cpuset(&explicit_empty.cpu_policy).is_err());

        let mut explicit_invalid = docker_runtime();
        explicit_invalid.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some("3-1".to_string()),
            sync_active_processor_count: true,
        };
        assert!(resolve_docker_cpuset(&explicit_invalid.cpu_policy).is_err());
    }

    #[test]
    fn build_effective_env_synthesizes_preset_user_args_and_active_processor_count() {
        let mut runtime = docker_runtime();
        runtime.jvm_preset = JvmPresetConfig { preset: JvmPresetId::G1Basic };
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(2),
            explicit_set: None,
            sync_active_processor_count: true,
        };
        runtime.jvm_args = vec!["-Dfoo=bar".to_string(), "-XX:+UseStringDeduplication".to_string()];

        let temp_dir = tempdir().unwrap();
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());
        let effective = resolve_effective_startup_config(&server, &test_settings());
        let (env, meta) =
            build_docker_effective_env(&runtime, &super::adapt_effective_launch_config(&effective))
                .unwrap();
        let jvm_opts = env.iter().find(|(k, _)| k == "JVM_OPTS").unwrap().1.clone();
        let jvm_xx_opts = env
            .iter()
            .find(|(k, _)| k == "JVM_XX_OPTS")
            .unwrap()
            .1
            .clone();

        assert_eq!(env.iter().find(|(k, _)| k == "MAX_MEMORY").unwrap().1, "4G");
        assert_eq!(env.iter().find(|(k, _)| k == "INIT_MEMORY").unwrap().1, "2G");
        assert!(jvm_opts.contains("-Dfoo=bar"));
        assert!(jvm_xx_opts.contains("-XX:+UseG1GC"));
        assert!(jvm_xx_opts.contains("-XX:ActiveProcessorCount=2"));
        assert!(jvm_xx_opts.contains("-XX:+UseStringDeduplication"));
        assert_eq!(meta.preset, "g1_basic");
        assert_eq!(meta.active_processor_count, ActiveProcessorCountDecision::Injected(2));
    }

    #[test]
    fn build_effective_env_skips_active_processor_count_when_user_already_provided() {
        let mut runtime = docker_runtime();
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(3),
            explicit_set: None,
            sync_active_processor_count: true,
        };
        runtime.jvm_args = vec!["-XX:ActiveProcessorCount=9".to_string()];

        let temp_dir = tempdir().unwrap();
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());
        let effective = resolve_effective_startup_config(&server, &test_settings());
        let (env, meta) =
            build_docker_effective_env(&runtime, &super::adapt_effective_launch_config(&effective))
                .unwrap();
        let jvm_xx_opts = env
            .iter()
            .find(|(k, _)| k == "JVM_XX_OPTS")
            .unwrap()
            .1
            .clone();

        assert!(jvm_xx_opts.contains("-XX:ActiveProcessorCount=9"));
        assert!(!jvm_xx_opts.contains("-XX:ActiveProcessorCount=3"));
        assert_eq!(meta.active_processor_count, ActiveProcessorCountDecision::SkippedByJvmArgs);
    }

    #[test]
    fn build_effective_env_respects_runtime_env_takeover() {
        let mut runtime = docker_runtime();
        runtime.jvm_preset = JvmPresetConfig { preset: JvmPresetId::AikarG1 };
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(2),
            explicit_set: None,
            sync_active_processor_count: true,
        };
        runtime
            .env
            .insert("JVM_OPTS".to_string(), "-Dmanual=true".to_string());
        runtime
            .env
            .insert("JVM_XX_OPTS".to_string(), "-XX:ActiveProcessorCount=99".to_string());

        let temp_dir = tempdir().unwrap();
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());
        let effective = resolve_effective_startup_config(&server, &test_settings());
        let (env, meta) =
            build_docker_effective_env(&runtime, &super::adapt_effective_launch_config(&effective))
                .unwrap();

        assert_eq!(env.iter().find(|(k, _)| k == "JVM_OPTS").unwrap().1, "-Dmanual=true");
        assert_eq!(
            env.iter().find(|(k, _)| k == "JVM_XX_OPTS").unwrap().1,
            "-XX:ActiveProcessorCount=99"
        );
        assert!(meta.jvm_opts_overridden_by_runtime_env);
        assert!(meta.jvm_xx_opts_overridden_by_runtime_env);
        assert_eq!(
            meta.active_processor_count,
            ActiveProcessorCountDecision::SkippedByRuntimeEnvOverride
        );
    }

    #[test]
    fn resolve_docker_launch_spec_keeps_console_pipe_and_cpuset() {
        let mut runtime = docker_runtime();
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some("1,3,5".to_string()),
            sync_active_processor_count: true,
        };

        let temp_dir = tempdir().unwrap();
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());
        let spec = resolve_docker_launch_spec(&server, &runtime, &test_settings()).unwrap();

        assert_eq!(spec.cpuset_cpus, Some("1,3,5".to_string()));
        assert!(spec
            .environment
            .iter()
            .any(|(k, v)| k == "CREATE_CONSOLE_IN_PIPE" && v == "true"));
    }

    #[test]
    fn build_docker_launch_detail_exposes_previewable_start_shape() {
        let mut runtime = docker_runtime();
        runtime.jvm_preset = JvmPresetConfig { preset: JvmPresetId::AikarG1 };
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(2),
            explicit_set: None,
            sync_active_processor_count: true,
        };
        runtime.jvm_args = vec!["-Dfoo=bar".to_string()];
        runtime
            .env
            .insert("RCON_PASSWORD".to_string(), "secret-pass".to_string());

        let temp_dir = tempdir().unwrap();
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());
        let detail = build_docker_launch_detail(&server, &runtime, &test_settings()).unwrap();

        assert_eq!(detail.runtime_kind, "docker_itzg");
        assert_eq!(detail.cpuset_applied.as_deref(), Some("0-1"));
        assert_eq!(detail.effective_max_memory, 4096);
        assert_eq!(detail.effective_min_memory, 2048);
        assert_eq!(detail.jvm_preset, "aikar_g1");
        assert_eq!(detail.jvm_opts_preview.as_deref(), Some("-Dfoo=bar"));
        assert!(detail
            .jvm_xx_opts_preview
            .as_deref()
            .is_some_and(|value| value.contains("-XX:ActiveProcessorCount=2")));
        assert_eq!(detail.active_processor_count_status, "injected");
        assert_eq!(detail.active_processor_count_value, Some(2));
        assert!(detail
            .command_preview
            .contains("docker run -d --name sea-test --cpuset-cpus 0-1"));
        let joined = detail.docker_args_preview.join(" ");
        assert!(joined.contains("RCON_PASSWORD=<redacted>"));
        assert!(joined.contains("JVM_OPTS=-Dfoo=bar"));
        assert!(joined.contains("JVM_XX_OPTS="));
    }

    #[test]
    fn build_docker_launch_detail_prefers_instance_config_over_runtime_values() {
        let temp_dir = tempdir().unwrap();
        let mut runtime = docker_runtime();
        runtime.jvm_preset = JvmPresetConfig { preset: JvmPresetId::None };
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Off,
            count: None,
            explicit_set: None,
            sync_active_processor_count: true,
        };
        runtime.jvm_args = vec!["-Druntime.flag=true".to_string()];
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());

        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            concat!(
                "max_memory = 3072\n",
                "min_memory = 1536\n",
                "jvm_args = [\"-Dinstance.flag=true\"]\n",
                "[cpu_policy]\n",
                "mode = \"explicit\"\n",
                "explicit_set = \"0,2\"\n",
                "sync_active_processor_count = true\n",
                "[jvm_preset]\n",
                "preset = \"paper_recommended_lite\"\n"
            ),
        )
        .unwrap();

        let detail = build_docker_launch_detail(&server, &runtime, &test_settings()).unwrap();

        assert_eq!(detail.effective_max_memory, 3072);
        assert_eq!(detail.effective_min_memory, 1536);
        assert_eq!(detail.cpuset_applied.as_deref(), Some("0,2"));
        assert_eq!(detail.jvm_preset, "paper_recommended_lite");
        assert_eq!(detail.active_processor_count_value, Some(2));
        assert_eq!(
            detail.jvm_opts_preview.as_deref(),
            Some("-Dusing.aikars.flags=https://mcflags.emc.gs -Dinstance.flag=true")
        );
        assert!(detail
            .jvm_xx_opts_preview
            .as_deref()
            .is_some_and(|value| value.contains("-XX:+UseG1GC")));
        assert!(detail.command_preview.contains("--cpuset-cpus 0,2"));
        let joined = detail.docker_args_preview.join(" ");
        assert!(joined.contains("MAX_MEMORY=3G"));
        assert!(joined.contains("INIT_MEMORY=1536M"));
    }

    #[test]
    fn runtime_jvm_preset_name_is_stable() {
        assert_eq!(JvmPresetId::None.as_str(), "none");
        assert_eq!(JvmPresetId::AikarG1.as_str(), "aikar_g1");
    }

    #[test]
    fn build_docker_launch_detail_refreshes_after_instance_config_changes() {
        let temp_dir = tempdir().unwrap();
        let mut runtime = docker_runtime();
        runtime.jvm_preset = JvmPresetConfig { preset: JvmPresetId::None };
        runtime.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Off,
            count: None,
            explicit_set: None,
            sync_active_processor_count: true,
        };
        runtime.jvm_args = vec!["-Druntime.flag=true".to_string()];
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());

        let detail_before =
            build_docker_launch_detail(&server, &runtime, &test_settings()).unwrap();
        assert_eq!(detail_before.effective_max_memory, 4096);
        assert_eq!(detail_before.cpuset_applied, None);
        assert_eq!(detail_before.jvm_preset, "none");
        assert_eq!(detail_before.jvm_opts_preview.as_deref(), Some("-Druntime.flag=true"));

        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            concat!(
                "max_memory = 5120\n",
                "min_memory = 1536\n",
                "jvm_args = [\"-Dupdated.flag=true\"]\n",
                "[cpu_policy]\n",
                "mode = \"explicit\"\n",
                "explicit_set = \"1,3\"\n",
                "sync_active_processor_count = true\n",
                "[jvm_preset]\n",
                "preset = \"aikar_g1\"\n"
            ),
        )
        .unwrap();

        let detail_after = build_docker_launch_detail(&server, &runtime, &test_settings()).unwrap();

        assert_eq!(detail_after.effective_max_memory, 5120);
        assert_eq!(detail_after.effective_min_memory, 1536);
        assert_eq!(detail_after.cpuset_applied.as_deref(), Some("1,3"));
        assert_eq!(detail_after.jvm_preset, "aikar_g1");
        assert_eq!(detail_after.active_processor_count_value, Some(2));
        assert_eq!(detail_after.jvm_opts_preview.as_deref(), Some("-Dupdated.flag=true"));
        assert!(detail_after
            .jvm_xx_opts_preview
            .as_deref()
            .is_some_and(|value| value.contains("-XX:ActiveProcessorCount=2")));
        assert!(detail_after.command_preview.contains("--cpuset-cpus 1,3"));
        assert!(!detail_after
            .jvm_opts_preview
            .as_deref()
            .is_some_and(|value| value.contains("-Druntime.flag=true")));
    }

    #[test]
    fn resolve_docker_launch_spec_surfaces_invalid_instance_config() {
        let temp_dir = tempdir().unwrap();
        let runtime = docker_runtime();
        let server = docker_server(temp_dir.path().to_string_lossy().to_string(), runtime.clone());

        let config_dir = temp_dir.path().join("SeaLantern");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "max_memory = [\n").unwrap();

        let err = resolve_docker_launch_spec(&server, &runtime, &test_settings())
            .expect_err("invalid instance config should block docker launch synthesis");

        assert!(err.contains("解析实例配置失败"));
    }
}
