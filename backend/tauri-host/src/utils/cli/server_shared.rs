use crate::models::server::{
    DockerItzgRuntimeConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig, ServerStatus,
};
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::logger;

use super::server_endpoint::{
    render_cli_web_binding_hint, render_cli_web_browser_url, render_docker_rcon_operator_hint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CliServerRuntimeKind {
    Local,
    Docker,
}

impl CliServerRuntimeKind {
    pub(super) fn as_runtime_label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Docker => "docker_itzg",
        }
    }
}

pub(super) fn format_aliases(aliases: &[String]) -> String {
    if aliases.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", aliases.join(","))
    }
}

pub(super) fn describe_server_instance(server: &ServerInstance) -> String {
    format!(
        "server_id={} name={} runtime={} port={} aliases={}",
        server.id,
        server.name,
        server.runtime_kind,
        server.port,
        format_aliases(&server.aliases)
    )
}

pub(super) fn describe_status(status: &ServerStatus) -> &'static str {
    status.as_str()
}

pub(super) fn resolve_cli_target_hint(command: &CliServerCommand) -> &str {
    command
        .name
        .as_deref()
        .or(command.positional_name.as_deref())
        .or(command.server_tag.as_deref())
        .unwrap_or("<name>")
}

pub(super) fn trace_cli_action(action: &str, detail: &str) {
    logger::log_user_action("cli.server", action, detail);
}

pub(super) fn trace_cli_error(action: &str, detail: &str, error: &str) {
    logger::log_user_action_error("cli.server", action, detail, error);
}

pub(super) fn trace_compose_action(action: &str, detail: &str) {
    logger::log_user_action("cli.compose", action, detail);
}

pub(super) fn trace_compose_error(action: &str, detail: &str, error: &str) {
    logger::log_user_action_error("cli.compose", action, detail, error);
}

pub(super) fn trace_docker_action(action: &str, detail: &str) {
    logger::log_user_action("cli.docker", action, detail);
}

pub(super) fn trace_docker_error(action: &str, detail: &str, error: &str) {
    logger::log_user_action_error("cli.docker", action, detail, error);
}

pub(super) fn print_created_server(
    server: &ServerInstance,
    game_port: u16,
    web_port: Option<u16>,
    runtime_kind: CliServerRuntimeKind,
    aliases: &[String],
) {
    for line in render_created_server_lines(server, game_port, web_port, runtime_kind, aliases) {
        println!("{}", line);
    }
}

fn render_created_server_lines(
    server: &ServerInstance,
    game_port: u16,
    web_port: Option<u16>,
    runtime_kind: CliServerRuntimeKind,
    aliases: &[String],
) -> Vec<String> {
    let mut lines = vec![
        "已创建/接管服务器记录".to_string(),
        format!("  id: {}", server.id),
        format!("  name: {}", server.name),
        format!("  runtime: {}", runtime_kind.as_runtime_label()),
        format!("  server_path: {}", server.path),
        format!("  game_port: {}", game_port),
    ];
    if let Some(port) = web_port {
        lines.push(format!("  web_port: {}", port));
    }
    if !aliases.is_empty() {
        lines.push(format!("  aliases: {}", aliases.join(", ")));
    }
    lines.extend(runtime_created_lines(&server.runtime));
    lines.extend(render_follow_up_lines(server, web_port));
    lines
}

fn runtime_created_lines(runtime: &ServerRuntimeConfig) -> Vec<String> {
    match runtime {
        ServerRuntimeConfig::Local(local) => render_local_created_lines(local),
        ServerRuntimeConfig::DockerItzg(docker) => render_docker_created_lines(docker),
    }
}

fn render_local_created_lines(runtime: &LocalRuntimeConfig) -> Vec<String> {
    let mut lines = vec![
        format!("  local.startup_mode: {}", runtime.startup_mode),
        format!("  local.java_path: {}", runtime.java_path),
        format!("  local.entry_path: {}", runtime.jar_path),
    ];
    if let Some(custom_command) = &runtime.custom_command {
        lines.push(format!("  local.custom_command: {}", custom_command));
    }
    lines
}

fn render_docker_created_lines(runtime: &DockerItzgRuntimeConfig) -> Vec<String> {
    let mut lines = vec![
        format!("  docker.image: {}:{}", runtime.image, runtime.image_tag),
        format!("  docker.container_name: {}", runtime.container_name),
        format!("  docker.data_dir_mount: {}", runtime.data_dir_mount),
        format!("  docker.backend: {}", runtime.docker_backend_kind.as_str()),
        format!("  docker.command_mode: {}", runtime.command_mode.as_str()),
    ];
    if let Some(rcon) = &runtime.rcon {
        lines.push(format!("  docker.rcon_endpoint: {}:{}", rcon.host, rcon.port));
    }
    lines
}

fn render_follow_up_lines(server: &ServerInstance, web_port: Option<u16>) -> Vec<String> {
    let mut lines = vec!["后续操作:".to_string()];
    lines.push(format!("  inspect: sealantern server inspect {}", server.id));
    lines.push(format!("  status:  sealantern server status {}", server.id));
    lines.push(format!("  logs:    sealantern server logs {} --lines 50", server.id));
    lines.push(format!("  start:   sealantern server start {}", server.id));

    match &server.runtime {
        ServerRuntimeConfig::Local(_) => {
            lines.push(format!(
                "  hint:    本地服务器启动后，可执行 sealantern server send {} say hello",
                server.id
            ));
        }
        ServerRuntimeConfig::DockerItzg(runtime) => {
            lines.push("  doctor:  sealantern docker doctor".to_string());
            lines.push(format!("  compose: sealantern compose generate {}", server.id));
            if runtime.command_mode.as_str() == "rcon" {
                if let Some(rcon) = &runtime.rcon {
                    lines.push(
                        "  hint:    Docker 容器启动后，请先观察 status/logs；如目标是 itzg，请等待 health=healthy 后再发送命令"
                            .to_string(),
                    );
                    lines.push(format!(
                        "  hint:    {}",
                        render_docker_rcon_operator_hint(rcon, &server.id)
                    ));
                } else {
                    lines.push(
                        "  hint:    Docker 命令通道当前使用 RCON，但当前记录缺少 endpoint 配置"
                            .to_string(),
                    );
                }
            }
        }
    }

    if let Some(port) = web_port {
        lines.push(format!("  web:     {}", render_cli_web_browser_url(port, &server.id)));
        if let Some(hint) = render_cli_web_binding_hint(port) {
            lines.push(format!("  web.bind: {}", hint));
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{render_created_server_lines, CliServerRuntimeKind};
    use crate::models::server::{
        CpuPolicyConfig, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
        JvmPresetConfig, LocalRuntimeConfig, LocalTerminalMode, RconConfig, ServerInstance,
        ServerRuntimeConfig,
    };
    use crate::utils::cli::server_test_support::{lock_env, EnvGuard};
    use std::collections::BTreeMap;

    fn sample_local_server() -> ServerInstance {
        ServerInstance {
            id: "server-1".to_string(),
            name: "fabric-main".to_string(),
            aliases: vec!["fabric".to_string()],
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
                custom_command: Some("java -jar server.jar nogui".to_string()),
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                terminal_mode: LocalTerminalMode::PipeManaged,
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn sample_docker_server() -> ServerInstance {
        ServerInstance {
            id: "docker-1".to_string(),
            name: "paper-docker".to_string(),
            aliases: vec!["paper".to_string()],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/docker/paper".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sealantern-paper".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/paper".to_string(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: vec![],
                volume_mounts: vec![],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: Some(RconConfig {
                    host: "127.0.0.1".to_string(),
                    port: 25575,
                    password: "secret-pass".to_string(),
                }),
                jvm_args: vec![],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn render_created_server_lines_include_local_runtime_summary() {
        let _env_lock = lock_env();
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "127.0.0.1");

        let lines = render_created_server_lines(
            &sample_local_server(),
            25565,
            Some(8000),
            CliServerRuntimeKind::Local,
            &["fabric".to_string()],
        );
        let joined = lines.join("\n");

        assert!(joined.contains("runtime: local"));
        assert!(joined.contains("web_port: 8000"));
        assert!(joined.contains("local.java_path: C:/Java/bin/java.exe"));
        assert!(joined.contains("local.custom_command: java -jar server.jar nogui"));
        assert!(joined.contains("后续操作:"));
        assert!(joined.contains("inspect: sealantern server inspect server-1"));
        assert!(joined.contains("start:   sealantern server start server-1"));
        assert!(
            joined.contains("本地服务器启动后，可执行 sealantern server send server-1 say hello")
        );
        assert!(joined.contains("web:     http://127.0.0.1:8000/console/server-1"));
    }

    #[test]
    fn render_created_server_lines_include_headless_bind_hint_when_web_binds_all_interfaces() {
        let _env_lock = lock_env();
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");

        let lines = render_created_server_lines(
            &sample_local_server(),
            25565,
            Some(8000),
            CliServerRuntimeKind::Local,
            &["fabric".to_string()],
        );
        let joined = lines.join("\n");

        assert!(joined.contains("web:     http://127.0.0.1:8000/console/server-1"));
        assert!(joined.contains("web.bind: CLI Web 当前绑定 0.0.0.0:8000"));
    }

    #[test]
    fn render_created_server_lines_include_docker_runtime_summary() {
        let lines = render_created_server_lines(
            &sample_docker_server(),
            25565,
            None,
            CliServerRuntimeKind::Docker,
            &["paper".to_string()],
        );
        let joined = lines.join("\n");

        assert!(joined.contains("runtime: docker_itzg"));
        assert!(joined.contains("docker.image: itzg/minecraft-server:java21"));
        assert!(joined.contains("docker.container_name: sealantern-paper"));
        assert!(joined.contains("docker.command_mode: rcon"));
        assert!(joined.contains("docker.rcon_endpoint: 127.0.0.1:25575"));
        assert!(joined.contains("start:   sealantern server start docker-1"));
        assert!(joined.contains("doctor:  sealantern docker doctor"));
        assert!(joined.contains("compose: sealantern compose generate docker-1"));
        assert!(joined.contains("如目标是 itzg，请等待 health=healthy 后再发送命令"));
        assert!(joined.contains("hint:    命令通道: RCON 127.0.0.1:25575"));
        assert!(joined.contains("sealantern server send docker-1 <command>"));
    }
}
