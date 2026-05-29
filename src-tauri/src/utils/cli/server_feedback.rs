use crate::models::server::{
    DockerCommandMode, ServerInstance, ServerRuntimeConfig, ServerStatus, ServerStatusInfo,
};
use crate::utils::server_status::{
    status_detail_health, status_detail_indicates_running, status_detail_runtime_kind,
    status_is_docker_command_ready,
};

use super::server_endpoint::render_docker_rcon_operator_hint;
pub(super) use super::server_feedback_preflight::{
    render_runtime_preflight_failure_hint_lines,
    render_runtime_preflight_failure_hint_lines_from_error,
};

pub(super) fn render_post_start_feedback_lines(
    server: Option<&ServerInstance>,
    status: &ServerStatusInfo,
) -> Vec<String> {
    let mut lines = vec![render_status_snapshot_line(status)];

    if let Some(detail) = render_status_detail_line(server, status) {
        lines.push(detail);
    }

    if let Some(error) = &status.error_message {
        lines.push(format!("启动提示: {}", error));
    }

    lines.extend(render_runtime_start_state_hint_lines(status));

    if let Some(server) = server {
        lines.extend(render_runtime_start_hint_lines(server));
    }

    lines
}

fn render_runtime_start_state_hint_lines(status: &ServerStatusInfo) -> Vec<String> {
    let detail = status.detail_message.as_deref();
    if !status_detail_runtime_kind(detail)
        .is_some_and(|runtime| runtime.eq_ignore_ascii_case("docker_itzg"))
    {
        return Vec::new();
    }

    match status_detail_health(detail) {
        Some(health) if health.eq_ignore_ascii_case("starting") => {
            vec![
                "Docker 提示: 容器已启动，但健康检查仍为 starting；如目标是 itzg/minecraft-server，请等待 health=healthy 后再发送命令或判断启动完成。"
                    .to_string(),
            ]
        }
        Some(health)
            if health.eq_ignore_ascii_case("healthy") && status_is_docker_command_ready(status) =>
        {
            vec!["Docker 提示: 容器健康检查已通过，命令通道可继续尝试。".to_string()]
        }
        _ => Vec::new(),
    }
}

pub(super) fn render_runtime_start_failure_hint_lines(
    server: &ServerInstance,
    error: &str,
) -> Vec<String> {
    let mut lines = vec![format!("可继续执行: sealantern server inspect {}", server.id)];
    lines.push(format!("可继续执行: sealantern server status {}", server.id));
    lines.push(format!("可继续执行: sealantern server logs {} --lines 50", server.id));

    if let ServerRuntimeConfig::DockerItzg(runtime) = &server.runtime {
        lines.push("Docker 建议: 先执行 sealantern docker doctor".to_string());
        lines.push(format!(
            "Docker 建议: 如镜像未缓存，可先执行 sealantern docker pull {}:{}",
            runtime.image, runtime.image_tag
        ));
        lines.push(format!(
            "Docker 建议: 网络恢复或镜像就绪后，可重试 sealantern server start {}",
            server.id
        ));

        let error_lower = error.to_ascii_lowercase();
        if error_lower.contains("rcon") {
            lines.push(
                "Docker 建议: 如命令通道为 RCON，请确认容器已完成启动、RCON 端口可达且密码配置正确"
                    .to_string(),
            );
        }
    } else if error_indicates_not_running(error) || error.contains("异常退出") {
        lines.push(format!(
            "本地建议: 如 Java 进程已异常退出，可先执行 sealantern server status {} 与 sealantern server logs {} --lines 100 排查后重试",
            server.id, server.id
        ));
    }

    lines
}

pub(super) fn render_runtime_stop_failure_hint_lines(
    server: &ServerInstance,
    error: &str,
) -> Vec<String> {
    let mut lines = vec![format!("可继续执行: sealantern server status {}", server.id)];
    lines.push(format!("可继续执行: sealantern server logs {} --lines 50", server.id));
    lines.push(format!("可继续执行: sealantern server inspect {}", server.id));

    if let ServerRuntimeConfig::DockerItzg(runtime) = &server.runtime {
        lines.push(format!(
            "Docker 建议: 如容器卡在 stopping / unhealthy，可执行 sealantern server force-stop {}",
            server.id
        ));
        lines.push(
            "Docker 建议: 如需排查宿主环境与网络映射，可先执行 sealantern docker doctor"
                .to_string(),
        );

        let error_lower = error.to_ascii_lowercase();
        if error_lower.contains("rcon") {
            lines.push(format!(
                "Docker 建议: 当前记录命令模式为 {}，如 RCON 不可用可考虑改用 docker_stdio 或先确认 {} 容器已就绪",
                runtime.command_mode.as_str(),
                runtime.container_name
            ));
        }
    }

    lines
}

pub(super) fn render_send_command_failure_hint_lines(
    server: &ServerInstance,
    command: &str,
    error: &str,
) -> Vec<String> {
    let mut lines = vec![format!("可继续执行: sealantern server status {}", server.id)];
    lines.push(format!("可继续执行: sealantern server logs {} --lines 50", server.id));
    lines.push(format!("可继续执行: sealantern server inspect {}", server.id));

    let trimmed_command = command.trim();
    if !trimmed_command.is_empty() {
        lines.push(format!("重试命令: sealantern server send {} {}", server.id, trimmed_command));
    }

    match &server.runtime {
        ServerRuntimeConfig::Local(runtime) => {
            lines.push(format!(
                "本地建议: 当前入口为 {} ({})，请先确认 Java 进程仍在运行且 stdin 未断开",
                runtime.jar_path, runtime.startup_mode
            ));
            if error_indicates_not_running(error) {
                lines.push(format!(
                    "本地建议: 如服务端已退出，可重新执行 sealantern server start {}",
                    server.id
                ));
            }
        }
        ServerRuntimeConfig::DockerItzg(runtime) => {
            lines.push(format!(
                "Docker 目标: container={} command_mode={}",
                runtime.container_name,
                runtime.command_mode.as_str()
            ));

            let error_lower = error.to_ascii_lowercase();
            if docker_error_indicates_missing_container(&error_lower) {
                lines.push(format!(
                    "Docker 建议: 容器尚未创建或已被移除，可先执行 sealantern server start {}",
                    server.id
                ));
                lines.push(
                    "Docker 建议: 如启动前需要排查宿主环境，可先执行 sealantern docker doctor"
                        .to_string(),
                );
            }

            if error_indicates_not_running(error) {
                lines.push(
                    "Docker 建议: 目标容器当前不在 running/healthy 状态，请先等待启动完成或先查看 status/logs"
                        .to_string(),
                );
            }

            if error_lower.contains("rcon") {
                if let Some(rcon) = &runtime.rcon {
                    lines.push(format!(
                        "Docker 建议: 请检查 RCON endpoint={}:{} password=<redacted>，并确认容器已完成启动",
                        rcon.host, rcon.port
                    ));
                    lines.push(render_docker_rcon_operator_hint(rcon, &server.id));
                } else {
                    lines.push(
                        "Docker 建议: 当前命令模式为 RCON，但记录中缺少 RCON endpoint，请检查 runtime 配置"
                            .to_string(),
                    );
                }

                if error_lower.contains("early eof")
                    || error_lower.contains("connection refused")
                    || error_lower.contains("timed out")
                    || error_lower.contains("timeout")
                    || error_lower.contains("password")
                    || error_lower.contains("认证")
                {
                    lines.push(
                        "Docker 建议: 这更像 RCON 尚未就绪、连接被拒绝或认证失败；如目标是 itzg/minecraft-server，请优先等待容器 health=healthy 后再重试"
                            .to_string(),
                    );
                }
            }

            if runtime.command_mode == DockerCommandMode::DockerStdio
                && (error_lower.contains("docker_stdio")
                    || error_lower.contains("mc-send-to-console"))
            {
                lines.push(
                    "Docker 建议: 当前镜像或容器不支持 docker stdio 命令通道，可切换到 --command-mode rcon"
                        .to_string(),
                );
            }
        }
    }

    dedupe_lines(lines)
}

pub(super) fn render_post_stop_feedback_lines(
    server: Option<&ServerInstance>,
    status: &ServerStatusInfo,
) -> Vec<String> {
    let mut lines = vec![render_status_snapshot_line(status)];

    if let Some(detail) = render_status_detail_line(server, status) {
        lines.push(detail);
    }

    if let Some(error) = &status.error_message {
        lines.push(format!("停服提示: {}", error));
    }

    if let Some(server) = server {
        lines.extend(render_runtime_stop_hint_lines(server, status));
    }

    lines
}

pub(super) fn render_status_snapshot_line(status: &ServerStatusInfo) -> String {
    match status.pid {
        Some(pid) => format!("当前状态: {} (pid: {})", status.status.as_str(), pid),
        None => format!("当前状态: {}", status.status.as_str()),
    }
}

pub(super) fn render_status_detail_line(
    server: Option<&ServerInstance>,
    status: &ServerStatusInfo,
) -> Option<String> {
    let detail = status.detail_message.as_deref()?.trim();
    if detail.is_empty() {
        return None;
    }

    let summary = match server.map(|value| &value.runtime) {
        Some(ServerRuntimeConfig::Local(_)) => {
            select_detail_fields(detail, &["is_running", "exit_code"])
        }
        Some(ServerRuntimeConfig::DockerItzg(_)) => select_detail_fields(
            detail,
            &[
                "container",
                "state",
                "running",
                "health",
                "exit_code",
                "backend",
                "command_mode",
            ],
        ),
        None => None,
    }
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| detail.to_string());

    Some(format!("状态详情: {}", summary))
}

pub(super) fn render_runtime_start_hint_lines(server: &ServerInstance) -> Vec<String> {
    match &server.runtime {
        ServerRuntimeConfig::Local(runtime) => {
            vec![format!("启动入口: {} ({})", runtime.jar_path, runtime.startup_mode)]
        }
        ServerRuntimeConfig::DockerItzg(runtime) => {
            let mut lines = vec![format!(
                "容器运行时: {} (backend: {})",
                runtime.container_name,
                runtime.docker_backend_kind.as_str()
            )];

            match runtime.command_mode {
                DockerCommandMode::Rcon => {
                    if let Some(rcon) = &runtime.rcon {
                        lines.push(render_docker_rcon_operator_hint(rcon, &server.id));
                    } else {
                        lines.push(
                            "命令通道: RCON，但当前记录缺少 endpoint 配置，请检查 docker runtime 配置"
                                .to_string(),
                        );
                    }
                }
                DockerCommandMode::DockerStdio => {
                    lines.push(
                        "命令通道: docker stdio，镜像内需要提供 mc-send-to-console，并按 itzg 语义启用 CREATE_CONSOLE_IN_PIPE".to_string(),
                    );
                }
            }

            lines
        }
    }
}

pub(super) fn render_runtime_stop_hint_lines(
    server: &ServerInstance,
    status: &ServerStatusInfo,
) -> Vec<String> {
    let mut lines = Vec::new();

    if matches!(
        status.status,
        ServerStatus::Stopping | ServerStatus::Running | ServerStatus::Starting
    ) || status_detail_indicates_running(status.detail_message.as_deref())
    {
        lines.push("当前已进入停服流程，可继续观察状态或日志输出。".to_string());
    } else if matches!(status.status, ServerStatus::Stopped) {
        lines.push("当前状态显示服务器已经停止。".to_string());
    }

    match &server.runtime {
        ServerRuntimeConfig::Local(runtime) => {
            lines.push(format!("本地入口: {} ({})", runtime.jar_path, runtime.startup_mode))
        }
        ServerRuntimeConfig::DockerItzg(runtime) => {
            lines.push(format!(
                "目标容器: {} (command_mode: {})",
                runtime.container_name,
                runtime.command_mode.as_str()
            ));

            let detail = status.detail_message.as_deref().unwrap_or_default();
            if detail.contains("state=exited") {
                lines.push(
                    "Docker 提示: 容器已退出但仍保留，可直接再次执行 start 复用同一条服务器记录。"
                        .to_string(),
                );
            } else if detail.contains("state=missing") {
                lines.push(
                    "Docker 提示: 容器已被移除；再次 start 时会按当前记录重新创建容器。"
                        .to_string(),
                );
            }
        }
    }

    lines
}

fn error_indicates_not_running(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("not running")
        || lower.contains("不可接收命令")
        || lower.contains("已停止")
        || lower.contains("stdin")
        || lower.contains("写入失败")
        || lower.contains("刷新失败")
        || lower.contains("broken pipe")
        || lower.contains("state=exited")
        || lower.contains("running=false")
}

fn docker_error_indicates_missing_container(error_lower: &str) -> bool {
    error_lower.contains("no such container")
        || error_lower.contains("container missing")
        || error_lower.contains("目标容器不存在")
        || error_lower.contains("docker 容器不存在")
        || error_lower.contains("尚未创建")
}

fn dedupe_lines(lines: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::with_capacity(lines.len());
    for line in lines {
        if !deduped.iter().any(|existing| existing == &line) {
            deduped.push(line);
        }
    }
    deduped
}

fn select_detail_fields(detail: &str, keys: &[&str]) -> Option<String> {
    let mut values = Vec::new();

    for key in keys {
        if let Some(value) = extract_detail_field(detail, key) {
            values.push(format!("{}={}", key, value));
        }
    }

    if values.is_empty() {
        None
    } else {
        Some(values.join(" "))
    }
}

fn extract_detail_field<'a>(detail: &'a str, key: &str) -> Option<&'a str> {
    detail.split_whitespace().find_map(|part| {
        let (field, value) = part.split_once('=')?;
        if field == key {
            Some(value)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        render_post_start_feedback_lines, render_post_stop_feedback_lines,
        render_runtime_start_failure_hint_lines, render_runtime_start_hint_lines,
        render_runtime_stop_failure_hint_lines, render_send_command_failure_hint_lines,
        render_status_detail_line, status_detail_indicates_running,
    };
    use crate::models::server::{
        DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig, LocalRuntimeConfig,
        RconConfig, ServerInstance, ServerRuntimeConfig, ServerStatus, ServerStatusInfo,
    };
    use std::collections::BTreeMap;

    fn sample_local_server() -> ServerInstance {
        ServerInstance {
            id: "local-1".to_string(),
            name: "Local One".to_string(),
            aliases: vec![],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: "E:/servers/local-1".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
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

    fn sample_docker_server(command_mode: DockerCommandMode) -> ServerInstance {
        ServerInstance {
            id: "docker-1".to_string(),
            name: "Docker One".to_string(),
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
                command_mode,
                rcon: Some(RconConfig {
                    host: "127.0.0.1".to_string(),
                    port: 25575,
                    password: "secret".to_string(),
                }),
            }),
        }
    }

    #[test]
    fn status_detail_indicates_running_detects_running_true_token() {
        assert!(status_detail_indicates_running(Some(
            "runtime=docker_itzg container=sea-test state=running running=true health=healthy"
        )));
        assert!(!status_detail_indicates_running(Some(
            "runtime=docker_itzg container=sea-test state=exited running=false health=none"
        )));
    }

    #[test]
    fn render_status_detail_line_prefers_local_focus_fields() {
        let server = sample_local_server();
        let status = ServerStatusInfo {
            id: server.id.clone(),
            status: ServerStatus::Starting,
            pid: Some(1357),
            uptime: Some(3),
            detail_message: Some("runtime=local is_running=true exit_code=none".to_string()),
            error_message: None,
        };

        let detail = render_status_detail_line(Some(&server), &status).expect("detail line");
        assert_eq!(detail, "状态详情: is_running=true exit_code=none");
    }

    #[test]
    fn render_post_start_feedback_lines_include_docker_rcon_hint() {
        let server = sample_docker_server(DockerCommandMode::Rcon);
        let status = ServerStatusInfo {
            id: server.id.clone(),
            status: ServerStatus::Starting,
            pid: Some(2468),
            uptime: Some(1),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=starting exit_code=0 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: None,
        };

        let lines = render_post_start_feedback_lines(Some(&server), &status).join("\n");
        assert!(lines.contains("当前状态: starting (pid: 2468)"));
        assert!(lines.contains(
            "状态详情: container=sea-test state=running running=true health=starting exit_code=0 backend=cli command_mode=rcon"
        ));
        assert!(lines.contains("容器运行时: sea-test (backend: cli)"));
        assert!(lines.contains(
            "命令通道: RCON 127.0.0.1:25575，可执行 sealantern server send docker-1 <command>"
        ));
    }

    #[test]
    fn render_runtime_start_hint_lines_include_docker_stdio_notice() {
        let server = sample_docker_server(DockerCommandMode::DockerStdio);
        let lines = render_runtime_start_hint_lines(&server).join("\n");

        assert!(lines.contains("容器运行时: sea-test (backend: cli)"));
        assert!(lines.contains("命令通道: docker stdio，镜像内需要提供 mc-send-to-console"));
    }

    #[test]
    fn render_runtime_start_failure_hint_lines_include_docker_recovery_actions() {
        let server = sample_docker_server(DockerCommandMode::Rcon);
        let lines = render_runtime_start_failure_hint_lines(
            &server,
            "docker run 失败: image=itzg/minecraft-server:java21 network timeout",
        )
        .join("\n");

        assert!(lines.contains("sealantern server inspect docker-1"));
        assert!(lines.contains("sealantern server status docker-1"));
        assert!(lines.contains("sealantern server logs docker-1 --lines 50"));
        assert!(lines.contains("sealantern docker doctor"));
        assert!(lines.contains("sealantern docker pull itzg/minecraft-server:java21"));
        assert!(lines.contains("sealantern server start docker-1"));
    }

    #[test]
    fn render_post_stop_feedback_lines_include_stopping_hint_for_docker_runtime() {
        let server = sample_docker_server(DockerCommandMode::Rcon);
        let status = ServerStatusInfo {
            id: server.id.clone(),
            status: ServerStatus::Stopping,
            pid: Some(2468),
            uptime: Some(10),
            detail_message: Some(
                "runtime=docker_itzg container=sea-test state=running running=true health=healthy exit_code=0 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: None,
        };

        let lines = render_post_stop_feedback_lines(Some(&server), &status).join("\n");
        assert!(lines.contains("当前状态: stopping (pid: 2468)"));
        assert!(lines.contains("当前已进入停服流程，可继续观察状态或日志输出。"));
        assert!(lines.contains("目标容器: sea-test (command_mode: rcon)"));
    }

    #[test]
    fn render_runtime_stop_failure_hint_lines_include_docker_recovery_actions() {
        let server = sample_docker_server(DockerCommandMode::Rcon);
        let lines = render_runtime_stop_failure_hint_lines(
            &server,
            "等待服务器停止超时: server_id=docker-1 timeout=30s last_status=error detail=runtime=docker_itzg container=sea-test state=running running=true health=unhealthy exit_code=0 backend=cli command_mode=rcon error=Docker 容器健康检查失败",
        )
        .join("\n");

        assert!(lines.contains("sealantern server status docker-1"));
        assert!(lines.contains("sealantern server logs docker-1 --lines 50"));
        assert!(lines.contains("sealantern server inspect docker-1"));
        assert!(lines.contains("sealantern server force-stop docker-1"));
        assert!(lines.contains("sealantern docker doctor"));
    }

    #[test]
    fn render_send_command_failure_hint_lines_include_docker_recovery_actions() {
        let server = sample_docker_server(DockerCommandMode::Rcon);
        let lines = render_send_command_failure_hint_lines(
            &server,
            "say hello",
            "Docker 容器不存在: sea-test。Error response from daemon: No such container: sea-test",
        )
        .join("\n");

        assert!(lines.contains("sealantern server status docker-1"));
        assert!(lines.contains("sealantern server logs docker-1 --lines 50"));
        assert!(lines.contains("sealantern server inspect docker-1"));
        assert!(lines.contains("sealantern server send docker-1 say hello"));
        assert!(lines.contains("container=sea-test command_mode=rcon"));
        assert!(lines.contains("sealantern server start docker-1"));
        assert!(lines.contains("sealantern docker doctor"));
    }

    #[test]
    fn render_send_command_failure_hint_lines_include_local_recovery_actions() {
        let server = sample_local_server();
        let lines = render_send_command_failure_hint_lines(
            &server,
            "say hello",
            "本地服务端 stdin 写入失败: broken pipe",
        )
        .join("\n");

        assert!(lines.contains("sealantern server status local-1"));
        assert!(lines.contains("sealantern server logs local-1 --lines 50"));
        assert!(lines.contains("Java 进程仍在运行且 stdin 未断开"));
        assert!(lines.contains("sealantern server start local-1"));
    }

    #[test]
    fn render_send_command_failure_hint_lines_do_not_report_docker_stdio_incompatibility_for_rcon_errors(
    ) {
        let server = sample_docker_server(DockerCommandMode::Rcon);
        let lines = render_send_command_failure_hint_lines(
            &server,
            "say hello",
            "通过 RCON 连接 Docker 容器失败: container=sea-test endpoint=127.0.0.1:25575 command_mode=rcon error=early eof。请确认容器已完成启动、RCON 端口可达、密码正确，或切换到 --command-mode docker_stdio。",
        )
        .join("\n");

        assert!(lines.contains("health=healthy 后再重试"));
        assert!(!lines.contains("不支持 docker stdio 命令通道"));
    }
}
