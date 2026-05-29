use crate::models::server::{ServerInstance, ServerRuntimeConfig};
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_feedback::render_runtime_start_failure_hint_lines;
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::{
    describe_server_instance, print_created_server, trace_cli_action, trace_cli_error,
    CliServerRuntimeKind,
};

pub(super) fn handle_created_server_flow<FEnsureStarted, FOrchestrate>(
    command: &CliServerCommand,
    server: &ServerInstance,
    ports: &PreparedPorts,
    runtime_kind: CliServerRuntimeKind,
    ensure_started: FEnsureStarted,
    orchestrate: FOrchestrate,
) -> Result<(), String>
where
    FEnsureStarted: Fn(&ServerInstance) -> Result<(), String>,
    FOrchestrate: Fn(&CliServerCommand, &ServerInstance, &PreparedPorts) -> Result<(), String>,
{
    if command.create_only {
        trace_cli_action(
            "create_only_completed",
            &format!("server_id={} runtime={}", server.id, runtime_kind.as_runtime_label()),
        );
        println!(
            "create-only 模式已完成，仅创建/接管服务器记录，未执行首次启动: id={} name={}",
            server.id, server.name
        );
        for line in render_create_only_follow_up_lines(server) {
            println!("{}", line);
        }
    } else if command.detach {
        trace_cli_action(
            "transport_detach",
            &format!("server_id={} runtime={}", server.id, runtime_kind.as_runtime_label()),
        );
        if let Err(err) = ensure_started(server) {
            print_post_create_start_failure(server, &err);
            return Err(err);
        }
        for line in render_detach_follow_up_lines(server) {
            println!("{}", line);
        }
    } else if let Err(err) = orchestrate(command, server, ports) {
        print_post_create_start_failure(server, &err);
        return Err(err);
    }

    trace_cli_action("completed", &describe_server_instance(server));
    Ok(())
}

pub(super) fn print_post_create_start_failure(server: &ServerInstance, error: &str) {
    trace_cli_error(
        "post_create_start_failed",
        &format!("server_id={} runtime={}", server.id, server.runtime_kind),
        error,
    );

    eprintln!("服务器记录已创建，但首次启动失败: id={} name={}", server.id, server.name);
    eprintln!("失败原因: {}", error);

    for line in render_runtime_start_failure_hint_lines(server, error) {
        eprintln!("{}", line);
    }
}

pub(super) fn print_created_server_record(
    server: &ServerInstance,
    game_port: u16,
    web_port: Option<u16>,
    runtime_kind: CliServerRuntimeKind,
    aliases: &[String],
) -> Result<(), String> {
    print_created_server(server, game_port, web_port, runtime_kind, aliases);
    Ok(())
}

fn render_create_only_follow_up_lines(server: &ServerInstance) -> Vec<String> {
    let mut lines = vec![format!("可继续执行: sealantern server inspect {}", server.id)];
    lines.push(format!("可继续执行: sealantern server start {}", server.id));

    match &server.runtime {
        ServerRuntimeConfig::Local(_) => {
            lines.push("本地提示: 当前仅落库，不会预先验证首次运行日志；如需确认启动链路，请后续执行 start 或 CLI/Web 附加启动。".to_string());
        }
        ServerRuntimeConfig::DockerItzg(runtime) => {
            lines.push(format!("可继续执行: sealantern compose generate {}", server.id));
            lines.push(
                "Docker 提示: 当前未触发首次 docker run；这适合离线预配置、先导出 compose，或等待镜像准备完成后再启动。"
                    .to_string(),
            );
            lines.push(format!(
                "Docker 提示: 若宿主机尚未缓存镜像 {}:{}，后续 start 仍需要先执行 sealantern docker pull {}:{}，或改用本地/私有镜像。",
                runtime.image, runtime.image_tag, runtime.image, runtime.image_tag
            ));
        }
    }

    lines
}

fn render_detach_follow_up_lines(server: &ServerInstance) -> Vec<String> {
    let mut lines =
        vec![format!("detach 模式已返回，可继续执行: sealantern server status {}", server.id)];
    lines.push(format!("如需查看日志: sealantern server logs {} --lines 50", server.id));

    if matches!(&server.runtime, ServerRuntimeConfig::DockerItzg(_)) {
        lines.push(
            "Docker 提示: 如目标是 itzg/minecraft-server，请等待容器 health=healthy 后再发送命令或判断完全就绪。"
                .to_string(),
        );
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{render_create_only_follow_up_lines, render_detach_follow_up_lines};
    use crate::models::server::{
        DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig, LocalRuntimeConfig,
        ServerInstance, ServerRuntimeConfig,
    };
    use std::collections::BTreeMap;

    fn sample_local_server() -> ServerInstance {
        ServerInstance {
            id: "local-1".to_string(),
            name: "fabric-main".to_string(),
            aliases: vec![],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: "E:/servers/fabric-main".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/fabric-main/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: vec![],
            }),
        }
    }

    fn sample_docker_server() -> ServerInstance {
        ServerInstance {
            id: "docker-1".to_string(),
            name: "paper-docker".to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/docker/paper-docker".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "latest".to_string(),
                container_name: "sealantern-paper-docker".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/paper-docker".to_string(),
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
    fn create_only_follow_up_lines_include_offline_docker_guidance() {
        let lines = render_create_only_follow_up_lines(&sample_docker_server()).join("\n");

        assert!(lines.contains("sealantern server start docker-1"));
        assert!(lines.contains("sealantern compose generate docker-1"));
        assert!(lines.contains("离线预配置"));
        assert!(lines.contains("sealantern docker pull itzg/minecraft-server:latest"));
    }

    #[test]
    fn create_only_follow_up_lines_include_local_start_hint() {
        let lines = render_create_only_follow_up_lines(&sample_local_server()).join("\n");

        assert!(lines.contains("sealantern server start local-1"));
        assert!(lines.contains("当前仅落库"));
    }

    #[test]
    fn detach_follow_up_lines_include_docker_health_guidance() {
        let lines = render_detach_follow_up_lines(&sample_docker_server()).join("\n");

        assert!(lines.contains("sealantern server status docker-1"));
        assert!(lines.contains("sealantern server logs docker-1 --lines 50"));
        assert!(lines.contains("health=healthy"));
    }
}
