use crate::models::server::{
    DockerItzgRuntimeConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    ServerStatusInfo,
};
use crate::services::server::manager::ServerRegistryDedupeReport;
use crate::services::server::manager::LocalLaunchDetail;
use crate::services::server::runtime::docker_itzg::DockerLaunchDetail;
use crate::utils::server_status::status_is_docker_command_ready;

use super::server_endpoint::render_docker_rcon_operator_hint;
use super::server_shared::format_aliases;

pub(super) fn render_server_management_help() -> &'static str {
    "server 管理子命令:\n  sealantern server list\n  sealantern server inspect <id|name|alias>\n  sealantern server status <id|name|alias>\n  sealantern server logs <id|name|alias> [--lines 50] [--follow] [--interval 1000]\n  sealantern server send <id|name|alias> <command...>\n  sealantern server start <id|name|alias> [--web [port]|--web:port] [--cli]\n  sealantern server stop <id|name|alias>\n  sealantern server force-stop <id|name|alias>\n  sealantern server restart <id|name|alias>\n  sealantern server dedupe-audit\n  sealantern server dedupe-apply"
}

pub(super) fn render_server_dedupe_report_lines(
    report: &ServerRegistryDedupeReport,
    applied: bool,
) -> Vec<String> {
    let mut lines = vec![format!(
        "重复记录{}报告: total_servers={} duplicate_groups={} removed={}",
        if applied { "清理" } else { "审计" },
        report.total_servers,
        report.duplicate_groups.len(),
        report.removed_ids.len()
    )];

    if report.duplicate_groups.is_empty() {
        lines.push("未发现需要处理的重复服务器记录。".to_string());
        return lines;
    }

    for group in &report.duplicate_groups {
        lines.push(format!(
            "group canonical={} name={} reasons=[{}]",
            group.canonical_id,
            group.canonical_name,
            group.reasons.join(", ")
        ));

        for entry in &group.entries {
            let disposition = if entry.id == group.canonical_id {
                "keep"
            } else if group.removable_ids.iter().any(|id| id == &entry.id) {
                "remove"
            } else {
                "blocked"
            };
            lines.push(format!(
                "  - [{}] id={} runtime={} path={} active={} created_at={} last_started_at={}",
                disposition,
                entry.id,
                entry.runtime_kind,
                entry.path,
                entry.active,
                entry.created_at,
                entry
                    .last_started_at
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string())
            ));
        }

        if !group.blocked_ids.is_empty() {
            lines.push(format!("  blocked_ids: {}", group.blocked_ids.join(", ")));
        }
    }

    if applied && !report.removed_ids.is_empty() {
        lines.push(format!("removed_ids: {}", report.removed_ids.join(", ")));
        lines.push(
            "说明: 仅删除 SeaLantern 记录与运行路径映射，不删除服务器目录或 Docker 容器。"
                .to_string(),
        );
    } else if !applied {
        lines.push(
            "提示: 如需按当前规则清理可移除记录，可执行 `sealantern server dedupe-apply`。"
                .to_string(),
        );
        lines.push("说明: 清理动作只移除记录，不删除服务器目录或 Docker 容器。".to_string());
    }

    lines
}

pub(super) fn render_server_list_lines(
    snapshots: &[(ServerInstance, ServerStatusInfo)],
) -> Vec<String> {
    if snapshots.is_empty() {
        return vec!["暂无服务器记录。".to_string()];
    }

    let mut lines = Vec::with_capacity(snapshots.len() + 2);
    lines.push(format!(
        "{:<36} {:<18} {:<24} {:<10} {:<20}",
        "ID", "名称", "runtime", "状态", "端口"
    ));
    lines.push("-".repeat(116));
    for (server, status) in snapshots {
        lines.push(format!(
            "{:<36} {:<18} {:<24} {:<10} {:<20}",
            server.id,
            server.name,
            runtime_summary(server),
            status.status.as_str(),
            port_summary(server)
        ));
    }
    lines
}

pub(super) fn render_server_status_lines(
    server: &ServerInstance,
    status: &ServerStatusInfo,
) -> Vec<String> {
    let mut lines = vec![
        "服务器状态".to_string(),
        format!("  id: {}", server.id),
        format!("  name: {}", server.name),
        format!("  aliases: {}", format_aliases(&server.aliases)),
        format!("  runtime: {}", runtime_summary(server)),
        format!("  status: {}", status.status.as_str()),
        format!("  ports: {}", port_summary(server)),
        format!("  pid: {}", format_optional_pid(status.pid)),
        format!("  uptime: {}", format_optional_uptime(status.uptime)),
    ];
    if let Some(detail) = &status.detail_message {
        lines.push(format!("  detail: {}", detail));
    }
    if let Some(error) = &status.error_message {
        lines.push(format!("  error: {}", error));
    }
    lines.extend(runtime_brief_lines(server));
    lines.extend(status_action_hint_lines(server, status));
    lines
}

pub(super) fn render_server_inspect_lines(
    server: &ServerInstance,
    status: &ServerStatusInfo,
) -> Vec<String> {
    let mut lines = vec![
        "服务器详情".to_string(),
        format!("  id: {}", server.id),
        format!("  name: {}", server.name),
        format!("  aliases: {}", format_aliases(&server.aliases)),
        format!("  runtime: {}", runtime_summary(server)),
        format!("  core: {}", server.core_type),
        format!("  mc_version: {}", server.mc_version),
        format!("  path: {}", server.path),
        format!("  ports: {}", port_summary(server)),
        format!("  memory_min_mb: {}", server.min_memory),
        format!("  memory_max_mb: {}", server.max_memory),
        format!("  created_at: {}", server.created_at),
        format!("  last_started_at: {}", format_optional_timestamp(server.last_started_at)),
        format!("  status: {}", status.status.as_str()),
        format!("  pid: {}", format_optional_pid(status.pid)),
        format!("  uptime: {}", format_optional_uptime(status.uptime)),
    ];
    if let Some(detail) = &status.detail_message {
        lines.push(format!("  detail: {}", detail));
    }
    if let Some(error) = &status.error_message {
        lines.push(format!("  error: {}", error));
    }
    lines.extend(runtime_detail_lines(server));
    lines
}

pub(super) fn redact_secret(secret: &str) -> String {
    if secret.trim().is_empty() {
        return "<empty>".to_string();
    }
    format!("<redacted:{} chars>", secret.chars().count())
}

fn runtime_summary(server: &ServerInstance) -> String {
    match &server.runtime {
        ServerRuntimeConfig::Local(runtime) => format!("local/{}", runtime.startup_mode),
        ServerRuntimeConfig::DockerItzg(runtime) => format!(
            "docker_itzg/{}/{}",
            runtime.docker_backend_kind.as_str(),
            runtime.command_mode.as_str()
        ),
    }
}

fn runtime_brief_lines(server: &ServerInstance) -> Vec<String> {
    match &server.runtime {
        ServerRuntimeConfig::Local(runtime) => vec![
            format!("  local.startup_mode: {}", runtime.startup_mode),
            format!("  local.java_path: {}", runtime.java_path),
            format!("  local.entry_path: {}", runtime.jar_path),
        ],
        ServerRuntimeConfig::DockerItzg(runtime) => {
            let mut lines = vec![
                format!("  docker.container_name: {}", runtime.container_name),
                format!("  docker.image: {}:{}", runtime.image, runtime.image_tag),
                format!("  docker.backend: {}", runtime.docker_backend_kind.as_str()),
                format!("  docker.command_mode: {}", runtime.command_mode.as_str()),
            ];
            if let Some(rcon) = &runtime.rcon {
                lines.push(format!("  docker.rcon_endpoint: {}:{}", rcon.host, rcon.port));
                lines.push(format!(
                    "  docker.command_hint: {}",
                    render_docker_rcon_operator_hint(rcon, &server.id)
                ));
            }
            lines
        }
    }
}

fn status_action_hint_lines(server: &ServerInstance, status: &ServerStatusInfo) -> Vec<String> {
    let mut lines = Vec::new();

    match &server.runtime {
        ServerRuntimeConfig::Local(_) => {
            if status.status.as_str() == "error" {
                lines.push(format!("  hint: sealantern server logs {} --lines 100", server.id));
                lines.push(format!("  hint: sealantern server start {}", server.id));
            }
        }
        ServerRuntimeConfig::DockerItzg(runtime) => {
            let detail = status.detail_message.as_deref().unwrap_or_default();
            if detail.contains("state=missing") {
                lines.push(format!(
                    "  hint: 容器当前不存在，可执行 sealantern server start {}",
                    server.id
                ));
            }

            if status.status.as_str() == "error" {
                lines.push(format!("  hint: sealantern server logs {} --lines 100", server.id));
                lines.push("  hint: sealantern docker doctor".to_string());
                lines.push(format!(
                    "  hint: 如容器卡死，可执行 sealantern server force-stop {}",
                    server.id
                ));
            }

            if runtime.command_mode.as_str() == "rcon" && status_is_docker_command_ready(status) {
                lines.push(format!("  hint: sealantern server send {} say hello", server.id));
            } else if runtime.command_mode.as_str() == "rcon" && status.status.as_str() == "running"
            {
                lines.push(
                    "  hint: 容器已 running，但命令通道可能尚未 ready；如目标是 itzg，请等待 health=healthy 后再发送命令"
                        .to_string(),
                );
            }
        }
    }

    lines
}

fn runtime_detail_lines(server: &ServerInstance) -> Vec<String> {
    match &server.runtime {
        ServerRuntimeConfig::Local(runtime) => render_local_runtime_detail_lines(server, runtime),
        ServerRuntimeConfig::DockerItzg(runtime) => {
            render_docker_runtime_detail_lines(server, runtime)
        }
    }
}

fn render_local_runtime_detail_lines(
    server: &ServerInstance,
    runtime: &LocalRuntimeConfig,
) -> Vec<String> {
    let mut lines = vec![
        format!("  local.java_path: {}", runtime.java_path),
        format!("  local.jar_path: {}", runtime.jar_path),
        format!("  local.startup_mode: {}", runtime.startup_mode),
    ];
    if let Some(custom_command) = &runtime.custom_command {
        lines.push(format!("  local.custom_command: {}", custom_command));
    }
    if !runtime.jvm_args.is_empty() {
        lines.push(format!("  local.jvm_args: {}", runtime.jvm_args.join(" ")));
    }
    if let Some(detail) = local_launch_detail_lines(server) {
        lines.extend(detail);
    }
    lines
}

fn local_launch_detail_lines(server: &ServerInstance) -> Option<Vec<String>> {
    let detail =
        crate::services::server::manager::build_local_launch_detail_for_server(server).ok()?;
    Some(render_local_launch_detail_lines(&detail))
}

fn render_local_launch_detail_lines(detail: &LocalLaunchDetail) -> Vec<String> {
    let mut lines = vec![
        format!("  local.launch_target: {}", detail.launch_target),
        format!("  local.command_preview: {}", detail.command_preview),
    ];
    if detail.effective_jvm_args.is_empty() {
        lines.push("  local.effective_jvm_args: []".to_string());
    } else {
        lines.push(format!("  local.effective_jvm_args: {}", detail.effective_jvm_args.join(" ")));
    }
    lines
}

fn render_docker_runtime_detail_lines(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
) -> Vec<String> {
    let mut lines = vec![
        format!("  docker.image: {}:{}", runtime.image, runtime.image_tag),
        format!("  docker.container_name: {}", runtime.container_name),
        format!("  docker.type: {}", runtime.type_value),
        format!("  docker.version: {}", runtime.version),
        format!("  docker.data_dir_mount: {}", runtime.data_dir_mount),
        format!("  docker.published_game_port: {}", runtime.published_game_port),
        format!("  docker.backend: {}", runtime.docker_backend_kind.as_str()),
        format!("  docker.command_mode: {}", runtime.command_mode.as_str()),
        format!("  docker.cpu_policy.mode: {}", runtime.cpu_policy.mode.as_str()),
        format!(
            "  docker.cpu_policy.count: {}",
            runtime
                .cpu_policy
                .count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string())
        ),
        format!(
            "  docker.cpu_policy.explicit_set: {}",
            runtime
                .cpu_policy
                .explicit_set
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("-")
        ),
        format!(
            "  docker.cpu_policy.sync_active_processor_count: {}",
            runtime.cpu_policy.sync_active_processor_count
        ),
        format!("  docker.jvm_preset: {}", runtime.jvm_preset.preset.as_str()),
        format!("  docker.extra_ports: {}", format_docker_extra_ports(runtime)),
        format!("  docker.volume_mounts: {}", format_docker_volume_mounts(runtime)),
        format!("  docker.env_count: {}", runtime.env.len()),
    ];
    if !runtime.jvm_args.is_empty() {
        lines.push(format!("  docker.jvm_args: {}", runtime.jvm_args.join(" ")));
    }
    if let Some(detail) = docker_launch_detail_lines(server) {
        lines.extend(detail);
    }
    if let Some(rcon) = &runtime.rcon {
        lines.push(format!("  docker.rcon_host: {}", rcon.host));
        lines.push(format!("  docker.rcon_port: {}", rcon.port));
        lines.push(format!("  docker.rcon_password: {}", redact_secret(&rcon.password)));
    }
    lines
}

fn docker_launch_detail_lines(server: &ServerInstance) -> Option<Vec<String>> {
    let detail =
        crate::services::server::manager::build_docker_launch_detail_for_server(server).ok()?;
    Some(render_docker_launch_detail_lines(&detail))
}

fn render_docker_launch_detail_lines(detail: &DockerLaunchDetail) -> Vec<String> {
    let mut lines = vec![
        format!("  docker.launch.runtime_kind: {}", detail.runtime_kind),
        format!("  docker.launch.image: {}:{}", detail.image, detail.image_tag),
        format!("  docker.launch.container_name: {}", detail.container_name),
        format!(
            "  docker.launch.cpuset_applied: {}",
            detail.cpuset_applied.as_deref().unwrap_or("-")
        ),
        format!("  docker.launch.jvm_preset: {}", detail.jvm_preset),
        format!(
            "  docker.launch.jvm_opts_preview: {}",
            detail.jvm_opts_preview.as_deref().unwrap_or("-")
        ),
        format!(
            "  docker.launch.jvm_xx_opts_preview: {}",
            detail.jvm_xx_opts_preview.as_deref().unwrap_or("-")
        ),
        format!(
            "  docker.launch.active_processor_count_status: {}",
            detail.active_processor_count_status
        ),
        format!(
            "  docker.launch.active_processor_count_value: {}",
            detail
                .active_processor_count_value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string())
        ),
        format!("  docker.launch.jvm_opts_args_count: {}", detail.jvm_opts_args_count),
        format!("  docker.launch.jvm_xx_opts_args_count: {}", detail.jvm_xx_opts_args_count),
        format!(
            "  docker.launch.jvm_opts_overridden_by_runtime_env: {}",
            detail.jvm_opts_overridden_by_runtime_env
        ),
        format!(
            "  docker.launch.jvm_xx_opts_overridden_by_runtime_env: {}",
            detail.jvm_xx_opts_overridden_by_runtime_env
        ),
        format!("  docker.launch.command_preview: {}", detail.command_preview),
    ];

    if detail.docker_args_preview.is_empty() {
        lines.push("  docker.launch.args_preview: []".to_string());
    } else {
        lines.push(format!(
            "  docker.launch.args_preview: {}",
            detail.docker_args_preview.join(" ")
        ));
    }

    lines
}

fn port_summary(server: &ServerInstance) -> String {
    match &server.runtime {
        ServerRuntimeConfig::Local(_) => server.port.to_string(),
        ServerRuntimeConfig::DockerItzg(runtime) => {
            let mut ports = vec![format!("{}->25565/tcp", runtime.published_game_port)];
            ports.extend(runtime.extra_ports.iter().map(|port| {
                let protocol = if port.protocol.trim().is_empty() {
                    "tcp"
                } else {
                    port.protocol.as_str()
                };
                format!("{}->{}{}{}", port.host_port, port.container_port, "/", protocol)
            }));
            ports.join(",")
        }
    }
}

fn format_docker_extra_ports(runtime: &DockerItzgRuntimeConfig) -> String {
    if runtime.extra_ports.is_empty() {
        return "[]".to_string();
    }

    let ports = runtime
        .extra_ports
        .iter()
        .map(|port| {
            let protocol = if port.protocol.trim().is_empty() {
                "tcp"
            } else {
                port.protocol.as_str()
            };
            format!("{}->{}{}{}", port.host_port, port.container_port, "/", protocol)
        })
        .collect::<Vec<_>>();
    format!("[{}]", ports.join(", "))
}

fn format_docker_volume_mounts(runtime: &DockerItzgRuntimeConfig) -> String {
    if runtime.volume_mounts.is_empty() {
        return "[]".to_string();
    }

    let mounts = runtime
        .volume_mounts
        .iter()
        .map(|mount| {
            let suffix = if mount.read_only { ":ro" } else { "" };
            format!("{}:{}{}", mount.source, mount.target, suffix)
        })
        .collect::<Vec<_>>();
    format!("[{}]", mounts.join(", "))
}

fn format_optional_pid(pid: Option<u32>) -> String {
    pid.map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_uptime(uptime: Option<u64>) -> String {
    uptime
        .map(|value| format!("{}s", value))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_timestamp(timestamp: Option<u64>) -> String {
    timestamp
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        redact_secret, render_server_dedupe_report_lines, render_server_inspect_lines,
        render_server_list_lines, render_server_management_help, render_server_status_lines,
    };
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, DockerBackendKind, DockerCommandMode,
        DockerItzgRuntimeConfig, JvmPresetConfig, JvmPresetId, LocalRuntimeConfig, PublishedPort,
        RconConfig, ServerInstance, ServerRuntimeConfig, ServerStatus, ServerStatusInfo,
        VolumeMount,
    };
    use crate::services::server::manager::{
        DuplicateServerRecordEntry, DuplicateServerRecordGroup, ServerRegistryDedupeReport,
    };
    use std::collections::BTreeMap;

    fn sample_status(status: ServerStatus) -> ServerStatusInfo {
        ServerStatusInfo {
            id: "fabric-main-id".to_string(),
            status,
            pid: Some(1234),
            uptime: Some(88),
            detail_message: Some("runtime=local/jar".to_string()),
            error_message: None,
        }
    }

    fn sample_local_server() -> ServerInstance {
        ServerInstance {
            id: "fabric-main-id".to_string(),
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
            last_started_at: Some(10),
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/fabric-main/server.jar".to_string(),
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
        let mut env = BTreeMap::new();
        env.insert("TYPE".to_string(), "PAPER".to_string());

        ServerInstance {
            id: "paper-docker-id".to_string(),
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
            last_started_at: Some(20),
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sealantern-paper".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/docker/paper".to_string(),
                published_game_port: 25565,
                env,
                extra_ports: vec![PublishedPort {
                    host_port: 24454,
                    container_port: 24454,
                    protocol: "udp".to_string(),
                }],
                volume_mounts: vec![VolumeMount {
                    source: "E:/docker/paper/plugins".to_string(),
                    target: "/data/plugins".to_string(),
                    read_only: true,
                }],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: Some(RconConfig {
                    host: "127.0.0.1".to_string(),
                    port: 25575,
                    password: "secret-pass".to_string(),
                }),
                jvm_args: vec!["-Dfoo=bar".to_string()],
                cpu_policy: CpuPolicyConfig {
                    mode: CpuPolicyMode::Explicit,
                    count: None,
                    explicit_set: Some("0-3,6".to_string()),
                    sync_active_processor_count: true,
                },
                jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
            }),
        }
    }

    #[test]
    fn render_server_management_help_includes_restart_and_force_stop() {
        let help = render_server_management_help();
        assert!(help.contains("server restart <id|name|alias>"));
        assert!(help.contains("server force-stop <id|name|alias>"));
    }

    #[test]
    fn render_server_list_lines_include_docker_runtime_and_ports() {
        let lines = render_server_list_lines(&[(
            sample_docker_server(),
            sample_status(ServerStatus::Running),
        )]);
        let joined = lines.join("\n");
        assert!(joined.contains("docker_itzg/cli/rcon"));
        assert!(joined.contains("25565->25565/tcp,24454->24454/udp"));
    }

    #[test]
    fn render_server_status_lines_include_local_runtime_brief() {
        let lines = render_server_status_lines(
            &sample_local_server(),
            &sample_status(ServerStatus::Running),
        );
        let joined = lines.join("\n");
        assert!(joined.contains("runtime: local/jar"));
        assert!(joined.contains("local.entry_path: E:/servers/fabric-main/server.jar"));
        assert!(joined.contains("uptime: 88s"));
        assert!(joined.contains("detail: runtime=local/jar"));
    }

    #[test]
    fn render_server_inspect_lines_include_local_launch_detail() {
        let lines = render_server_inspect_lines(
            &sample_local_server(),
            &sample_status(ServerStatus::Running),
        );
        let joined = lines.join("\n");

        assert!(joined.contains("local.launch_target: E:/servers/fabric-main/server.jar"));
        assert!(joined.contains("local.command_preview:"));
        assert!(joined.contains("-jar server.jar nogui"));
        assert!(joined.contains("local.effective_jvm_args:"));
        assert!(joined.contains("-Dfile.encoding=UTF-8"));
    }

    #[test]
    fn render_server_status_lines_include_docker_command_hint_for_rcon_runtime() {
        let lines = render_server_status_lines(
            &sample_docker_server(),
            &sample_status(ServerStatus::Running),
        );
        let joined = lines.join("\n");

        assert!(joined.contains("docker.rcon_endpoint: 127.0.0.1:25575"));
        assert!(joined.contains("docker.command_hint: 命令通道: RCON 127.0.0.1:25575"));
        assert!(joined.contains("sealantern server send paper-docker-id <command>"));
        assert!(joined.contains("hint: sealantern server send paper-docker-id say hello"));
    }

    #[test]
    fn render_server_status_lines_waits_for_healthy_before_send_hint() {
        let server = sample_docker_server();
        let status = ServerStatusInfo {
            id: server.id.clone(),
            status: ServerStatus::Running,
            pid: Some(55),
            uptime: Some(5),
            detail_message: Some(
                "runtime=docker_itzg container=sealantern-paper state=running running=true health=starting exit_code=0 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: None,
        };

        let joined = render_server_status_lines(&server, &status).join("\n");

        assert!(!joined.contains("hint: sealantern server send paper-docker-id say hello"));
        assert!(joined.contains("health=healthy 后再发送命令"));
    }

    #[test]
    fn render_server_status_lines_include_actionable_hints_for_docker_error_and_missing_container()
    {
        let server = sample_docker_server();

        let missing_status = ServerStatusInfo {
            id: server.id.clone(),
            status: ServerStatus::Stopped,
            pid: None,
            uptime: None,
            detail_message: Some(
                "runtime=docker_itzg container=sealantern-paper state=missing".to_string(),
            ),
            error_message: None,
        };
        let missing_lines = render_server_status_lines(&server, &missing_status).join("\n");
        assert!(missing_lines
            .contains("容器当前不存在，可执行 sealantern server start paper-docker-id"));

        let error_status = ServerStatusInfo {
            id: server.id.clone(),
            status: ServerStatus::Error,
            pid: None,
            uptime: Some(12),
            detail_message: Some(
                "runtime=docker_itzg container=sealantern-paper state=exited running=false health=none exit_code=137 backend=cli command_mode=rcon"
                    .to_string(),
            ),
            error_message: Some("Docker 容器已退出: container=sealantern-paper, status=exited, exit_code=137".to_string()),
        };
        let error_lines = render_server_status_lines(&server, &error_status).join("\n");
        assert!(error_lines.contains("hint: sealantern server logs paper-docker-id --lines 100"));
        assert!(error_lines.contains("hint: sealantern docker doctor"));
        assert!(error_lines
            .contains("hint: 如容器卡死，可执行 sealantern server force-stop paper-docker-id"));
    }

    #[test]
    fn render_server_inspect_lines_include_docker_runtime_detail_and_redaction() {
        let mut status = sample_status(ServerStatus::Error);
        status.error_message = Some("rcon unavailable".to_string());
        let lines = render_server_inspect_lines(&sample_docker_server(), &status);
        let joined = lines.join("\n");
        assert!(joined.contains("docker.container_name: sealantern-paper"));
        assert!(joined.contains("docker.extra_ports: [24454->24454/udp]"));
        assert!(joined.contains("docker.volume_mounts: [E:/docker/paper/plugins:/data/plugins:ro]"));
        assert!(joined.contains("docker.cpu_policy.mode: explicit"));
        assert!(joined.contains("docker.cpu_policy.explicit_set: 0-3,6"));
        assert!(joined.contains("docker.cpu_policy.sync_active_processor_count: true"));
        assert!(joined.contains("docker.jvm_preset: aikar_g1"));
        assert!(joined.contains("docker.jvm_args: -Dfoo=bar"));
        assert!(joined.contains("docker.launch.runtime_kind: docker_itzg"));
        assert!(joined.contains("docker.launch.cpuset_applied: 0-3,6"));
        assert!(joined.contains("docker.launch.jvm_preset: aikar_g1"));
        assert!(joined.contains("docker.launch.jvm_opts_preview: -Dfoo=bar"));
        assert!(joined.contains("docker.launch.jvm_xx_opts_preview:"));
        assert!(joined.contains("-XX:ActiveProcessorCount=5"));
        assert!(joined.contains("docker.launch.command_preview: docker run -d --name sealantern-paper --cpuset-cpus 0-3,6"));
        assert!(joined.contains(
            "docker.launch.args_preview: run -d --name sealantern-paper --cpuset-cpus 0-3,6"
        ));
        assert!(joined.contains("docker.rcon_password: <redacted:11 chars>"));
        assert!(joined.contains("error: rcon unavailable"));
    }

    #[test]
    fn redact_secret_hides_password_length_only() {
        assert_eq!(redact_secret("secret-pass"), "<redacted:11 chars>");
    }

    #[test]
    fn render_server_dedupe_report_lines_explains_record_only_cleanup() {
        let report = ServerRegistryDedupeReport {
            total_servers: 3,
            duplicate_groups: vec![DuplicateServerRecordGroup {
                canonical_id: "keep-1".to_string(),
                canonical_name: "paper".to_string(),
                reasons: vec!["name".to_string(), "path".to_string()],
                entries: vec![
                    DuplicateServerRecordEntry {
                        id: "keep-1".to_string(),
                        name: "paper".to_string(),
                        path: "E:/servers/paper".to_string(),
                        runtime_kind: "local".to_string(),
                        created_at: 10,
                        last_started_at: Some(12),
                        active: false,
                    },
                    DuplicateServerRecordEntry {
                        id: "drop-1".to_string(),
                        name: "paper".to_string(),
                        path: "E:/servers/paper".to_string(),
                        runtime_kind: "local".to_string(),
                        created_at: 1,
                        last_started_at: None,
                        active: false,
                    },
                ],
                removable_ids: vec!["drop-1".to_string()],
                blocked_ids: vec![],
            }],
            removed_ids: vec!["drop-1".to_string()],
        };

        let joined = render_server_dedupe_report_lines(&report, true).join("\n");
        assert!(joined.contains("重复记录清理报告"));
        assert!(joined.contains("[keep] id=keep-1"));
        assert!(joined.contains("[remove] id=drop-1"));
        assert!(joined.contains("不删除服务器目录或 Docker 容器"));
    }
}
