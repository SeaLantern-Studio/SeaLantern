use crate::models::server::{DockerItzgRuntimeConfig, ServerInstance, ServerRuntimeConfig};
use crate::services::server::log_pipeline;
use crate::utils::docker_cli::{
    docker_error_indicates_missing_container, docker_executable_path, render_docker_command_error,
};

use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

use super::server_shared::{trace_cli_action, trace_cli_error};

const DOCKER_FOLLOW_TAIL_LINES: usize = 0;

pub(super) fn read_recent_server_logs(
    server: &ServerInstance,
    lines: usize,
) -> Result<Vec<String>, String> {
    read_recent_server_logs_with(server, lines, read_recent_docker_logs, |server_id, limit| {
        log_pipeline::get_logs_checked(server_id, 0, Some(limit))
    })
}

pub(super) fn read_recent_cli_logs(server: &ServerInstance) -> Result<Vec<String>, String> {
    read_recent_server_logs(server, 20)
}

pub(super) fn follow_server_logs(server: &ServerInstance, interval_ms: u64) -> Result<(), String> {
    match &server.runtime {
        ServerRuntimeConfig::DockerItzg(runtime) => follow_docker_logs(server, runtime),
        ServerRuntimeConfig::Local(_) => follow_local_server_logs(server, interval_ms),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn read_recent_server_logs_with<FDocker, FLocal>(
    server: &ServerInstance,
    lines: usize,
    mut read_docker: FDocker,
    mut read_local: FLocal,
) -> Result<Vec<String>, String>
where
    FDocker: FnMut(&DockerItzgRuntimeConfig, usize) -> Result<Vec<String>, String>,
    FLocal: FnMut(&str, usize) -> Result<Vec<String>, String>,
{
    match &server.runtime {
        ServerRuntimeConfig::DockerItzg(runtime) => {
            let fallback = read_local(&server.id, lines)?;
            match read_docker(runtime, lines) {
                Ok(logs) if !logs.is_empty() => Ok(logs),
                Ok(_logs) if !fallback.is_empty() => Ok(fallback),
                Ok(logs) => Ok(logs),
                Err(err) if docker_error_indicates_missing_container(&err) => {
                    let mut logs = render_missing_container_log_lines(&err);
                    if !fallback.is_empty() {
                        logs.push("[Sea Lantern] 已附加当前缓存日志（如果有）。".to_string());
                        logs.extend(fallback);
                    }
                    Ok(logs)
                }
                Err(err) if !fallback.is_empty() => {
                    let mut logs = vec![format!(
                        "[Sea Lantern] Docker 实时日志读取失败，已回退到缓存日志: {}",
                        err
                    )];
                    logs.extend(fallback);
                    Ok(logs)
                }
                Err(err) => Err(err),
            }
        }
        ServerRuntimeConfig::Local(_) => read_local(&server.id, lines),
    }
}

fn follow_local_server_logs(server: &ServerInstance, interval_ms: u64) -> Result<(), String> {
    println!(
        "进入日志持续跟踪: {}，轮询间隔={}ms，按 Ctrl+C 结束。",
        server.name, interval_ms
    );

    let mut offset = log_pipeline::get_logs_checked(&server.id, 0, None)?.len();
    loop {
        let next_logs = log_pipeline::get_logs_checked(&server.id, offset, None)?;
        if !next_logs.is_empty() {
            offset += next_logs.len();
            for line in next_logs {
                println!("{}", line);
            }
        }
        thread::sleep(Duration::from_millis(interval_ms));
    }
}

fn read_recent_docker_logs(
    runtime: &DockerItzgRuntimeConfig,
    lines: usize,
) -> Result<Vec<String>, String> {
    trace_cli_action(
        "manage_logs_docker_snapshot",
        &format!("container={} lines={}", runtime.container_name, lines),
    );
    let docker_path = docker_executable_path()?;
    let output = Command::new(docker_path)
        .args(build_docker_logs_args(&runtime.container_name, lines, false))
        .output()
        .map_err(|err| format!("执行 docker logs 失败: {}", err))?;

    if !output.status.success() {
        let error = render_docker_command_error(
            "docker logs",
            &output,
            None,
            Some(&runtime.container_name),
        );
        trace_cli_error(
            "manage_logs_docker_snapshot_failed",
            &format!("container={}", runtime.container_name),
            &error,
        );
        return Err(error);
    }

    Ok(output_to_lines(&output))
}

fn follow_docker_logs(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
) -> Result<(), String> {
    trace_cli_action(
        "manage_logs_docker_follow",
        &format!("server_id={} container={}", server.id, runtime.container_name),
    );
    println!(
        "进入 Docker 日志持续跟踪: {}，container={}，按 Ctrl+C 结束。",
        server.name, runtime.container_name
    );

    if !docker_container_exists(&runtime.container_name)? {
        trace_cli_action(
            "manage_logs_docker_follow_skipped_missing_container",
            &format!("server_id={} container={}", server.id, runtime.container_name),
        );
        for line in render_missing_container_log_lines(&format!(
            "docker logs -f 目标容器不存在: {}",
            runtime.container_name
        )) {
            println!("{}", line);
        }
        return Ok(());
    }

    let docker_path = docker_executable_path()?;
    let status = Command::new(docker_path)
        .args(build_docker_logs_args(&runtime.container_name, DOCKER_FOLLOW_TAIL_LINES, true))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| format!("执行 docker logs -f 失败: {}", err))?;

    if status.success() {
        return Ok(());
    }

    let error = format!(
        "docker logs -f 退出异常: container={} exit_code={:?}",
        runtime.container_name,
        status.code()
    );
    trace_cli_error(
        "manage_logs_docker_follow_failed",
        &format!("container={}", runtime.container_name),
        &error,
    );
    Err(error)
}

fn build_docker_logs_args(container_name: &str, lines: usize, follow: bool) -> Vec<String> {
    let mut args = vec!["logs".to_string()];
    if follow {
        args.push("-f".to_string());
    }
    args.push("--tail".to_string());
    args.push(lines.to_string());
    args.push(container_name.to_string());
    args
}

fn output_to_lines(output: &Output) -> Vec<String> {
    let mut lines = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    lines.extend(
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .map(|line| line.to_string()),
    );
    lines
}

fn docker_container_exists(container_name: &str) -> Result<bool, String> {
    let docker_path = docker_executable_path()?;
    let output = Command::new(docker_path)
        .args(["inspect", container_name, "--format", "{{.Name}}"])
        .output()
        .map_err(|err| format!("执行 docker inspect 失败: {}", err))?;

    if output.status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if docker_error_indicates_missing_container(&stderr) {
        return Ok(false);
    }

    Err(if stderr.is_empty() {
        format!(
            "docker inspect 失败: container={} exit_code={:?}",
            container_name,
            output.status.code()
        )
    } else {
        format!("docker inspect 失败: container={} {}", container_name, stderr)
    })
}

fn render_missing_container_log_lines(detail: &str) -> Vec<String> {
    vec![
        "[Sea Lantern] 当前 Docker 容器尚未创建或尚未首次启动，因此没有可读取的容器日志。"
            .to_string(),
        format!("[Sea Lantern] 详情: {}", detail),
        "[Sea Lantern] 可先执行 `sealantern server start <id>` 或 `sealantern docker pull <image[:tag]>` 后重试。"
            .to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        build_docker_logs_args, docker_container_exists, read_recent_cli_logs,
        read_recent_server_logs_with, render_missing_container_log_lines,
    };
    use crate::models::server::{
        DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig, LocalRuntimeConfig,
        ServerInstance, ServerRuntimeConfig,
    };
    use std::collections::BTreeMap;

    fn sample_local_server() -> ServerInstance {
        ServerInstance {
            id: "local-1".to_string(),
            name: "local-one".to_string(),
            aliases: vec![],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: "E:/servers/local-one".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/local-one/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        }
    }

    fn sample_docker_server() -> ServerInstance {
        ServerInstance {
            id: "docker-1".to_string(),
            name: "docker-one".to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/docker/docker-one".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sea-docker-one".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/docker-one".to_string(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: vec![],
                volume_mounts: vec![],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: Vec::new(),
                cpu_policy: crate::models::server::CpuPolicyConfig::default(),
                jvm_preset: crate::models::server::JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn build_docker_logs_args_supports_tail_and_follow() {
        let args = build_docker_logs_args("sea-docker-one", 25, true);
        assert_eq!(args, vec!["logs", "-f", "--tail", "25", "sea-docker-one"]);
    }

    #[test]
    fn read_recent_server_logs_uses_local_cache_for_local_runtime() {
        let logs = read_recent_server_logs_with(
            &sample_local_server(),
            20,
            |_, _| Err("docker should not be used".to_string()),
            |_, _| Ok(vec!["local-line".to_string()]),
        )
        .expect("local logs should resolve");

        assert_eq!(logs, vec!["local-line".to_string()]);
    }

    #[test]
    fn read_recent_server_logs_prefers_docker_snapshot_for_docker_runtime() {
        let logs = read_recent_server_logs_with(
            &sample_docker_server(),
            20,
            |runtime, lines| {
                assert_eq!(runtime.container_name, "sea-docker-one");
                assert_eq!(lines, 20);
                Ok(vec!["docker-line".to_string()])
            },
            |_, _| Ok(vec!["cached-line".to_string()]),
        )
        .expect("docker logs should resolve");

        assert_eq!(logs, vec!["docker-line".to_string()]);
    }

    #[test]
    fn read_recent_server_logs_falls_back_to_cache_when_docker_snapshot_fails() {
        let logs = read_recent_server_logs_with(
            &sample_docker_server(),
            20,
            |_, _| Err("docker logs unavailable".to_string()),
            |_, _| Ok(vec!["cached-line".to_string()]),
        )
        .expect("cached logs should resolve");

        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Docker 实时日志读取失败"));
        assert_eq!(logs[1], "cached-line");
    }

    #[test]
    fn read_recent_server_logs_returns_friendly_message_for_missing_container() {
        let logs = read_recent_server_logs_with(
            &sample_docker_server(),
            20,
            |_, _| Err("docker logs 失败: container=sea-docker-one Error response from daemon: No such container: sea-docker-one".to_string()),
            |_, _| Ok(vec![]),
        )
        .expect("missing container should become friendly output");

        assert!(logs[0].contains("尚未创建或尚未首次启动"));
        assert!(logs[1].contains("No such container"));
        assert!(logs[2].contains("sealantern server start <id>"));
    }

    #[test]
    fn render_missing_container_log_lines_remains_actionable() {
        let lines = render_missing_container_log_lines(
            "docker logs -f 失败: container=sea-docker-one Error response from daemon: No such container: sea-docker-one",
        );

        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("尚未创建或尚未首次启动"));
        assert!(lines[1].contains("No such container"));
        assert!(lines[2].contains("sealantern docker pull <image[:tag]>"));
    }

    #[test]
    fn read_recent_server_logs_missing_container_message_stays_start_actionable() {
        let logs =
            render_missing_container_log_lines("docker logs -f 目标容器不存在: sea-docker-one");

        assert!(logs[0].contains("尚未创建或尚未首次启动"));
        assert!(logs[2].contains("sealantern server start <id>"));
    }

    #[test]
    fn docker_container_exists_symbol_is_present_for_runtime_preflight() {
        let fn_ptr: fn(&str) -> Result<bool, String> = docker_container_exists;
        let _ = fn_ptr;
    }

    #[test]
    fn read_recent_cli_logs_returns_without_panicking() {
        let _ = read_recent_cli_logs(&sample_local_server());
    }

    #[test]
    fn read_recent_server_logs_surfaces_local_cache_failures() {
        let error = read_recent_server_logs_with(
            &sample_local_server(),
            20,
            |_, _| Err("docker should not be used".to_string()),
            |_, _| Err("log db broken".to_string()),
        )
        .expect_err("local log cache failure should not be silently downgraded to empty output");

        assert_eq!(error, "log db broken");
    }
}
