use super::{DockerContainerState, RuntimeStatusSnapshot, DOCKER_ITZG_RUNTIME_KIND};
use crate::models::server::{
    DockerItzgRuntimeConfig, PublishedPort, ServerInstance, ServerStatus, VolumeMount,
};
use crate::utils::docker_cli::docker_error_indicates_missing_container;
use std::path::Path;
use std::process::Output;

pub(super) fn ensure_runtime_path_ready(server: &ServerInstance) -> Result<(), String> {
    let path = Path::new(&server.path);
    std::fs::create_dir_all(path)
        .map_err(|e| format!("创建 Docker 数据目录失败 ({}): {}", path.display(), e))
}

pub(super) fn build_effective_env(runtime: &DockerItzgRuntimeConfig) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = runtime
        .env
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

    let eula_value = env_value_or_default(&env, "EULA", "TRUE");
    upsert_env(&mut env, "TYPE", runtime.type_value.clone());
    upsert_env(&mut env, "VERSION", runtime.version.clone());
    upsert_env(&mut env, "EULA", eula_value);
    if runtime.command_mode == crate::models::server::DockerCommandMode::DockerStdio {
        upsert_env(&mut env, "CREATE_CONSOLE_IN_PIPE", "true".to_string());
    }

    env
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
