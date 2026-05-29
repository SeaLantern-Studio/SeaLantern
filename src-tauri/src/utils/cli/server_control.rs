use std::thread;
use std::time::Duration;

use crate::models::server::{ServerInstance, ServerStatus, ServerStatusInfo};
use crate::services::global;
use crate::services::server::manager::StartServerReport;
use crate::utils::server_status::{
    status_detail_indicates_running, status_is_terminal_start_ready,
};

use super::server_feedback::{
    render_post_start_feedback_lines, render_post_stop_feedback_lines,
    render_runtime_start_failure_hint_lines, render_runtime_stop_failure_hint_lines,
};
use super::server_shared::{trace_cli_action, trace_cli_error};

const RESTART_STOP_POLL_INTERVAL_MS: u64 = 500;
const START_OBSERVE_POLL_INTERVAL_MS: u64 = 700;
const LOCAL_START_OBSERVE_TIMEOUT_SECS: u64 = 6;
const DOCKER_START_OBSERVE_TIMEOUT_SECS: u64 = 20;
pub(super) const DEFAULT_RESTART_STOP_TIMEOUT_SECS: u64 = 30;

pub(super) fn restart_server_with_wait(
    server: &ServerInstance,
    action_scope: &str,
    timeout_secs: u64,
) -> Result<(), String> {
    let manager = global::server_manager();
    let before = manager.get_server_status(&server.id);
    trace_cli_action(
        &format!("{}_precheck", action_scope),
        &format!(
            "server_id={} status={} detail={}",
            server.id,
            before.status.as_str(),
            before.detail_message.as_deref().unwrap_or("")
        ),
    );

    if status_requires_running_wait(&before) {
        request_stop_with_feedback_with_options(
            server,
            action_scope,
            "已请求停止服务器，等待退出...",
            false,
        )?;
        if let Err(err) = wait_for_server_stop(&server.id, timeout_secs, action_scope) {
            print_stop_failure_feedback(server, &err);
            return Err(err);
        }
        trace_cli_action(
            &format!("{}_wait_stopped", action_scope),
            &format!("server_id={}", server.id),
        );
    }

    trace_cli_action(
        &format!("{}_start_trigger", action_scope),
        &format!("server_id={}", server.id),
    );
    let report = match manager.start_server(&server.id) {
        Ok(report) => report,
        Err(err) => {
            print_start_failure_feedback(server, &err);
            return Err(err);
        }
    };
    print_start_report(&report, "服务器正在重新启动...");
    if report.skipped_existing_state {
        trace_cli_action(
            &format!("{}_start_skip_existing_state", action_scope),
            &format!("server_id={}", report.server_id),
        );
        return Ok(());
    }
    let observe_timeout_secs = start_observe_timeout_secs(server);
    if let Some(observation_error) =
        observe_server_start_briefly(&report.server_id, action_scope, observe_timeout_secs)
    {
        print_start_failure_feedback(server, &observation_error);
        return Err(observation_error);
    }
    Ok(())
}

pub(super) fn start_server_with_feedback(
    server: &ServerInstance,
    action_scope: &str,
    headline: &str,
) -> Result<StartServerReport, String> {
    trace_cli_action(
        &format!("{}_start_trigger", action_scope),
        &format!("server_id={}", server.id),
    );
    let report = match global::server_manager().start_server(&server.id) {
        Ok(report) => report,
        Err(err) => {
            print_start_failure_feedback(server, &err);
            return Err(err);
        }
    };
    print_start_report(&report, headline);
    if report.skipped_existing_state {
        trace_cli_action(
            &format!("{}_start_skip_existing_state", action_scope),
            &format!("server_id={}", report.server_id),
        );
        return Ok(report);
    }
    let observe_timeout_secs = start_observe_timeout_secs(server);
    if let Some(observation_error) =
        observe_server_start_briefly(&report.server_id, action_scope, observe_timeout_secs)
    {
        print_start_failure_feedback(server, &observation_error);
        return Err(observation_error);
    }
    Ok(report)
}

fn start_observe_timeout_secs(server: &ServerInstance) -> u64 {
    match server.runtime_kind.as_str() {
        "docker_itzg" => DOCKER_START_OBSERVE_TIMEOUT_SECS,
        _ => LOCAL_START_OBSERVE_TIMEOUT_SECS,
    }
}

fn print_start_failure_feedback(server: &ServerInstance, error: &str) {
    trace_cli_error(
        "start_report_failed",
        &format!("server_id={} runtime={}", server.id, server.runtime_kind),
        error,
    );

    eprintln!("服务器启动失败: id={} name={}", server.id, server.name);
    eprintln!("失败原因: {}", error);
    for line in render_runtime_start_failure_hint_lines(server, error) {
        eprintln!("{}", line);
    }
}

fn print_stop_failure_feedback(server: &ServerInstance, error: &str) {
    trace_cli_error(
        "stop_report_failed",
        &format!("server_id={} runtime={}", server.id, server.runtime_kind),
        error,
    );

    eprintln!("服务器停止/重启等待失败: id={} name={}", server.id, server.name);
    eprintln!("失败原因: {}", error);
    for line in render_runtime_stop_failure_hint_lines(server, error) {
        eprintln!("{}", line);
    }
}

pub(super) fn request_stop_with_feedback(
    server: &ServerInstance,
    action_scope: &str,
    headline: &str,
) -> Result<ServerStatusInfo, String> {
    request_stop_with_feedback_with_options(server, action_scope, headline, true)
}

pub(super) fn stop_server_with_feedback(
    server: &ServerInstance,
    action_scope: &str,
    headline: &str,
) -> Result<ServerStatusInfo, String> {
    stop_server_with_feedback_with_options(server, action_scope, headline, true)
}

pub(super) fn print_start_report(report: &StartServerReport, headline: &str) {
    println!("{} {}", report.server_name, headline);
    if report.skipped_existing_state {
        println!("检测到目标当前已处于运行/过渡态，已跳过重复启动。");
    }
    if let Some(fallback) = &report.fallback {
        println!(
            "已触发启动回退: {} -> {} ({})",
            fallback.from_mode, fallback.to_mode, fallback.reason
        );
    }

    let manager = global::server_manager();
    let server = manager
        .find_server_clone_optional(&report.server_id)
        .ok()
        .flatten();
    let status = manager.get_server_status(&report.server_id);

    for line in render_post_start_feedback_lines(server.as_ref(), &status) {
        println!("{}", line);
    }

    trace_cli_action(
        "start_report",
        &format!(
            "server_id={} status={} pid={} detail={} error={}",
            report.server_id,
            status.status.as_str(),
            status
                .pid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            status.detail_message.as_deref().unwrap_or(""),
            status.error_message.as_deref().unwrap_or("")
        ),
    );

    println!("可继续执行: sealantern server status {}", report.server_id);
    println!("如需查看启动日志: sealantern server logs {} --lines 50", report.server_id);
}

fn observe_server_start_briefly(
    server_id: &str,
    action_scope: &str,
    timeout_secs: u64,
) -> Option<String> {
    let manager = global::server_manager();
    let started = std::time::Instant::now();
    let mut last_snapshot = manager.get_server_status(server_id);

    trace_cli_action(
        &format!("{}_start_observe_begin", action_scope),
        &format!(
            "server_id={} status={} detail={} error={}",
            server_id,
            last_snapshot.status.as_str(),
            last_snapshot.detail_message.as_deref().unwrap_or(""),
            last_snapshot.error_message.as_deref().unwrap_or("")
        ),
    );

    while started.elapsed().as_secs() < timeout_secs {
        if start_observation_is_terminal(&last_snapshot) {
            break;
        }

        thread::sleep(Duration::from_millis(START_OBSERVE_POLL_INTERVAL_MS));
        let next = manager.get_server_status(server_id);
        if status_snapshot_changed(&last_snapshot, &next) {
            println!(
                "启动观察: status={} detail={} error={}",
                next.status.as_str(),
                next.detail_message.as_deref().unwrap_or("-"),
                next.error_message.as_deref().unwrap_or("-")
            );
            trace_cli_action(
                &format!("{}_start_observe_update", action_scope),
                &format!(
                    "server_id={} status={} detail={} error={}",
                    server_id,
                    next.status.as_str(),
                    next.detail_message.as_deref().unwrap_or(""),
                    next.error_message.as_deref().unwrap_or("")
                ),
            );
        }
        last_snapshot = next;
    }

    trace_cli_action(
        &format!("{}_start_observe_end", action_scope),
        &format!(
            "server_id={} status={} detail={} error={}",
            server_id,
            last_snapshot.status.as_str(),
            last_snapshot.detail_message.as_deref().unwrap_or(""),
            last_snapshot.error_message.as_deref().unwrap_or("")
        ),
    );

    print_start_observation_follow_up(server_id, &last_snapshot, timeout_secs);

    render_start_observation_terminal_error(server_id, &last_snapshot)
}

fn print_start_observation_follow_up(
    server_id: &str,
    status: &ServerStatusInfo,
    timeout_secs: u64,
) {
    if status.status == ServerStatus::Running && !start_observation_is_terminal(status) {
        println!("启动观察结束: 容器/进程仍在预热，约 {}s 内尚未进入最终就绪态。", timeout_secs);
        if let Some(detail) = status.detail_message.as_deref() {
            println!("启动观察详情: {}", detail);
        }
        println!(
            "可继续执行: sealantern server status {} 观察 health 变化；如目标是 itzg，请等待 health=healthy。",
            server_id
        );
    }
}

fn start_observation_is_terminal(status: &ServerStatusInfo) -> bool {
    status_is_terminal_start_ready(status)
}

fn status_snapshot_changed(previous: &ServerStatusInfo, next: &ServerStatusInfo) -> bool {
    previous.status != next.status
        || previous.pid != next.pid
        || previous.detail_message != next.detail_message
        || previous.error_message != next.error_message
}

fn render_start_observation_terminal_error(
    server_id: &str,
    status: &ServerStatusInfo,
) -> Option<String> {
    match status.status {
        ServerStatus::Error => Some(format!(
            "启动观察确认失败: server_id={} status=error detail={} error={}",
            server_id,
            status.detail_message.as_deref().unwrap_or(""),
            status.error_message.as_deref().unwrap_or("")
        )),
        ServerStatus::Stopped => Some(format!(
            "启动观察确认失败: server_id={} status=stopped detail={} error={}",
            server_id,
            status.detail_message.as_deref().unwrap_or(""),
            status.error_message.as_deref().unwrap_or("")
        )),
        _ => None,
    }
}

fn request_stop_with_feedback_with_options(
    server: &ServerInstance,
    action_scope: &str,
    headline: &str,
    show_follow_up: bool,
) -> Result<ServerStatusInfo, String> {
    let manager = global::server_manager();
    let before = manager.get_server_status(&server.id);
    trace_cli_action(
        &format!("{}_stop_precheck", action_scope),
        &format!(
            "server_id={} status={} detail={} error={}",
            server.id,
            before.status.as_str(),
            before.detail_message.as_deref().unwrap_or(""),
            before.error_message.as_deref().unwrap_or("")
        ),
    );

    if !status_requires_running_wait(&before) {
        println!("{} 当前已不在运行态。", server.name);
        for line in render_post_stop_feedback_lines(Some(server), &before) {
            println!("{}", line);
        }
        return Ok(before);
    }

    trace_cli_action(
        &format!("{}_request_stop", action_scope),
        &format!("server_id={}", server.id),
    );
    manager.request_stop_server(&server.id)?;
    let after = manager.get_server_status(&server.id);

    println!("{} {}", server.name, headline);
    for line in render_post_stop_feedback_lines(Some(server), &after) {
        println!("{}", line);
    }

    trace_cli_action(
        "stop_report",
        &format!(
            "server_id={} status={} pid={} detail={} error={}",
            server.id,
            after.status.as_str(),
            after
                .pid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            after.detail_message.as_deref().unwrap_or(""),
            after.error_message.as_deref().unwrap_or("")
        ),
    );

    if show_follow_up {
        println!("可继续执行: sealantern server status {}", server.id);
        println!("如需查看停服日志: sealantern server logs {} --lines 50", server.id);
    }

    Ok(after)
}

fn stop_server_with_feedback_with_options(
    server: &ServerInstance,
    action_scope: &str,
    headline: &str,
    show_follow_up: bool,
) -> Result<ServerStatusInfo, String> {
    let manager = global::server_manager();
    let before = manager.get_server_status(&server.id);
    trace_cli_action(
        &format!("{}_stop_precheck", action_scope),
        &format!(
            "server_id={} status={} detail={} error={}",
            server.id,
            before.status.as_str(),
            before.detail_message.as_deref().unwrap_or(""),
            before.error_message.as_deref().unwrap_or("")
        ),
    );

    if !status_requires_running_wait(&before) {
        println!("{} 当前已不在运行态。", server.name);
        for line in render_post_stop_feedback_lines(Some(server), &before) {
            println!("{}", line);
        }
        return Ok(before);
    }

    println!("{} {}", server.name, headline);
    trace_cli_action(
        &format!("{}_stop_execute", action_scope),
        &format!("server_id={}", server.id),
    );
    manager.stop_server(&server.id)?;

    let after = manager.get_server_status(&server.id);
    for line in render_post_stop_feedback_lines(Some(server), &after) {
        println!("{}", line);
    }

    trace_cli_action(
        "stop_report",
        &format!(
            "server_id={} status={} pid={} detail={} error={}",
            server.id,
            after.status.as_str(),
            after
                .pid
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            after.detail_message.as_deref().unwrap_or(""),
            after.error_message.as_deref().unwrap_or("")
        ),
    );

    if show_follow_up {
        println!("可继续执行: sealantern server status {}", server.id);
        println!("如需查看停服日志: sealantern server logs {} --lines 50", server.id);
    }

    Ok(after)
}

pub(super) fn wait_for_server_stop(
    server_id: &str,
    timeout_secs: u64,
    action_scope: &str,
) -> Result<(), String> {
    let manager = global::server_manager();
    let started = std::time::Instant::now();
    loop {
        let status = manager.get_server_status(server_id);
        if !status_requires_running_wait(&status) {
            return Ok(());
        }

        if started.elapsed().as_secs() >= timeout_secs {
            let error = format!(
                "等待服务器停止超时: server_id={} timeout={}s last_status={} detail={} error={}",
                server_id,
                timeout_secs,
                status.status.as_str(),
                status.detail_message.as_deref().unwrap_or(""),
                status.error_message.as_deref().unwrap_or("")
            );
            trace_cli_error(
                &format!("{}_wait_timeout", action_scope),
                &format!("server_id={}", server_id),
                &error,
            );
            return Err(error);
        }

        thread::sleep(Duration::from_millis(RESTART_STOP_POLL_INTERVAL_MS));
    }
}

pub(super) fn status_requires_running_wait(status: &ServerStatusInfo) -> bool {
    if matches!(
        status.status,
        ServerStatus::Running | ServerStatus::Starting | ServerStatus::Stopping
    ) {
        return true;
    }

    matches!(status.status, ServerStatus::Error)
        && status_detail_indicates_running(status.detail_message.as_deref())
}

#[cfg(test)]
mod tests {
    use super::{
        print_start_observation_follow_up, render_start_observation_terminal_error,
        start_observation_is_terminal, start_observe_timeout_secs, status_requires_running_wait,
        status_snapshot_changed, DOCKER_START_OBSERVE_TIMEOUT_SECS,
        LOCAL_START_OBSERVE_TIMEOUT_SECS,
    };
    use crate::models::server::{
        DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig, LocalRuntimeConfig,
        ServerInstance, ServerRuntimeConfig, ServerStatus, ServerStatusInfo,
    };
    use std::collections::BTreeMap;

    fn local_server() -> ServerInstance {
        ServerInstance {
            id: "local-1".to_string(),
            name: "Local 1".to_string(),
            aliases: vec![],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: "E:/servers/local-1".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/local-1/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: vec![],
            }),
        }
    }

    fn docker_server() -> ServerInstance {
        ServerInstance {
            id: "docker-1".to_string(),
            name: "Docker 1".to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/docker/docker-1".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sea-test".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/docker-1".to_string(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: vec![],
                volume_mounts: vec![],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
            }),
        }
    }

    #[test]
    fn status_requires_running_wait_for_docker_error_with_running_detail() {
        let status = ServerStatusInfo {
            id: "docker-1".to_string(),
            status: ServerStatus::Error,
            pid: Some(456),
            uptime: Some(42),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=unhealthy exit_code=0 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: Some("Docker 容器健康检查失败".to_string()),
        };

        assert!(status_requires_running_wait(&status));
    }

    #[test]
    fn status_requires_running_wait_returns_false_for_stopped_error_detail() {
        let status = ServerStatusInfo {
            id: "docker-1".to_string(),
            status: ServerStatus::Error,
            pid: None,
            uptime: Some(42),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=exited running=false health=none exit_code=137 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: Some("Docker 容器已退出".to_string()),
        };

        assert!(!status_requires_running_wait(&status));
    }

    #[test]
    fn start_observation_is_terminal_waits_for_docker_health_but_accepts_healthy_running() {
        let running = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Running,
            pid: Some(1),
            uptime: Some(1),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=healthy"
                    .to_string(),
            ),
            error_message: None,
        };
        let docker_running_not_ready = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Running,
            pid: Some(1),
            uptime: Some(1),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=starting"
                    .to_string(),
            ),
            error_message: None,
        };
        let error = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Error,
            pid: None,
            uptime: Some(1),
            detail_message: Some("runtime=docker_itzg running=false".to_string()),
            error_message: Some("startup failed".to_string()),
        };
        let starting = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Starting,
            pid: Some(1),
            uptime: Some(1),
            detail_message: None,
            error_message: None,
        };

        assert!(start_observation_is_terminal(&running));
        assert!(!start_observation_is_terminal(&docker_running_not_ready));
        assert!(start_observation_is_terminal(&error));
        assert!(!start_observation_is_terminal(&starting));
    }

    #[test]
    fn status_snapshot_changed_detects_detail_or_error_differences() {
        let base = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Starting,
            pid: Some(1),
            uptime: Some(1),
            detail_message: Some("runtime=local is_running=true".to_string()),
            error_message: None,
        };
        let changed = ServerStatusInfo {
            detail_message: Some("runtime=local is_running=true exit_code=none".to_string()),
            ..base.clone()
        };

        assert!(status_snapshot_changed(&base, &changed));
        assert!(!status_snapshot_changed(&base, &base));
    }

    #[test]
    fn render_start_observation_terminal_error_returns_error_for_error_status() {
        let status = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Error,
            pid: None,
            uptime: Some(1),
            detail_message: Some("runtime=local is_running=false exit_code=7".to_string()),
            error_message: Some("服务器异常退出 (退出码：7)".to_string()),
        };

        let message = render_start_observation_terminal_error("server-1", &status)
            .expect("error status should become observation failure");

        assert!(message.contains("status=error"));
        assert!(message.contains("退出码：7"));
    }

    #[test]
    fn render_start_observation_terminal_error_returns_error_for_stopped_status() {
        let status = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Stopped,
            pid: None,
            uptime: Some(1),
            detail_message: Some("runtime=local is_running=false exit_code=0".to_string()),
            error_message: None,
        };

        let message = render_start_observation_terminal_error("server-1", &status)
            .expect("stopped status should become observation failure");

        assert!(message.contains("status=stopped"));
        assert!(message.contains("exit_code=0"));
    }

    #[test]
    fn print_start_observation_follow_up_keeps_non_terminal_running_snapshot_non_failing() {
        let status = ServerStatusInfo {
            id: "server-1".to_string(),
            status: ServerStatus::Running,
            pid: Some(77),
            uptime: Some(3),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=starting exit_code=0 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: None,
        };

        print_start_observation_follow_up("server-1", &status, 6);
        assert!(!start_observation_is_terminal(&status));
        assert!(render_start_observation_terminal_error("server-1", &status).is_none());
    }

    #[test]
    fn start_observe_timeout_secs_uses_longer_window_for_docker() {
        assert_eq!(start_observe_timeout_secs(&local_server()), LOCAL_START_OBSERVE_TIMEOUT_SECS);
        assert_eq!(start_observe_timeout_secs(&docker_server()), DOCKER_START_OBSERVE_TIMEOUT_SECS);
    }
}
