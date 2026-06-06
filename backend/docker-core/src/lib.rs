use std::collections::BTreeMap;
use std::process::Output;

pub use sea_lantern_server_config_core::types::{
    CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId,
};
use sea_lantern_server_config_core::{
    resolve_cpu_policy as resolve_shared_cpu_policy,
    resolve_unbounded_active_processor_count as resolve_shared_active_processor_count,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerCommandFailureKind {
    Network,
    ImageNotFound,
    Auth,
    DaemonUnavailable,
    Timeout,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerImageAvailability {
    LocalCached,
    RemoteResolvable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerImageInspectOutcome {
    Available(DockerImageAvailability),
    SoftFailure {
        failure_kind: DockerCommandFailureKind,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DockerBackendKind {
    #[default]
    Cli,
    EngineApi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DockerCommandMode {
    #[default]
    Rcon,
    DockerStdio,
}

impl DockerBackendKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::EngineApi => "engine_api",
        }
    }
}

impl DockerCommandMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rcon => "rcon",
            Self::DockerStdio => "docker_stdio",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedPort {
    pub host_port: u16,
    pub container_port: u16,
    #[serde(default)]
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerItzgRuntimeConfig {
    pub image: String,
    pub image_tag: String,
    pub container_name: String,
    pub type_value: String,
    pub version: String,
    pub data_dir_mount: String,
    pub published_game_port: u16,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub extra_ports: Vec<PublishedPort>,
    #[serde(default)]
    pub volume_mounts: Vec<VolumeMount>,
    pub docker_backend_kind: DockerBackendKind,
    pub command_mode: DockerCommandMode,
    #[serde(default)]
    pub rcon: Option<RconConfig>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDockerItzgServerRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub core_type: String,
    pub mc_version: String,
    pub port: u16,
    pub runtime: DockerItzgRuntimeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveProcessorCountDecision {
    Disabled,
    Injected(u16),
    SkippedByJvmArgs,
    SkippedByRuntimeEnvOverride,
}

impl ActiveProcessorCountDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Injected(_) => "injected",
            Self::SkippedByJvmArgs => "skipped_by_jvm_args",
            Self::SkippedByRuntimeEnvOverride => "skipped_by_runtime_env_override",
        }
    }

    pub fn value(&self) -> Option<u16> {
        match self {
            Self::Injected(value) => Some(*value),
            Self::Disabled | Self::SkippedByJvmArgs | Self::SkippedByRuntimeEnvOverride => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerJvmSynthesisMeta {
    pub preset: &'static str,
    pub jvm_opts_args_count: usize,
    pub jvm_xx_opts_args_count: usize,
    pub jvm_opts_overridden_by_runtime_env: bool,
    pub jvm_xx_opts_overridden_by_runtime_env: bool,
    pub active_processor_count: ActiveProcessorCountDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerEffectiveLaunchConfig {
    pub max_memory: u32,
    pub min_memory: u32,
    pub jvm_args: Vec<String>,
    pub cpu_policy: CpuPolicyConfig,
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerLaunchSpec {
    pub cpuset_cpus: Option<String>,
    pub environment: Vec<(String, String)>,
    pub effective_max_memory: u32,
    pub effective_min_memory: u32,
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
    pub effective_max_memory: u32,
    pub effective_min_memory: u32,
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

pub fn parse_docker_backend(value: Option<&str>) -> Result<DockerBackendKind, String> {
    match value.unwrap_or("cli").trim().to_ascii_lowercase().as_str() {
        "cli" => Ok(DockerBackendKind::Cli),
        "engine_api" | "engine-api" => Ok(DockerBackendKind::EngineApi),
        other => Err(format!("不支持的 docker backend: {}", other)),
    }
}

pub fn parse_command_mode(value: Option<&str>) -> Result<DockerCommandMode, String> {
    match value.unwrap_or("rcon").trim().to_ascii_lowercase().as_str() {
        "rcon" => Ok(DockerCommandMode::Rcon),
        "docker_stdio" | "docker-stdio" | "stdio" => Ok(DockerCommandMode::DockerStdio),
        other => Err(format!("不支持的 docker command mode: {}", other)),
    }
}

pub fn format_memory_env_value(memory_mb: u32) -> String {
    if memory_mb > 0 && memory_mb.is_multiple_of(1024) {
        format!("{}G", memory_mb / 1024)
    } else {
        format!("{}M", memory_mb)
    }
}

pub fn docker_itzg_image_looks_compatible(image: &str) -> bool {
    let normalized = image.trim().trim_matches('/').to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }

    normalized == "minecraft-server" || normalized.ends_with("/minecraft-server")
}

pub fn validate_docker_itzg_image_compatibility(image: &str) -> Result<(), String> {
    if docker_itzg_image_looks_compatible(image) {
        return Ok(());
    }

    Err(format!(
        "当前 docker runtime 目标是 Minecraft server 容器，但镜像名看起来不兼容: {}。请使用 itzg/minecraft-server 或你自己的 */minecraft-server 镜像名；如果这是私有镜像/镜像代理，也请保持最终镜像名仍为 minecraft-server",
        image.trim()
    ))
}

pub fn split_docker_image_reference_tag(image_ref: &str) -> (&str, Option<&str>) {
    let trimmed = image_ref.trim();
    if trimmed.is_empty() || trimmed.contains('@') {
        return (trimmed, None);
    }

    let last_slash = trimmed.rfind('/');
    let last_colon = trimmed.rfind(':');

    match (last_slash, last_colon) {
        (_, None) => (trimmed, None),
        (Some(slash), Some(colon)) if colon > slash => {
            let tag = &trimmed[colon + 1..];
            if tag.trim().is_empty() {
                (trimmed, None)
            } else {
                (&trimmed[..colon], Some(tag))
            }
        }
        (None, Some(colon)) => {
            let tag = &trimmed[colon + 1..];
            if tag.trim().is_empty() {
                (trimmed, None)
            } else {
                (&trimmed[..colon], Some(tag))
            }
        }
        _ => (trimmed, None),
    }
}

pub fn resolve_docker_image_and_tag(
    image: Option<&str>,
    image_tag: Option<&str>,
    default_image: &str,
    default_tag: &str,
) -> Result<(String, String), String> {
    let raw_image = image
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default_image);

    if raw_image.contains('@') {
        return Err(
            "当前 CLI 暂不支持使用 Docker digest 作为 --image；请改用仓库名 + --image-tag，或直接传带 tag 的镜像引用"
                .to_string(),
        );
    }

    if has_empty_embedded_tag(raw_image) {
        return Err(format!(
            "Docker 镜像引用不能以空 tag 结尾: {}。请删除末尾冒号，或在 --image-tag / 镜像引用里提供非空 tag",
            raw_image
        ));
    }

    let (image_name, embedded_tag) = split_docker_image_reference_tag(raw_image);
    let resolved_image = image_name.trim();
    if resolved_image.is_empty() {
        return Err("Docker 镜像名不能为空".to_string());
    }

    let resolved_tag = image_tag
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| embedded_tag.map(str::to_string))
        .unwrap_or_else(|| default_tag.to_string());

    Ok((resolved_image.to_string(), resolved_tag))
}

fn has_empty_embedded_tag(image_ref: &str) -> bool {
    let trimmed = image_ref.trim();
    if trimmed.is_empty() || trimmed.contains('@') || !trimmed.ends_with(':') {
        return false;
    }

    let last_slash = trimmed.rfind('/');
    let last_colon = trimmed.rfind(':');

    match (last_slash, last_colon) {
        (_, None) => false,
        (Some(slash), Some(colon)) => colon > slash,
        (None, Some(_)) => true,
    }
}

pub fn format_docker_image_reference(image: &str, image_tag: &str) -> String {
    format!("{}:{}", image, image_tag)
}

pub fn format_published_port(port: &PublishedPort) -> String {
    let protocol = if port.protocol.trim().is_empty() {
        "tcp"
    } else {
        port.protocol.trim()
    };
    format!("{}:{}/{}", port.host_port, port.container_port, protocol)
}

pub fn format_volume_mount(mount: &VolumeMount) -> String {
    if mount.read_only {
        format!("{}:{}:ro", mount.source, mount.target)
    } else {
        format!("{}:{}", mount.source, mount.target)
    }
}

pub fn docker_image_ref(runtime: &DockerItzgRuntimeConfig) -> String {
    format!("{}:{}", runtime.image, runtime.image_tag)
}

pub fn runtime_env_value<'a>(runtime: &'a DockerItzgRuntimeConfig, key: &str) -> Option<&'a str> {
    runtime
        .env
        .iter()
        .find(|(existing_key, _)| existing_key.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

pub fn resolve_docker_cpuset(cpu_policy: &CpuPolicyConfig) -> Result<Option<String>, String> {
    let resolved = resolve_shared_cpu_policy(cpu_policy).map_err(map_docker_cpu_policy_error)?;
    Ok(resolved.map(|resolved| resolved.cpuset_display))
}

pub fn resolve_docker_active_processor_count(
    cpu_policy: &CpuPolicyConfig,
) -> Result<Option<u16>, String> {
    resolve_shared_active_processor_count(cpu_policy).map_err(map_docker_cpu_policy_error)
}

pub fn requested_stop_timeout_secs(runtime: &DockerItzgRuntimeConfig) -> Option<u64> {
    requested_stop_timeout_secs_checked(runtime).unwrap_or_default()
}

pub fn requested_stop_timeout_secs_checked(
    runtime: &DockerItzgRuntimeConfig,
) -> Result<Option<u64>, String> {
    runtime_env_value(runtime, "STOP_DURATION")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let parsed = value
                .parse::<u64>()
                .map_err(|e| format!("STOP_DURATION 无效 '{}': {}", value, e))?;
            if parsed == 0 {
                return Err("STOP_DURATION 必须大于 0 秒".to_string());
            }
            Ok(parsed)
        })
        .transpose()
}

pub fn build_docker_launch_spec(
    runtime: &DockerItzgRuntimeConfig,
    effective: &DockerEffectiveLaunchConfig,
) -> Result<DockerLaunchSpec, String> {
    let cpuset_cpus = resolve_docker_cpuset(&effective.cpu_policy)?;
    let (environment, meta) = build_docker_effective_env(runtime, effective)?;
    Ok(DockerLaunchSpec {
        cpuset_cpus,
        environment,
        effective_max_memory: effective.max_memory,
        effective_min_memory: effective.min_memory,
        jvm_opts_args_count: meta.jvm_opts_args_count,
        jvm_xx_opts_args_count: meta.jvm_xx_opts_args_count,
        jvm_synthesis: meta,
    })
}

pub fn build_docker_run_args(
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

pub fn build_docker_launch_detail(
    runtime_kind: &str,
    runtime: &DockerItzgRuntimeConfig,
    launch_spec: &DockerLaunchSpec,
) -> DockerLaunchDetail {
    let docker_args_preview =
        sanitize_docker_args_preview(&build_docker_run_args(runtime, launch_spec));
    let command_preview = format_command_preview("docker", &docker_args_preview);

    DockerLaunchDetail {
        runtime_kind: runtime_kind.to_string(),
        image: runtime.image.clone(),
        image_tag: runtime.image_tag.clone(),
        container_name: runtime.container_name.clone(),
        cpuset_applied: launch_spec.cpuset_cpus.clone(),
        effective_max_memory: launch_spec.effective_max_memory,
        effective_min_memory: launch_spec.effective_min_memory,
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
    }
}

pub fn build_docker_effective_env(
    runtime: &DockerItzgRuntimeConfig,
    effective: &DockerEffectiveLaunchConfig,
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
    upsert_env(&mut env, "MEMORY", format_memory_env_value(effective.max_memory));
    upsert_env(&mut env, "MAX_MEMORY", format_memory_env_value(effective.max_memory));
    upsert_env(&mut env, "INIT_MEMORY", format_memory_env_value(effective.min_memory));
    if runtime.command_mode == DockerCommandMode::DockerStdio {
        upsert_env(&mut env, "CREATE_CONSOLE_IN_PIPE", "true".to_string());
    }

    let preset = effective.jvm_preset.preset.preset_args();
    let runtime_jvm_xx_override = env_contains_key(&env, "JVM_XX_OPTS");
    let runtime_jvm_opts_override = env_contains_key(&env, "JVM_OPTS");
    let user_has_apc = jvm_args_contain_active_processor_count(&effective.jvm_args);

    let active_processor_count = match resolve_docker_active_processor_count(&effective.cpu_policy)?
    {
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
        effective.jvm_args.iter().cloned(),
    );

    if !runtime_jvm_xx_override && !managed_jvm_xx_opts.is_empty() {
        upsert_env(&mut env, "JVM_XX_OPTS", managed_jvm_xx_opts.join(" "));
    }
    if !runtime_jvm_opts_override && !managed_jvm_opts.is_empty() {
        upsert_env(&mut env, "JVM_OPTS", managed_jvm_opts.join(" "));
    }

    let meta = DockerJvmSynthesisMeta {
        preset: effective.jvm_preset.preset.as_str(),
        jvm_opts_args_count: managed_jvm_opts.len(),
        jvm_xx_opts_args_count: managed_jvm_xx_opts.len(),
        jvm_opts_overridden_by_runtime_env: runtime_jvm_opts_override,
        jvm_xx_opts_overridden_by_runtime_env: runtime_jvm_xx_override,
        active_processor_count,
    };

    Ok((env, meta))
}

pub fn parse_memory_env_value(value: &str) -> Option<u32> {
    parse_memory_env_value_checked(value).unwrap_or_default()
}

pub fn parse_memory_env_value_checked(value: &str) -> Result<Option<u32>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let upper = trimmed.to_ascii_uppercase();
    if let Some(number) = upper.strip_suffix('G') {
        let parsed = number
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("内存值无效 '{}': {}", value, e))?;
        return Ok(Some(parsed.saturating_mul(1024)));
    }
    if let Some(number) = upper.strip_suffix('M') {
        let parsed = number
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("内存值无效 '{}': {}", value, e))?;
        return Ok(Some(parsed));
    }

    let parsed = upper
        .parse::<u32>()
        .map_err(|e| format!("内存值无效 '{}': {}", value, e))?;
    Ok(Some(parsed))
}

pub fn classify_docker_command_failure(message: &str) -> DockerCommandFailureKind {
    let lower = message.to_ascii_lowercase();
    if lower.contains("pull access denied")
        || lower.contains("requested access to the resource is denied")
        || lower.contains("unauthorized")
        || lower.contains("authentication required")
    {
        return DockerCommandFailureKind::Auth;
    }
    if lower.contains("manifest unknown")
        || lower.contains("not found") && lower.contains("manifest")
        || lower.contains("failed to resolve reference") && lower.contains("not found")
    {
        return DockerCommandFailureKind::ImageNotFound;
    }
    if lower.contains("cannot connect to the docker daemon")
        || lower.contains("failed to connect to the docker api")
        || lower.contains("open //./pipe/docker")
        || lower.contains("is the docker daemon running")
    {
        return DockerCommandFailureKind::DaemonUnavailable;
    }
    if lower.contains("connectex")
        || lower.contains("dial tcp")
        || lower.contains("i/o timeout")
        || lower.contains("tls handshake timeout")
        || lower.contains("no https proxy")
        || lower.contains("context deadline exceeded")
    {
        return DockerCommandFailureKind::Network;
    }
    if lower.contains("超时")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("deadline exceeded")
    {
        return DockerCommandFailureKind::Timeout;
    }

    DockerCommandFailureKind::Other
}

pub fn docker_error_indicates_missing_container(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("no such container")
        || lower.contains("no such object")
        || lower.contains("container") && lower.contains("not found")
}

pub fn render_docker_command_error(
    action: &str,
    output: &Output,
    image_ref: Option<&str>,
    container_name: Option<&str>,
) -> String {
    let stderr = stderr_text(output);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let raw = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("退出码: {:?}", output.status.code())
    };

    let failure_kind = classify_docker_command_failure(&raw);
    let image_detail = image_ref
        .map(|value| format!(" image={}", value))
        .unwrap_or_default();
    let container_detail = container_name
        .map(|value| format!(" container={}", value))
        .unwrap_or_default();
    let detail = format!("{}{}", image_detail, container_detail);

    match failure_kind {
        DockerCommandFailureKind::Network => format!(
            "{} 失败:{} 镜像拉取网络不可达或超时。请检查 Docker Hub / 镜像仓库连通性、代理配置，或预先执行 `sealantern docker pull <image[:tag]>`。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::ImageNotFound => format!(
            "{} 失败:{} 镜像或标签不存在。请检查 `--image` / `--image-tag`，例如可优先尝试 `itzg/minecraft-server:latest`。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::Auth => format!(
            "{} 失败:{} 镜像仓库认证或访问被拒绝。请确认已登录对应 registry，且镜像可见。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::DaemonUnavailable => format!(
            "{} 失败:{} Docker daemon 当前不可用。请先运行 `sealantern docker doctor` 或确认 Docker Desktop / Engine 已启动。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::Timeout => format!(
            "{} 失败:{} Docker 镜像探测超时。不会阻断本地记录创建，但建议先执行 `sealantern docker pull <image[:tag]>` 或稍后重试。原始输出: {}",
            action, detail, raw
        ),
        DockerCommandFailureKind::Other => format!("{} 失败:{} {}", action, detail, raw),
    }
}

pub fn ensure_docker_command_success(
    output: Output,
    action: &str,
    image_ref: Option<&str>,
    container_name: Option<&str>,
) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }

    Err(render_docker_command_error(action, &output, image_ref, container_name))
}

pub fn interpret_docker_image_inspect_outputs(
    image_ref: &str,
    local_output: &Output,
    manifest_output: &Output,
) -> Result<DockerImageInspectOutcome, String> {
    if local_output.status.success() {
        return Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached));
    }

    if manifest_output.status.success() {
        return Ok(DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable));
    }

    classify_manifest_inspect_outcome(image_ref, manifest_output)
}

pub fn classify_manifest_inspect_outcome(
    image_ref: &str,
    manifest_output: &Output,
) -> Result<DockerImageInspectOutcome, String> {
    let message = render_docker_command_error(
        "docker manifest inspect",
        manifest_output,
        Some(image_ref),
        None,
    );
    let failure_kind = classify_docker_command_failure(&message);

    match failure_kind {
        DockerCommandFailureKind::ImageNotFound | DockerCommandFailureKind::Auth => Err(message),
        DockerCommandFailureKind::Network
        | DockerCommandFailureKind::Timeout
        | DockerCommandFailureKind::DaemonUnavailable
        | DockerCommandFailureKind::Other => {
            Ok(DockerImageInspectOutcome::SoftFailure { failure_kind, message })
        }
    }
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
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

fn map_docker_cpu_policy_error(error: String) -> String {
    if let Some(rest) = error.strip_prefix("CPU 限制 count 模式") {
        return format!("Docker CPU policy count 模式{}", rest);
    }
    if let Some(rest) = error.strip_prefix("CPU 限制 explicit 模式") {
        return format!("Docker CPU policy explicit 模式{}", rest);
    }
    if error.starts_with("CPU 核心") {
        return format!("Docker {}", error);
    }

    error
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

#[cfg(test)]
mod tests {
    use super::{
        build_docker_effective_env, build_docker_launch_detail, build_docker_launch_spec,
        build_docker_run_args, classify_docker_command_failure, classify_manifest_inspect_outcome,
        docker_error_indicates_missing_container, docker_image_ref,
        docker_itzg_image_looks_compatible, ensure_docker_command_success, exit_status_from_raw,
        format_docker_image_reference, format_memory_env_value, format_published_port,
        format_volume_mount, interpret_docker_image_inspect_outputs, parse_command_mode,
        parse_docker_backend, parse_memory_env_value, parse_memory_env_value_checked,
        render_docker_command_error, requested_stop_timeout_secs,
        requested_stop_timeout_secs_checked, resolve_docker_active_processor_count,
        resolve_docker_cpuset, resolve_docker_image_and_tag, runtime_env_value,
        split_docker_image_reference_tag, validate_docker_itzg_image_compatibility,
        ActiveProcessorCountDecision, CpuPolicyConfig, CpuPolicyMode,
        CreateDockerItzgServerRequest, DockerBackendKind, DockerCommandFailureKind,
        DockerCommandMode, DockerEffectiveLaunchConfig, DockerImageAvailability,
        DockerImageInspectOutcome, DockerItzgRuntimeConfig, JvmPresetConfig, JvmPresetId,
        PublishedPort, RconConfig, VolumeMount,
    };
    use std::collections::BTreeMap;
    use std::process::Output;

    fn failed_output(stderr: &str) -> Output {
        Output {
            status: exit_status_from_raw(1),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    fn successful_output(stdout: &str) -> Output {
        Output {
            status: exit_status_from_raw(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    fn failed_output_with_stdout(stdout: &str) -> Output {
        Output {
            status: exit_status_from_raw(1),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn parse_docker_backend_accepts_engine_api_aliases() {
        assert_eq!(parse_docker_backend(Some("engine_api")).unwrap(), DockerBackendKind::EngineApi);
        assert_eq!(parse_docker_backend(Some("engine-api")).unwrap(), DockerBackendKind::EngineApi);
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
    fn format_memory_env_value_uses_m_for_zero_and_non_gigabyte_values() {
        assert_eq!(format_memory_env_value(0), "0M");
        assert_eq!(format_memory_env_value(1536), "1536M");
        assert_eq!(format_memory_env_value(2048), "2G");
    }

    #[test]
    fn docker_itzg_image_looks_compatible_accepts_expected_shapes() {
        assert!(docker_itzg_image_looks_compatible("itzg/minecraft-server"));
        assert!(docker_itzg_image_looks_compatible("registry.local:5000/itzg/minecraft-server"));
        assert!(docker_itzg_image_looks_compatible("minecraft-server"));
        assert!(!docker_itzg_image_looks_compatible("example/web"));
    }

    #[test]
    fn validate_docker_itzg_image_compatibility_rejects_non_minecraft_image() {
        let err = validate_docker_itzg_image_compatibility("example/web")
            .expect_err("non minecraft image should be rejected");
        assert!(err.contains("minecraft-server"));
    }

    #[test]
    fn split_docker_image_reference_tag_preserves_registry_port_without_tag() {
        let (image, tag) =
            split_docker_image_reference_tag("registry.local:5000/itzg/minecraft-server");
        assert_eq!(image, "registry.local:5000/itzg/minecraft-server");
        assert_eq!(tag, None);
    }

    #[test]
    fn resolve_docker_image_and_tag_supports_embedded_tag() {
        let (image, tag) = resolve_docker_image_and_tag(
            Some("registry.local:5000/paper:test"),
            None,
            "itzg/minecraft-server",
            "latest",
        )
        .expect("image ref should resolve");

        assert_eq!(image, "registry.local:5000/paper");
        assert_eq!(tag, "test");
    }

    #[test]
    fn resolve_docker_image_and_tag_rejects_digest_reference() {
        let err = resolve_docker_image_and_tag(
            Some("itzg/minecraft-server@sha256:deadbeef"),
            None,
            "itzg/minecraft-server",
            "latest",
        )
        .expect_err("digest refs should be rejected");

        assert!(err.contains("digest"));
    }

    #[test]
    fn resolve_docker_image_and_tag_rejects_empty_embedded_tag() {
        let err = resolve_docker_image_and_tag(
            Some("itzg/minecraft-server:"),
            None,
            "itzg/minecraft-server",
            "latest",
        )
        .expect_err("empty embedded tag should be rejected");

        assert!(err.contains("空 tag"), "unexpected error: {}", err);
    }

    #[test]
    fn resolve_docker_image_and_tag_rejects_empty_embedded_tag_after_registry_path() {
        let err = resolve_docker_image_and_tag(
            Some("registry.local:5000/itzg/minecraft-server:"),
            None,
            "itzg/minecraft-server",
            "latest",
        )
        .expect_err("empty embedded tag should be rejected");

        assert!(err.contains("空 tag"), "unexpected error: {}", err);
    }

    #[test]
    fn format_docker_image_reference_formats_repo_and_tag() {
        assert_eq!(
            format_docker_image_reference("itzg/minecraft-server", "latest"),
            "itzg/minecraft-server:latest"
        );
    }

    #[test]
    fn classifies_manifest_unknown_as_image_not_found() {
        let failure = classify_docker_command_failure("manifest unknown: manifest unknown");
        assert_eq!(failure, DockerCommandFailureKind::ImageNotFound);
    }

    #[test]
    fn render_docker_command_error_prefers_stdout_when_stderr_is_empty() {
        let output = failed_output_with_stdout("manifest unknown: manifest unknown");
        let message = render_docker_command_error(
            "docker manifest inspect",
            &output,
            Some("itzg/minecraft-server:bad-tag"),
            None,
        );

        assert!(message.contains("镜像或标签不存在"));
        assert!(message.contains("manifest unknown"));
    }

    #[test]
    fn classify_manifest_inspect_outcome_returns_soft_failure_for_network_errors() {
        let outcome = classify_manifest_inspect_outcome(
            "itzg/minecraft-server:latest",
            &failed_output("dial tcp 1.2.3.4:443: connectex: timeout"),
        )
        .expect("network failures should downgrade to soft failure");

        match outcome {
            DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
                assert_eq!(failure_kind, DockerCommandFailureKind::Network);
                assert!(message.contains("镜像拉取网络不可达或超时"));
            }
            other => panic!("expected soft failure, got {:?}", other),
        }
    }

    #[test]
    fn interpret_docker_image_inspect_outputs_prefers_local_cached_success() {
        let outcome = interpret_docker_image_inspect_outputs(
            "itzg/minecraft-server:latest",
            &successful_output("[{\"Id\":\"sha256:abc\"}]"),
            &failed_output("manifest unknown: manifest unknown"),
        )
        .expect("local inspect success should short-circuit");

        assert_eq!(
            outcome,
            DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached)
        );
    }

    #[test]
    fn ensure_docker_command_success_surfaces_rendered_error() {
        let err = ensure_docker_command_success(
            failed_output("unauthorized: authentication required"),
            "docker pull",
            Some("itzg/minecraft-server:latest"),
            Some("sea-test"),
        )
        .expect_err("failed docker command should surface user-visible error");

        assert!(err.contains("镜像仓库认证或访问被拒绝"));
        assert!(err.contains("sea-test"));
    }

    #[test]
    fn detects_missing_container_errors() {
        assert!(docker_error_indicates_missing_container(
            "Error response from daemon: No such container: sea-test"
        ));
        assert!(docker_error_indicates_missing_container("container sea-test not found"));
    }

    #[test]
    fn format_published_port_defaults_protocol_to_tcp() {
        let port = PublishedPort {
            host_port: 25575,
            container_port: 25575,
            protocol: String::new(),
        };

        assert_eq!(format_published_port(&port), "25575:25575/tcp");
    }

    #[test]
    fn format_volume_mount_preserves_read_only_suffix() {
        let mount = VolumeMount {
            source: "E:/plugins".to_string(),
            target: "/data/plugins".to_string(),
            read_only: true,
        };

        assert_eq!(format_volume_mount(&mount), "E:/plugins:/data/plugins:ro");
    }

    #[test]
    fn runtime_env_value_and_stop_timeout_are_case_insensitive() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "latest".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::from([("stop_duration".to_string(), "180".to_string())]),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::Rcon,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };

        assert_eq!(runtime_env_value(&runtime, "STOP_DURATION"), Some("180"));
        assert_eq!(requested_stop_timeout_secs(&runtime), Some(180));
        assert_eq!(requested_stop_timeout_secs_checked(&runtime).unwrap(), Some(180));
    }

    #[test]
    fn requested_stop_timeout_secs_checked_rejects_zero_value() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "latest".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::from([("STOP_DURATION".to_string(), "0".to_string())]),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::Rcon,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };

        let error = requested_stop_timeout_secs_checked(&runtime)
            .expect_err("zero STOP_DURATION should not be silently downgraded");

        assert!(error.contains("必须大于 0 秒"), "unexpected error: {}", error);
        assert_eq!(requested_stop_timeout_secs(&runtime), None);
    }

    #[test]
    fn requested_stop_timeout_secs_checked_rejects_invalid_value() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "latest".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::from([("STOP_DURATION".to_string(), "abc".to_string())]),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::Rcon,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };

        let error = requested_stop_timeout_secs_checked(&runtime)
            .expect_err("invalid STOP_DURATION should not be silently downgraded");

        assert!(error.contains("STOP_DURATION 无效"), "unexpected error: {}", error);
        assert!(error.contains("abc"), "unexpected error: {}", error);
        assert_eq!(requested_stop_timeout_secs(&runtime), None);
    }

    #[test]
    fn parse_memory_env_value_supports_g_and_m() {
        assert_eq!(parse_memory_env_value("4G"), Some(4096));
        assert_eq!(parse_memory_env_value("1536M"), Some(1536));
        assert_eq!(parse_memory_env_value("2048"), Some(2048));
        assert_eq!(parse_memory_env_value_checked("4G").unwrap(), Some(4096));
        assert_eq!(parse_memory_env_value_checked("1536M").unwrap(), Some(1536));
        assert_eq!(parse_memory_env_value_checked("2048").unwrap(), Some(2048));
    }

    #[test]
    fn parse_memory_env_value_checked_rejects_invalid_values() {
        let error = parse_memory_env_value_checked("4X")
            .expect_err("invalid memory suffix should not be silently downgraded");

        assert!(error.contains("内存值无效"), "unexpected error: {}", error);
        assert!(error.contains("4X"), "unexpected error: {}", error);
        assert_eq!(parse_memory_env_value("4X"), None);
    }

    #[test]
    fn parse_memory_env_value_checked_treats_blank_as_absent() {
        assert_eq!(parse_memory_env_value_checked("   ").unwrap(), None);
        assert_eq!(parse_memory_env_value("   "), None);
    }

    #[test]
    fn resolve_docker_cpuset_supports_count_and_explicit_modes() {
        assert_eq!(
            resolve_docker_cpuset(&CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(4),
                explicit_set: None,
                sync_active_processor_count: true,
            })
            .unwrap(),
            Some("0-3".to_string())
        );

        assert_eq!(
            resolve_docker_cpuset(&CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("0-3,6,7".to_string()),
                sync_active_processor_count: true,
            })
            .unwrap(),
            Some("0-3,6-7".to_string())
        );
    }

    #[test]
    fn resolve_docker_active_processor_count_tracks_effective_core_count() {
        assert_eq!(
            resolve_docker_active_processor_count(&CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(3),
                explicit_set: None,
                sync_active_processor_count: true,
            })
            .unwrap(),
            Some(3)
        );

        assert_eq!(
            resolve_docker_active_processor_count(&CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("1,3,5".to_string()),
                sync_active_processor_count: true,
            })
            .unwrap(),
            Some(3)
        );
    }

    #[test]
    fn resolve_docker_cpuset_rejects_invalid_shapes() {
        assert!(resolve_docker_cpuset(&CpuPolicyConfig {
            mode: CpuPolicyMode::Count,
            count: Some(0),
            explicit_set: None,
            sync_active_processor_count: true,
        })
        .is_err());

        assert!(resolve_docker_cpuset(&CpuPolicyConfig {
            mode: CpuPolicyMode::Explicit,
            count: None,
            explicit_set: Some("3-1".to_string()),
            sync_active_processor_count: true,
        })
        .is_err());
    }

    #[test]
    fn build_docker_effective_env_synthesizes_expected_jvm_layers() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::new(),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::DockerStdio,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };
        let effective = DockerEffectiveLaunchConfig {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Dfoo=bar".to_string(), "-XX:+UseStringDeduplication".to_string()],
            cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            jvm_preset: JvmPresetConfig { preset: JvmPresetId::G1Basic },
        };

        let (env, meta) = build_docker_effective_env(&runtime, &effective).unwrap();
        let jvm_opts = env.iter().find(|(k, _)| k == "JVM_OPTS").unwrap().1.clone();
        let jvm_xx_opts = env
            .iter()
            .find(|(k, _)| k == "JVM_XX_OPTS")
            .unwrap()
            .1
            .clone();

        assert_eq!(env.iter().find(|(k, _)| k == "MAX_MEMORY").unwrap().1, "4G");
        assert_eq!(env.iter().find(|(k, _)| k == "INIT_MEMORY").unwrap().1, "2G");
        assert!(env
            .iter()
            .any(|(k, v)| k == "CREATE_CONSOLE_IN_PIPE" && v == "true"));
        assert!(jvm_opts.contains("-Dfoo=bar"));
        assert!(jvm_xx_opts.contains("-XX:+UseG1GC"));
        assert!(jvm_xx_opts.contains("-XX:ActiveProcessorCount=2"));
        assert!(jvm_xx_opts.contains("-XX:+UseStringDeduplication"));
        assert_eq!(meta.preset, "g1_basic");
        assert_eq!(meta.active_processor_count, ActiveProcessorCountDecision::Injected(2));
    }

    #[test]
    fn build_docker_effective_env_respects_runtime_env_takeover() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::from([
                ("JVM_OPTS".to_string(), "-Dmanual=true".to_string()),
                ("JVM_XX_OPTS".to_string(), "-XX:ActiveProcessorCount=99".to_string()),
            ]),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::Rcon,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };
        let effective = DockerEffectiveLaunchConfig {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Dfoo=bar".to_string()],
            cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
        };

        let (env, meta) = build_docker_effective_env(&runtime, &effective).unwrap();

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
    fn build_docker_run_args_includes_cpuset_and_runtime_env() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::new(),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::DockerStdio,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };
        let effective = DockerEffectiveLaunchConfig {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Dfoo=bar".to_string()],
            cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            jvm_preset: JvmPresetConfig::default(),
        };

        let spec = build_docker_launch_spec(&runtime, &effective).unwrap();
        let args = build_docker_run_args(&runtime, &spec);

        assert!(args.windows(2).any(|pair| pair == ["--cpuset-cpus", "0-1"]));
        assert!(args.iter().any(|arg| arg == "itzg/minecraft-server:java21"));
        assert!(args.iter().any(|arg| arg == "CREATE_CONSOLE_IN_PIPE=true"));
    }

    #[test]
    fn build_docker_launch_detail_redacts_sensitive_env_preview() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::from([("RCON_PASSWORD".to_string(), "secret-pass".to_string())]),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::DockerStdio,
            rcon: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };
        let effective = DockerEffectiveLaunchConfig {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Dfoo=bar".to_string()],
            cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
        };

        let spec = build_docker_launch_spec(&runtime, &effective).unwrap();
        let detail = build_docker_launch_detail("docker_itzg", &runtime, &spec);
        let joined = detail.docker_args_preview.join(" ");

        assert_eq!(detail.runtime_kind, "docker_itzg");
        assert_eq!(detail.cpuset_applied.as_deref(), Some("0-1"));
        assert!(detail
            .command_preview
            .contains("docker run -d --name sea-test --cpuset-cpus 0-1"));
        assert!(joined.contains("RCON_PASSWORD=<redacted>"));
        assert!(joined.contains("JVM_OPTS=-Dfoo=bar"));
    }

    #[test]
    fn docker_image_ref_formats_runtime_reference() {
        let runtime = DockerItzgRuntimeConfig {
            image: "itzg/minecraft-server".to_string(),
            image_tag: "java21".to_string(),
            container_name: "sea-test".to_string(),
            type_value: "PAPER".to_string(),
            version: "1.21.1".to_string(),
            data_dir_mount: "E:/docker/paper".to_string(),
            published_game_port: 25565,
            env: BTreeMap::new(),
            extra_ports: Vec::new(),
            volume_mounts: Vec::new(),
            docker_backend_kind: DockerBackendKind::Cli,
            command_mode: DockerCommandMode::Rcon,
            rcon: Some(RconConfig {
                host: "127.0.0.1".to_string(),
                port: 25575,
                password: "secret".to_string(),
            }),
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };

        assert_eq!(docker_image_ref(&runtime), "itzg/minecraft-server:java21");
    }

    #[test]
    fn create_docker_request_round_trips_core_fields() {
        let request = CreateDockerItzgServerRequest {
            name: "paper-prod".to_string(),
            aliases: vec!["main".to_string()],
            core_type: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            port: 25565,
            runtime: DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "latest".to_string(),
                container_name: "sea-paper".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/paper".to_string(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: Vec::new(),
                volume_mounts: Vec::new(),
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            },
        };

        assert_eq!(request.name, "paper-prod");
        assert_eq!(request.aliases, vec!["main"]);
        assert_eq!(request.runtime.container_name, "sea-paper");
    }
}
