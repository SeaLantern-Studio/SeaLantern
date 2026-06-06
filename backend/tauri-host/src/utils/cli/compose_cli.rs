use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::models::server::{DockerItzgRuntimeConfig, ServerInstance, VolumeMount};
use crate::services::server::runtime::docker_itzg::resolve_docker_launch_spec;
use sea_lantern_docker_core::resolve_docker_cpuset;

use super::server_ref::resolve_server_reference;
use super::server_shared::{trace_compose_action, trace_compose_error};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ComposeCommand {
    Help,
    Generate {
        target: String,
        output: Option<String>,
        full_stack: bool,
        sealantern_image: String,
        sealantern_http_port: u16,
        static_dir: Option<String>,
        sealantern_data_dir: Option<String>,
        docker_socket: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComposeGenerateOptions {
    output: Option<String>,
    full_stack: bool,
    sealantern_image: String,
    sealantern_http_port: u16,
    static_dir: Option<String>,
    sealantern_data_dir: Option<String>,
    docker_socket: String,
}

const DEFAULT_SEALANTERN_IMAGE: &str = "ghcr.io/sealantern-studio/sealantern:latest";
const DEFAULT_SEALANTERN_HTTP_PORT: u16 = 3000;
const DEFAULT_DOCKER_SOCKET: &str = "/var/run/docker.sock:/var/run/docker.sock";

pub(super) fn handle_compose_command(args: &[String]) -> i32 {
    trace_compose_action("invoke", &format!("args={}", args.join(" | ")));
    match parse_compose_command(args) {
        Ok(ComposeCommand::Help) => {
            print_compose_help();
            0
        }
        Ok(ComposeCommand::Generate {
            target,
            output,
            full_stack,
            sealantern_image,
            sealantern_http_port,
            static_dir,
            sealantern_data_dir,
            docker_socket,
        }) => {
            let options = ComposeGenerateOptions {
                output,
                full_stack,
                sealantern_image,
                sealantern_http_port,
                static_dir,
                sealantern_data_dir,
                docker_socket,
            };
            match generate_compose(target, options) {
                Ok(()) => 0,
                Err(err) => {
                    trace_compose_error("generate_failed", "", &err);
                    eprintln!("compose generate 失败: {}", err);
                    2
                }
            }
        }
        Err(err) => {
            trace_compose_error("parse_failed", "", &err);
            eprintln!("compose 参数错误: {}", err);
            print_compose_help();
            2
        }
    }
}

pub(super) fn print_compose_help() {
    println!(
        "用法: sealantern compose generate <id|name|alias> [--output <compose.yaml>] [--full-stack] [--sealantern-image <image>] [--http-port 3000] [--static-dir <dir>] [--sealantern-data <dir>] [--docker-socket <mount>]"
    );
    println!("  generate  为 docker_itzg 服务器导出 compose YAML");
}

fn parse_compose_command(args: &[String]) -> Result<ComposeCommand, String> {
    let Some(first) = args.first() else {
        return Ok(ComposeCommand::Help);
    };

    match first.as_str() {
        "help" | "--help" | "-h" => Ok(ComposeCommand::Help),
        "generate" => parse_generate_command(args),
        other => Err(format!("未知 compose 子命令: {}", other)),
    }
}

fn parse_generate_command(args: &[String]) -> Result<ComposeCommand, String> {
    let target = args
        .get(1)
        .cloned()
        .ok_or_else(|| "generate 需要服务器 ID / 名称 / 别名".to_string())?;

    let mut options = ComposeGenerateOptions {
        output: None,
        full_stack: false,
        sealantern_image: DEFAULT_SEALANTERN_IMAGE.to_string(),
        sealantern_http_port: DEFAULT_SEALANTERN_HTTP_PORT,
        static_dir: None,
        sealantern_data_dir: None,
        docker_socket: DEFAULT_DOCKER_SOCKET.to_string(),
    };
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--output" | "-o" => {
                let value = args
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| format!("{} 缺少值", args[index]))?;
                options.output = Some(value);
                index += 2;
            }
            other if other.starts_with("--output=") => {
                options.output = Some(other[9..].to_string());
                index += 1;
            }
            "--full-stack" => {
                options.full_stack = true;
                index += 1;
            }
            "--sealantern-image" => {
                options.sealantern_image = args
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| format!("{} 缺少值", args[index]))?;
                index += 2;
            }
            other if other.starts_with("--sealantern-image=") => {
                options.sealantern_image = other[19..].to_string();
                index += 1;
            }
            "--http-port" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("{} 缺少值", args[index]))?;
                options.sealantern_http_port = value
                    .parse::<u16>()
                    .map_err(|_| format!("无效的 HTTP 端口: {}", value))?;
                index += 2;
            }
            other if other.starts_with("--http-port=") => {
                let value = &other[12..];
                options.sealantern_http_port = value
                    .parse::<u16>()
                    .map_err(|_| format!("无效的 HTTP 端口: {}", value))?;
                index += 1;
            }
            "--static-dir" => {
                options.static_dir = Some(
                    args.get(index + 1)
                        .cloned()
                        .ok_or_else(|| format!("{} 缺少值", args[index]))?,
                );
                index += 2;
            }
            other if other.starts_with("--static-dir=") => {
                options.static_dir = Some(other[13..].to_string());
                index += 1;
            }
            "--sealantern-data" => {
                options.sealantern_data_dir = Some(
                    args.get(index + 1)
                        .cloned()
                        .ok_or_else(|| format!("{} 缺少值", args[index]))?,
                );
                index += 2;
            }
            other if other.starts_with("--sealantern-data=") => {
                options.sealantern_data_dir = Some(other[18..].to_string());
                index += 1;
            }
            "--docker-socket" => {
                options.docker_socket = args
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| format!("{} 缺少值", args[index]))?;
                index += 2;
            }
            other if other.starts_with("--docker-socket=") => {
                options.docker_socket = other[16..].to_string();
                index += 1;
            }
            other => return Err(format!("generate 不支持的参数: {}", other)),
        }
    }

    Ok(ComposeCommand::Generate {
        target,
        output: options.output,
        full_stack: options.full_stack,
        sealantern_image: options.sealantern_image,
        sealantern_http_port: options.sealantern_http_port,
        static_dir: options.static_dir,
        sealantern_data_dir: options.sealantern_data_dir,
        docker_socket: options.docker_socket,
    })
}

fn generate_compose(target: String, options: ComposeGenerateOptions) -> Result<(), String> {
    let server = resolve_server_reference(&target)?;
    let runtime = server
        .docker_itzg_runtime()
        .ok_or_else(|| format!("当前服务器不是 docker_itzg: {}", server.runtime_kind))?;

    trace_compose_action(
        "compose_generate",
        &format!(
            "server_id={} output={} full_stack={} sealantern_http_port={}",
            server.id,
            options.output.as_deref().unwrap_or("stdout"),
            options.full_stack,
            options.sealantern_http_port
        ),
    );

    let compose_yaml = build_compose_yaml(&server, runtime, &options)?;
    if let Some(path) = options.output {
        let output_path = PathBuf::from(&path);
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| format!("创建 compose 输出目录失败: {}", err))?;
            }
        }
        std::fs::write(&output_path, &compose_yaml)
            .map_err(|err| format!("写入 compose 文件失败: {}", err))?;
        println!("已生成 compose 文件: {}", output_path.to_string_lossy());
    } else {
        println!("{}", compose_yaml);
    }

    Ok(())
}

fn build_compose_yaml(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
    options: &ComposeGenerateOptions,
) -> Result<String, String> {
    let mut lines = Vec::new();

    if options.full_stack {
        append_sealantern_service(&mut lines, server, runtime, options)?;
        lines.push("".to_string());
    }

    let service_name = sanitize_compose_service_name(&runtime.container_name);
    if lines.is_empty() {
        lines.push("services:".to_string());
    }
    lines.push(format!("  {}:", service_name));
    lines.push(format!("    image: {}:{}", runtime.image, runtime.image_tag));
    lines.push(format!("    container_name: {}", runtime.container_name));
    lines.push("    restart: unless-stopped".to_string());

    if options.full_stack {
        lines.push("    depends_on:".to_string());
        lines.push("      - sealantern".to_string());
    }

    let environment = build_compose_environment(server, runtime)?;
    if !environment.is_empty() {
        lines.push("    environment:".to_string());
        for (key, value) in environment {
            lines.push(format!("      {}: \"{}\"", key, escape_yaml_double_quoted(&value)));
        }
    }

    if let Some(cpuset) = resolve_docker_cpuset(&runtime.cpu_policy)? {
        lines.push(format!("    cpuset: \"{}\"", escape_yaml_double_quoted(&cpuset)));
    }

    let ports = build_compose_ports(runtime);
    if !ports.is_empty() {
        lines.push("    ports:".to_string());
        for port in ports {
            lines.push(format!("      - \"{}\"", port));
        }
    }

    let volumes = build_compose_volumes(runtime);
    if !volumes.is_empty() {
        lines.push("    volumes:".to_string());
        for volume in volumes {
            lines.push(format!("      - \"{}\"", escape_yaml_double_quoted(&volume)));
        }
    }

    lines.push("".to_string());
    lines.push("# Generated by Sea Lantern CLI".to_string());
    lines.push(format!("# server_id={}", server.id));
    lines.push(format!("# server_name={}", server.name));
    lines.push(format!("# runtime_kind={}", server.runtime_kind));

    Ok(lines.join("\n"))
}

fn append_sealantern_service(
    lines: &mut Vec<String>,
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
    options: &ComposeGenerateOptions,
) -> Result<(), String> {
    let container_root = normalize_compose_mount_path(
        options
            .sealantern_data_dir
            .as_deref()
            .unwrap_or(server.path.as_str()),
    );
    let host_root = normalize_compose_mount_path(&server.path);

    lines.push("services:".to_string());
    lines.push("  sealantern:".to_string());
    lines.push(format!("    image: {}", options.sealantern_image));
    lines.push("    container_name: sealantern".to_string());
    lines.push("    restart: unless-stopped".to_string());
    lines.push("    environment:".to_string());
    lines.push("      SEALANTERN_HEADLESS_HTTP: \"1\"".to_string());
    lines.push(
        "      # 默认行为是仅监听 127.0.0.1；如需容器外访问，才显式改成 0.0.0.0:3000 或其他实际需要监听的地址"
            .to_string(),
    );
    lines.push("      SEALANTERN_HTTP_BIND: \"127.0.0.1:3000\"".to_string());
    lines.push(format!(
        "      SEALANTERN_SERVERS_CONTAINER_ROOT: \"{}\"",
        escape_yaml_double_quoted(&container_root)
    ));
    lines.push(format!(
        "      SEALANTERN_SERVERS_HOST_ROOT: \"{}\"",
        escape_yaml_double_quoted(&host_root)
    ));

    if let Some(static_dir) = &options.static_dir {
        lines.push(format!("      STATIC_DIR: \"{}\"", escape_yaml_double_quoted(static_dir)));
    }

    if runtime.command_mode.as_str() == "rcon" {
        if let Some(rcon) = &runtime.rcon {
            lines.push(format!(
                "      SEALANTERN_DOCKER_RCON_HOST: \"{}\"",
                escape_yaml_double_quoted(&rcon.host)
            ));
        }
    }

    lines.push("    ports:".to_string());
    lines.push(format!("      - \"{}:3000/tcp\"", options.sealantern_http_port));
    lines.push("    volumes:".to_string());
    lines.push(format!(
        "      - \"{}:{}\"",
        escape_yaml_double_quoted(&host_root),
        escape_yaml_double_quoted(&container_root)
    ));
    lines.push(format!("      - \"{}\"", escape_yaml_double_quoted(&options.docker_socket)));

    Ok(())
}

fn build_compose_environment(
    server: &ServerInstance,
    runtime: &DockerItzgRuntimeConfig,
) -> Result<BTreeMap<String, String>, String> {
    let settings = crate::models::settings::AppSettings::default();
    let launch_spec = resolve_docker_launch_spec(server, runtime, &settings)?;
    Ok(launch_spec.environment.into_iter().collect())
}

fn build_compose_ports(runtime: &DockerItzgRuntimeConfig) -> Vec<String> {
    let mut ports = vec![format!("{}:25565/tcp", runtime.published_game_port)];
    for port in &runtime.extra_ports {
        let protocol = if port.protocol.trim().is_empty() {
            "tcp"
        } else {
            port.protocol.as_str()
        };
        ports.push(format!("{}:{}/{}", port.host_port, port.container_port, protocol));
    }
    ports
}

fn build_compose_volumes(runtime: &DockerItzgRuntimeConfig) -> Vec<String> {
    let mut volumes =
        vec![format!("{}:/data", normalize_compose_mount_path(&runtime.data_dir_mount))];
    for mount in &runtime.volume_mounts {
        volumes.push(format_volume_mount(mount));
    }
    volumes
}

fn format_volume_mount(mount: &VolumeMount) -> String {
    let mut value = format!(
        "{}:{}",
        normalize_compose_mount_path(&mount.source),
        normalize_compose_mount_path(&mount.target)
    );
    if mount.read_only {
        value.push_str(":ro");
    }
    value
}

fn normalize_compose_mount_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn sanitize_compose_service_name(value: &str) -> String {
    let mut service = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            service.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_') {
            service.push(ch);
        } else {
            service.push('-');
        }
    }

    let trimmed = service.trim_matches('-');
    if trimmed.is_empty() {
        "minecraft".to_string()
    } else {
        trimmed.to_string()
    }
}

fn escape_yaml_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{
        build_compose_yaml, parse_compose_command, sanitize_compose_service_name, ComposeCommand,
        ComposeGenerateOptions, DEFAULT_DOCKER_SOCKET, DEFAULT_SEALANTERN_HTTP_PORT,
        DEFAULT_SEALANTERN_IMAGE,
    };
    use crate::models::server::{
        CpuPolicyConfig, CpuPolicyMode, DockerBackendKind, DockerCommandMode,
        DockerItzgRuntimeConfig, JvmPresetConfig, JvmPresetId, PublishedPort, ServerInstance,
        ServerRuntimeConfig, VolumeMount,
    };
    use std::collections::BTreeMap;

    fn sample_server() -> ServerInstance {
        let mut env = BTreeMap::new();
        env.insert("EULA".to_string(), "TRUE".to_string());
        env.insert("GUI".to_string(), "FALSE".to_string());
        env.insert("CONSOLE".to_string(), "TRUE".to_string());
        env.insert("MEMORY".to_string(), "4G".to_string());

        ServerInstance {
            id: "docker-id".to_string(),
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
                env,
                extra_ports: vec![PublishedPort {
                    host_port: 25575,
                    container_port: 25575,
                    protocol: "tcp".to_string(),
                }],
                volume_mounts: vec![VolumeMount {
                    source: "E:/docker/plugins".to_string(),
                    target: "/plugins".to_string(),
                    read_only: true,
                }],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: vec!["-Dfoo=bar".to_string()],
                cpu_policy: CpuPolicyConfig {
                    mode: CpuPolicyMode::Count,
                    count: Some(2),
                    explicit_set: None,
                    sync_active_processor_count: true,
                },
                jvm_preset: JvmPresetConfig { preset: JvmPresetId::G1Basic },
            }),
        }
    }

    #[test]
    fn parse_compose_generate_supports_output_flag() {
        let args = vec![
            "generate".to_string(),
            "paper".to_string(),
            "--output".to_string(),
            "E:/tmp/compose.yaml".to_string(),
        ];

        let command = parse_compose_command(&args).unwrap();
        assert_eq!(
            command,
            ComposeCommand::Generate {
                target: "paper".to_string(),
                output: Some("E:/tmp/compose.yaml".to_string()),
                full_stack: false,
                sealantern_image: DEFAULT_SEALANTERN_IMAGE.to_string(),
                sealantern_http_port: DEFAULT_SEALANTERN_HTTP_PORT,
                static_dir: None,
                sealantern_data_dir: None,
                docker_socket: DEFAULT_DOCKER_SOCKET.to_string(),
            }
        );
    }

    #[test]
    fn parse_compose_generate_supports_full_stack_flags() {
        let args = vec![
            "generate".to_string(),
            "paper".to_string(),
            "--full-stack".to_string(),
            "--http-port".to_string(),
            "8080".to_string(),
            "--sealantern-image=ghcr.io/test/sealantern:dev".to_string(),
            "--static-dir".to_string(),
            "/app/dist".to_string(),
            "--sealantern-data".to_string(),
            "/app/data/servers".to_string(),
            "--docker-socket".to_string(),
            "/var/run/docker.sock:/var/run/docker.sock".to_string(),
        ];

        let command = parse_compose_command(&args).unwrap();
        assert_eq!(
            command,
            ComposeCommand::Generate {
                target: "paper".to_string(),
                output: None,
                full_stack: true,
                sealantern_image: "ghcr.io/test/sealantern:dev".to_string(),
                sealantern_http_port: 8080,
                static_dir: Some("/app/dist".to_string()),
                sealantern_data_dir: Some("/app/data/servers".to_string()),
                docker_socket: "/var/run/docker.sock:/var/run/docker.sock".to_string(),
            }
        );
    }

    #[test]
    fn build_compose_yaml_contains_expected_docker_shape() {
        let server = sample_server();
        let runtime = server.docker_itzg_runtime().unwrap();
        let options = ComposeGenerateOptions {
            output: None,
            full_stack: false,
            sealantern_image: DEFAULT_SEALANTERN_IMAGE.to_string(),
            sealantern_http_port: DEFAULT_SEALANTERN_HTTP_PORT,
            static_dir: None,
            sealantern_data_dir: None,
            docker_socket: DEFAULT_DOCKER_SOCKET.to_string(),
        };

        let yaml = build_compose_yaml(&server, runtime, &options).unwrap();
        assert!(yaml.contains("services:"));
        assert!(yaml.contains("image: itzg/minecraft-server:java21"));
        assert!(yaml.contains("container_name: sealantern-paper"));
        assert!(yaml.contains("25565:25565/tcp"));
        assert!(yaml.contains("25575:25575/tcp"));
        assert!(yaml.contains("E:/docker/paper:/data"));
        assert!(yaml.contains("E:/docker/plugins:/plugins:ro"));
        assert!(yaml.contains("TYPE: \"PAPER\""));
        assert!(yaml.contains("VERSION: \"1.21.1\""));
        assert!(yaml.contains("GUI: \"FALSE\""));
        assert!(yaml.contains("CONSOLE: \"TRUE\""));
        assert!(yaml.contains("cpuset: \"0-1\""));
        assert!(yaml.contains("JVM_OPTS: \"-Dfoo=bar\""));
        assert!(yaml.contains("JVM_XX_OPTS:"));
        assert!(yaml.contains("-XX:ActiveProcessorCount=2"));
    }

    #[test]
    fn build_compose_yaml_respects_runtime_env_jvm_takeover() {
        let mut server = sample_server();
        let runtime = match &mut server.runtime {
            ServerRuntimeConfig::DockerItzg(runtime) => runtime,
            ServerRuntimeConfig::Local(_) => panic!("expected docker runtime"),
        };
        runtime
            .env
            .insert("JVM_OPTS".to_string(), "-Dmanual=true".to_string());
        runtime
            .env
            .insert("JVM_XX_OPTS".to_string(), "-XX:ActiveProcessorCount=99".to_string());

        let options = ComposeGenerateOptions {
            output: None,
            full_stack: false,
            sealantern_image: DEFAULT_SEALANTERN_IMAGE.to_string(),
            sealantern_http_port: DEFAULT_SEALANTERN_HTTP_PORT,
            static_dir: None,
            sealantern_data_dir: None,
            docker_socket: DEFAULT_DOCKER_SOCKET.to_string(),
        };

        let runtime = server.docker_itzg_runtime().unwrap();
        let yaml = build_compose_yaml(&server, runtime, &options).unwrap();

        assert!(yaml.contains("JVM_OPTS: \"-Dmanual=true\""));
        assert!(yaml.contains("JVM_XX_OPTS: \"-XX:ActiveProcessorCount=99\""));
        assert!(!yaml.contains("-XX:ActiveProcessorCount=2\""));
    }

    #[test]
    fn build_compose_yaml_supports_full_stack_template() {
        let server = sample_server();
        let runtime = server.docker_itzg_runtime().unwrap();
        let options = ComposeGenerateOptions {
            output: None,
            full_stack: true,
            sealantern_image: DEFAULT_SEALANTERN_IMAGE.to_string(),
            sealantern_http_port: 3000,
            static_dir: Some("/app/dist".to_string()),
            sealantern_data_dir: Some("/app/data/servers".to_string()),
            docker_socket: DEFAULT_DOCKER_SOCKET.to_string(),
        };

        let yaml = build_compose_yaml(&server, runtime, &options).unwrap();
        assert!(yaml.contains("  sealantern:"));
        assert!(yaml.contains("SEALANTERN_HEADLESS_HTTP: \"1\""));
        assert!(yaml.contains("SEALANTERN_HTTP_BIND: \"127.0.0.1:3000\""));
        assert!(yaml.contains("SEALANTERN_SERVERS_CONTAINER_ROOT: \"/app/data/servers\""));
        assert!(yaml.contains("SEALANTERN_SERVERS_HOST_ROOT: \"E:/docker/paper\""));
        assert!(yaml.contains("STATIC_DIR: \"/app/dist\""));
        assert!(yaml.contains("- \"3000:3000/tcp\""));
        assert!(yaml.contains("depends_on:"));
        assert!(yaml.contains("- sealantern"));
    }

    #[test]
    fn full_stack_compose_template_marks_external_bind_as_explicit_opt_in() {
        let server = sample_server();
        let runtime = server.docker_itzg_runtime().unwrap();
        let options = ComposeGenerateOptions {
            output: None,
            full_stack: true,
            sealantern_image: DEFAULT_SEALANTERN_IMAGE.to_string(),
            sealantern_http_port: 3000,
            static_dir: None,
            sealantern_data_dir: Some("/app/data/servers".to_string()),
            docker_socket: DEFAULT_DOCKER_SOCKET.to_string(),
        };

        let yaml = build_compose_yaml(&server, runtime, &options).unwrap();
        assert!(yaml.contains("默认行为是仅监听 127.0.0.1"));
        assert!(yaml.contains("SEALANTERN_HTTP_BIND: \"127.0.0.1:3000\""));
        assert!(yaml.contains("才显式改成 0.0.0.0:3000"));
        assert!(!yaml.contains("SEALANTERN_WEB_BIND: \"0.0.0.0\""));
        assert!(!yaml.contains("默认行为是仅监听 0.0.0.0"));
    }

    #[test]
    fn sanitize_compose_service_name_normalizes_symbols() {
        assert_eq!(sanitize_compose_service_name("Sea Lantern Docker"), "sea-lantern-docker");
    }
}
