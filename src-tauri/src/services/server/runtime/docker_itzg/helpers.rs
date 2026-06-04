use super::{DockerContainerState, RuntimeStatusSnapshot, DOCKER_ITZG_RUNTIME_KIND};
use crate::models::server::{
    CpuPolicyMode, DockerCommandMode, DockerItzgRuntimeConfig, JvmPresetId, PublishedPort,
    ServerInstance, ServerStatus, VolumeMount,
};
use crate::utils::docker_cli::docker_error_indicates_missing_container;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerLaunchSpec {
    pub cpuset_cpus: Option<String>,
    pub environment: Vec<(String, String)>,
    pub jvm_opts_args_count: usize,
    pub jvm_xx_opts_args_count: usize,
    pub jvm_synthesis: DockerJvmSynthesisMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DockerLaunchDetail {
    pub runtime_kind: String,
    pub image: String,
    pub image_tag: String,
    pub container_name: String,
    pub cpuset_applied: Option<String>,
    pub jvm_preset: String,
    pub jvm_opts_preview: Option<String>,
    pub jvm_xx_opts_preview: Option<String>,
    pub jvm_opts_args_count: usize,
    pub jvm_xx_opts_args_count: usize,
    pub jvm_opts_overridden_by_runtime_env: bool,
    pub jvm_xx_opts_overridden_by_runtime_env: bool,
    pub active_processor_count_status: String,
    pub active_processor_count_value: Option<u16>,
    pub docker_args_preview: Vec<String>,
    pub command_preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ActiveProcessorCountDecision {
    Disabled,
    Injected(u16),
    SkippedByJvmArgs,
    SkippedByRuntimeEnvOverride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockerJvmSynthesisMeta {
    pub preset: &'static str,
    pub jvm_opts_args_count: usize,
    pub jvm_xx_opts_args_count: usize,
    pub jvm_opts_overridden_by_runtime_env: bool,
    pub jvm_xx_opts_overridden_by_runtime_env: bool,
    pub active_processor_count: ActiveProcessorCountDecision,
}

pub(crate) fn resolve_docker_launch_spec(
    runtime: &DockerItzgRuntimeConfig,
) -> Result<DockerLaunchSpec, String> {
    let cpuset_cpus = resolve_runtime_cpuset(runtime)?;
    let (environment, meta) = build_effective_env(runtime)?;
    Ok(DockerLaunchSpec {
        cpuset_cpus,
        environment,
        jvm_opts_args_count: meta.jvm_opts_args_count,
        jvm_xx_opts_args_count: meta.jvm_xx_opts_args_count,
        jvm_synthesis: meta,
    })
}

impl ActiveProcessorCountDecision {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Injected(_) => "injected",
            Self::SkippedByJvmArgs => "skipped_by_jvm_args",
            Self::SkippedByRuntimeEnvOverride => "skipped_by_runtime_env_override",
        }
    }

    pub(crate) fn value(&self) -> Option<u16> {
        match self {
            Self::Injected(value) => Some(*value),
            Self::Disabled | Self::SkippedByJvmArgs | Self::SkippedByRuntimeEnvOverride => None,
        }
    }
}

pub(crate) fn build_docker_launch_detail(
    runtime: &DockerItzgRuntimeConfig,
) -> Result<DockerLaunchDetail, String> {
    let launch_spec = resolve_docker_launch_spec(runtime)?;
    let docker_args_preview =
        sanitize_docker_args_preview(&build_docker_run_args(runtime, &launch_spec));
    let command_preview = format_command_preview("docker", &docker_args_preview);

    Ok(DockerLaunchDetail {
        runtime_kind: DOCKER_ITZG_RUNTIME_KIND.to_string(),
        image: runtime.image.clone(),
        image_tag: runtime.image_tag.clone(),
        container_name: runtime.container_name.clone(),
        cpuset_applied: launch_spec.cpuset_cpus.clone(),
        jvm_preset: launch_spec.jvm_synthesis.preset.to_string(),
        jvm_opts_preview: preview_env_value(&launch_spec.environment, "JVM_OPTS"),
        jvm_xx_opts_preview: preview_env_value(&launch_spec.environment, "JVM_XX_OPTS"),
        jvm_opts_args_count: launch_spec.jvm_synthesis.jvm_opts_args_count,
        jvm_xx_opts_args_count: launch_spec.jvm_synthesis.jvm_xx_opts_args_count,
        jvm_opts_overridden_by_runtime_env: launch_spec
            .jvm_synthesis
            .jvm_opts_overridden_by_runtime_env,
        jvm_xx_opts_overridden_by_runtime_env: launch_spec
            .jvm_synthesis
            .jvm_xx_opts_overridden_by_runtime_env,
        active_processor_count_status: launch_spec
            .jvm_synthesis
            .active_processor_count
            .as_str()
            .to_string(),
        active_processor_count_value: launch_spec.jvm_synthesis.active_processor_count.value(),
        docker_args_preview,
        command_preview,
    })
}

pub(crate) fn build_docker_run_args(
    runtime: &DockerItzgRuntimeConfig,
    launch_spec: &DockerLaunchSpec,
) -> Vec<String> {
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        runtime.container_name.clone(),
    ];

    if let Some(cpuset) = &launch_spec.cpuset_cpus {
        args.push("--cpuset-cpus".to_string());
        args.push(cpuset.clone());
    }

    args.push("-p".to_string());
    args.push(format!("{}:25565/tcp", runtime.published_game_port));
    for port in &runtime.extra_ports {
        args.push("-p".to_string());
        args.push(format_published_port(port));
    }

    args.push("-v".to_string());
    args.push(format!("{}:/data", runtime.data_dir_mount));
    for mount in &runtime.volume_mounts {
        args.push("-v".to_string());
        args.push(format_volume_mount(mount));
    }

    for (key, value) in &launch_spec.environment {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }

    args.push(format!("{}:{}", runtime.image, runtime.image_tag));
    args
}

pub(crate) fn build_effective_env(
    runtime: &DockerItzgRuntimeConfig,
) -> Result<(Vec<(String, String)>, DockerJvmSynthesisMeta), String> {
    let mut env: Vec<(String, String)> = runtime
        .env
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

    let eula_value = env_value_or_default(&env, "EULA", "TRUE");
    upsert_env(&mut env, "TYPE", runtime.type_value.clone());
    upsert_env(&mut env, "VERSION", runtime.version.clone());
    upsert_env(&mut env, "EULA", eula_value);
    if runtime.command_mode == DockerCommandMode::DockerStdio {
        upsert_env(&mut env, "CREATE_CONSOLE_IN_PIPE", "true".to_string());
    }

    let preset = preset_args(runtime);
    let runtime_jvm_xx_override = env_contains_key(&env, "JVM_XX_OPTS");
    let runtime_jvm_opts_override = env_contains_key(&env, "JVM_OPTS");
    let user_has_apc = jvm_args_contain_active_processor_count(&runtime.jvm_args);

    let active_processor_count = match resolve_active_processor_count(runtime)? {
        Some(_) if runtime_jvm_xx_override => {
            ActiveProcessorCountDecision::SkippedByRuntimeEnvOverride
        }
        Some(_) if user_has_apc => ActiveProcessorCountDecision::SkippedByJvmArgs,
        Some(value) => ActiveProcessorCountDecision::Injected(value),
        None => ActiveProcessorCountDecision::Disabled,
    };

    let mut managed_jvm_opts = Vec::new();
    let mut managed_jvm_xx_opts = Vec::new();
    extend_partitioned_args(
        &mut managed_jvm_opts,
        &mut managed_jvm_xx_opts,
        preset.iter().map(|arg| (*arg).to_string()),
    );
    if let ActiveProcessorCountDecision::Injected(value) = active_processor_count {
        managed_jvm_xx_opts.push(format!("-XX:ActiveProcessorCount={}", value));
    }
    extend_partitioned_args(
        &mut managed_jvm_opts,
        &mut managed_jvm_xx_opts,
        runtime.jvm_args.iter().cloned(),
    );

    if !runtime_jvm_xx_override && !managed_jvm_xx_opts.is_empty() {
        upsert_env(&mut env, "JVM_XX_OPTS", managed_jvm_xx_opts.join(" "));
    }
    if !runtime_jvm_opts_override && !managed_jvm_opts.is_empty() {
        upsert_env(&mut env, "JVM_OPTS", managed_jvm_opts.join(" "));
    }

    let meta = DockerJvmSynthesisMeta {
        preset: runtime_jvm_preset_name(&runtime.jvm_preset.preset),
        jvm_opts_args_count: managed_jvm_opts.len(),
        jvm_xx_opts_args_count: managed_jvm_xx_opts.len(),
        jvm_opts_overridden_by_runtime_env: runtime_jvm_opts_override,
        jvm_xx_opts_overridden_by_runtime_env: runtime_jvm_xx_override,
        active_processor_count,
    };

    Ok((env, meta))
}

pub(crate) fn resolve_runtime_cpuset(
    runtime: &DockerItzgRuntimeConfig,
) -> Result<Option<String>, String> {
    match runtime.cpu_policy.mode {
        CpuPolicyMode::Off => Ok(None),
        CpuPolicyMode::Count => {
            let count = runtime
                .cpu_policy
                .count
                .ok_or_else(|| "Docker CPU policy count 模式缺少 count".to_string())?;
            if count == 0 {
                return Err("Docker CPU policy count 模式必须大于 0".to_string());
            }

            Ok(Some(format!("0-{}", count - 1)))
        }
        CpuPolicyMode::Explicit => {
            let raw = runtime
                .cpu_policy
                .explicit_set
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Docker CPU policy explicit 模式缺少 explicit_set".to_string())?;
            let indices = parse_cpu_set(raw)?;
            if indices.is_empty() {
                return Err("Docker CPU 核心集合解析后为空".to_string());
            }
            Ok(Some(format_range(&indices)))
        }
    }
}

pub(crate) fn resolve_active_processor_count(
    runtime: &DockerItzgRuntimeConfig,
) -> Result<Option<u16>, String> {
    if runtime.cpu_policy.mode == CpuPolicyMode::Off
        || !runtime.cpu_policy.sync_active_processor_count
    {
        return Ok(None);
    }

    match runtime.cpu_policy.mode {
        CpuPolicyMode::Off => Ok(None),
        CpuPolicyMode::Count => runtime
            .cpu_policy
            .count
            .filter(|value| *value > 0)
            .map(Some)
            .ok_or_else(|| "Docker CPU policy count 模式必须大于 0".to_string()),
        CpuPolicyMode::Explicit => {
            let raw = runtime
                .cpu_policy
                .explicit_set
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Docker CPU policy explicit 模式缺少 explicit_set".to_string())?;
            let indices = parse_cpu_set(raw)?;
            let count = u16::try_from(indices.len())
                .map_err(|_| "Docker CPU 核心集合数量过大".to_string())?;
            if count == 0 {
                return Err("Docker CPU 核心集合解析后为空".to_string());
            }
            Ok(Some(count))
        }
    }
}

pub(crate) fn runtime_jvm_preset_name(preset: &JvmPresetId) -> &'static str {
    match preset {
        JvmPresetId::None => "none",
        JvmPresetId::G1Basic => "g1_basic",
        JvmPresetId::AikarG1 => "aikar_g1",
        JvmPresetId::ThroughputBasic => "throughput_basic",
        JvmPresetId::PaperRecommendedLite => "paper_recommended_lite",
    }
}

fn preset_args(runtime: &DockerItzgRuntimeConfig) -> &'static [&'static str] {
    match runtime.jvm_preset.preset {
        JvmPresetId::None => &[],
        JvmPresetId::G1Basic => &[
            "-XX:+UseG1GC",
            "-XX:+ParallelRefProcEnabled",
            "-XX:MaxGCPauseMillis=200",
            "-XX:+UnlockExperimentalVMOptions",
        ],
        JvmPresetId::AikarG1 => &[
            "-XX:+UseG1GC",
            "-XX:+ParallelRefProcEnabled",
            "-XX:MaxGCPauseMillis=200",
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+DisableExplicitGC",
            "-XX:+AlwaysPreTouch",
            "-XX:G1NewSizePercent=30",
            "-XX:G1MaxNewSizePercent=40",
            "-XX:G1HeapRegionSize=8M",
            "-XX:G1ReservePercent=20",
            "-XX:G1HeapWastePercent=5",
            "-XX:G1MixedGCCountTarget=4",
            "-XX:InitiatingHeapOccupancyPercent=15",
            "-XX:G1MixedGCLiveThresholdPercent=90",
            "-XX:G1RSetUpdatingPauseTimePercent=5",
            "-XX:SurvivorRatio=32",
            "-XX:+PerfDisableSharedMem",
            "-XX:MaxTenuringThreshold=1",
        ],
        JvmPresetId::ThroughputBasic => {
            &["-XX:+UseParallelGC", "-XX:+UseAdaptiveSizePolicy", "-XX:MaxGCPauseMillis=500"]
        }
        JvmPresetId::PaperRecommendedLite => &[
            "-XX:+UseG1GC",
            "-XX:+ParallelRefProcEnabled",
            "-XX:MaxGCPauseMillis=150",
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+DisableExplicitGC",
            "-Dusing.aikars.flags=https://mcflags.emc.gs",
        ],
    }
}

fn extend_partitioned_args<I>(jvm_opts: &mut Vec<String>, jvm_xx_opts: &mut Vec<String>, args: I)
where
    I: IntoIterator<Item = String>,
{
    for arg in args {
        if arg.starts_with("-XX:") {
            jvm_xx_opts.push(arg);
        } else {
            jvm_opts.push(arg);
        }
    }
}

fn jvm_args_contain_active_processor_count(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg.starts_with("-XX:ActiveProcessorCount="))
}

fn env_contains_key(env: &[(String, String)], key: &str) -> bool {
    env.iter()
        .any(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
}

fn parse_cpu_set(raw: &str) -> Result<Vec<usize>, String> {
    let mut values = Vec::new();
    for chunk in raw.split(',') {
        let token = chunk.trim();
        if token.is_empty() {
            return Err(format!("Docker CPU 核心集合格式无效: {}", raw));
        }

        if let Some((start_raw, end_raw)) = token.split_once('-') {
            let start = parse_cpu_index(start_raw, raw)?;
            let end = parse_cpu_index(end_raw, raw)?;
            if end < start {
                return Err(format!("Docker CPU 核心区间无效: {}", raw));
            }
            values.extend(start..=end);
        } else {
            values.push(parse_cpu_index(token, raw)?);
        }
    }

    values.sort_unstable();
    values.dedup();
    Ok(values)
}

fn parse_cpu_index(raw: &str, whole: &str) -> Result<usize, String> {
    raw.trim()
        .parse::<usize>()
        .map_err(|_| format!("Docker CPU 核心集合格式无效: {}", whole))
}

fn format_range(indices: &[usize]) -> String {
    if indices.is_empty() {
        return String::new();
    }

    let mut ranges = Vec::new();
    let mut start = indices[0];
    let mut prev = indices[0];

    for &value in &indices[1..] {
        if value == prev + 1 {
            prev = value;
            continue;
        }

        ranges.push(format_segment(start, prev));
        start = value;
        prev = value;
    }

    ranges.push(format_segment(start, prev));
    ranges.join(",")
}

fn format_segment(start: usize, end: usize) -> String {
    if start == end {
        start.to_string()
    } else {
        format!("{}-{}", start, end)
    }
}

fn sanitize_docker_args_preview(args: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let current = &args[index];
        if current == "-e" {
            sanitized.push(current.clone());
            if let Some(env_assignment) = args.get(index + 1) {
                sanitized.push(sanitize_env_assignment_for_preview(env_assignment));
                index += 2;
                continue;
            }
        }

        sanitized.push(current.clone());
        index += 1;
    }

    sanitized
}

fn sanitize_env_assignment_for_preview(assignment: &str) -> String {
    let Some((key, value)) = assignment.split_once('=') else {
        return assignment.to_string();
    };

    if is_safe_preview_env_key(key) {
        format!("{}={}", key, value)
    } else {
        format!("{}=<redacted>", key)
    }
}

fn preview_env_value(env: &[(String, String)], key: &str) -> Option<String> {
    env.iter()
        .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.clone())
}

fn is_safe_preview_env_key(key: &str) -> bool {
    matches!(
        key.to_ascii_uppercase().as_str(),
        "TYPE"
            | "VERSION"
            | "EULA"
            | "CREATE_CONSOLE_IN_PIPE"
            | "MEMORY"
            | "MAX_MEMORY"
            | "INIT_MEMORY"
            | "JVM_OPTS"
            | "JVM_XX_OPTS"
    )
}

fn format_command_preview(program: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(quote_command_fragment(program));
    parts.extend(args.iter().map(|arg| quote_command_fragment(arg)));
    parts.join(" ")
}

fn quote_command_fragment(value: &str) -> String {
    let requires_quotes = value.is_empty()
        || value.chars().any(|ch| ch.is_whitespace())
        || value.contains('"')
        || value.contains('\'')
        || value.contains(';')
        || value.contains('&')
        || value.contains('|');

    if !requires_quotes {
        return value.to_string();
    }

    if value.contains('"') && !value.contains('\'') {
        return format!("'{}'", value);
    }

    format!("\"{}\"", value.replace('"', "\\\""))
}

pub(super) fn ensure_runtime_path_ready(server: &ServerInstance) -> Result<(), String> {
    let path = Path::new(&server.path);
    std::fs::create_dir_all(path)
        .map_err(|e| format!("创建 Docker 数据目录失败 ({}): {}", path.display(), e))
}

fn env_value_or_default(env: &[(String, String)], key: &str, default: &str) -> String {
    env.iter()
        .find(|(existing, _)| existing.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| default.to_string())
}

fn upsert_env(env: &mut Vec<(String, String)>, key: &str, value: String) {
    if let Some((_, existing_value)) = env
        .iter_mut()
        .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
    {
        *existing_value = value;
        return;
    }

    env.push((key.to_string(), value));
}

pub(super) fn format_published_port(port: &PublishedPort) -> String {
    let protocol = if port.protocol.trim().is_empty() {
        "tcp"
    } else {
        port.protocol.trim()
    };
    format!("{}:{}/{}", port.host_port, port.container_port, protocol)
}

pub(super) fn format_volume_mount(mount: &VolumeMount) -> String {
    if mount.read_only {
        format!("{}:{}:ro", mount.source, mount.target)
    } else {
        format!("{}:{}", mount.source, mount.target)
    }
}

pub(super) fn docker_image_ref(runtime: &DockerItzgRuntimeConfig) -> String {
    format!("{}:{}", runtime.image, runtime.image_tag)
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

pub(super) fn runtime_env_value<'a>(
    runtime: &'a DockerItzgRuntimeConfig,
    key: &str,
) -> Option<&'a str> {
    runtime
        .env
        .iter()
        .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

pub(super) fn requested_stop_timeout_secs(runtime: &DockerItzgRuntimeConfig) -> Option<u64> {
    runtime_env_value(runtime, "STOP_DURATION")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
}

#[cfg(test)]
mod tests {
    use super::{
        build_docker_launch_detail, build_effective_env, format_range, parse_cpu_set,
        resolve_active_processor_count, resolve_docker_launch_spec, resolve_runtime_cpuset,
        runtime_jvm_preset_name, ActiveProcessorCountDecision,
    };
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, DockerBackendKind, DockerCommandMode,
        DockerItzgRuntimeConfig, JvmPresetConfig, JvmPresetId, RconConfig,
    };
    use std::collections::BTreeMap;

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

    #[test]
    fn parse_cpu_set_supports_ranges_and_lists() {
        assert_eq!(parse_cpu_set("0-3,6,7").unwrap(), vec![0, 1, 2, 3, 6, 7]);
    }

    #[test]
    fn format_range_compacts_consecutive_values() {
        assert_eq!(format_range(&[0, 1, 2, 4, 6, 7]), "0-2,4,6-7");
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

        assert_eq!(resolve_runtime_cpuset(&runtime).unwrap(), Some("0-3".to_string()));
        assert_eq!(resolve_active_processor_count(&runtime).unwrap(), Some(4));
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

        assert_eq!(resolve_runtime_cpuset(&runtime).unwrap(), Some("0-3,6-7".to_string()));
        assert_eq!(resolve_active_processor_count(&runtime).unwrap(), Some(6));
    }

    #[test]
    fn resolve_runtime_cpuset_skips_off_mode() {
        let runtime = docker_runtime();
        assert_eq!(resolve_runtime_cpuset(&runtime).unwrap(), None);
        assert_eq!(resolve_active_processor_count(&runtime).unwrap(), None);
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
        assert!(resolve_runtime_cpuset(&count_zero).is_err());

        let mut explicit_empty = docker_runtime();
        explicit_empty.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some(" ".to_string()),
            sync_active_processor_count: true,
        };
        assert!(resolve_runtime_cpuset(&explicit_empty).is_err());

        let mut explicit_invalid = docker_runtime();
        explicit_invalid.cpu_policy = CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some("3-1".to_string()),
            sync_active_processor_count: true,
        };
        assert!(resolve_runtime_cpuset(&explicit_invalid).is_err());
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

        let (env, meta) = build_effective_env(&runtime).unwrap();
        let jvm_opts = env.iter().find(|(k, _)| k == "JVM_OPTS").unwrap().1.clone();
        let jvm_xx_opts = env
            .iter()
            .find(|(k, _)| k == "JVM_XX_OPTS")
            .unwrap()
            .1
            .clone();

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

        let (env, meta) = build_effective_env(&runtime).unwrap();
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

        let (env, meta) = build_effective_env(&runtime).unwrap();

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

        let spec = resolve_docker_launch_spec(&runtime).unwrap();

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

        let detail = build_docker_launch_detail(&runtime).unwrap();

        assert_eq!(detail.runtime_kind, "docker_itzg");
        assert_eq!(detail.cpuset_applied.as_deref(), Some("0-1"));
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
    fn runtime_jvm_preset_name_is_stable() {
        assert_eq!(runtime_jvm_preset_name(&JvmPresetId::None), "none");
        assert_eq!(runtime_jvm_preset_name(&JvmPresetId::AikarG1), "aikar_g1");
    }
}
