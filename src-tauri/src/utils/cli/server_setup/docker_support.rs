use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::models::server::{
    CreateDockerItzgServerRequest, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
    PublishedPort, VolumeMount,
};
use crate::services::server::installer::CoreType;
use crate::utils::cli::cli_env::{
    configured_servers_container_root, has_docker_host_path_mapping, is_container_like_environment,
};
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_docker::{
    build_docker_command_transport, default_docker_env, sanitize_name_like,
};
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_shared::{trace_cli_action, trace_cli_error};
use crate::utils::docker_cli::{
    docker_executable_path, format_docker_image_reference,
    inspect_docker_image_reference_with_soft_failures, resolve_docker_image_and_tag,
    DockerImageAvailability, DockerImageInspectOutcome,
};
use crate::utils::logger;
use crate::utils::path::strip_path_prefix_for_compare;

use super::metadata_support::{
    infer_core_type_from_local_inputs, infer_mc_version_from_folder, infer_mc_version_hint,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct DockerCreateDefaults {
    pub default_max_memory_mb: u32,
    pub default_min_memory_mb: u32,
}

const DEFAULT_DOCKER_IMAGE: &str = "itzg/minecraft-server";
pub(super) const DEFAULT_DOCKER_IMAGE_TAG: &str = "latest";

fn docker_itzg_image_looks_compatible(image: &str) -> bool {
    let normalized = image.trim().trim_matches('/').to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }

    normalized == "minecraft-server" || normalized.ends_with("/minecraft-server")
}

pub(super) fn validate_docker_itzg_image_compatibility(image: &str) -> Result<(), String> {
    if docker_itzg_image_looks_compatible(image) {
        return Ok(());
    }

    Err(format!(
        "当前 docker runtime 目标是 Minecraft server 容器，但镜像名看起来不兼容: {}。请使用 itzg/minecraft-server 或你自己的 */minecraft-server 镜像名；如果这是私有镜像/镜像代理，也请保持最终镜像名仍为 minecraft-server",
        image.trim()
    ))
}

pub(super) fn ensure_docker_environment() -> Result<(), String> {
    let docker_path = docker_executable_path()?;
    let output = std::process::Command::new(docker_path)
        .arg("info")
        .output()
        .map_err(|e| format!("执行 docker info 失败: {}", e))?;
    if output.status.success() {
        trace_cli_action("docker_environment_ready", "docker info succeeded");
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let error = if stderr.is_empty() {
        "docker 环境不可用，请确认 Docker Desktop 或 Docker Engine 已启动".to_string()
    } else {
        format!("docker 环境不可用: {}", stderr)
    };
    trace_cli_error("docker_environment_unavailable", "docker info", &error);
    Err(error)
}

pub(super) fn preflight_docker_image_reference(image: &str, image_tag: &str) -> Result<(), String> {
    validate_docker_itzg_image_compatibility(image)?;

    let image_ref = format_docker_image_reference(image, image_tag);
    match inspect_docker_image_reference_with_soft_failures(&image_ref)? {
        DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached) => {
            trace_cli_action(
                "docker_image_preflight",
                &format!("image_ref={} availability=local_cached", image_ref),
            );
            Ok(())
        }
        DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable) => {
            trace_cli_action(
                "docker_image_preflight",
                &format!("image_ref={} availability=remote_resolvable", image_ref),
            );
            Ok(())
        }
        DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
            trace_cli_action(
                "docker_image_preflight_soft_failure",
                &format!("image_ref={} failure_kind={:?}", image_ref, failure_kind),
            );
            logger::log_warn(&format!(
                "Docker 镜像预检已降级为软失败，将继续创建服务器记录: image_ref={} failure_kind={:?} detail={}",
                image_ref, failure_kind, message
            ));
            println!(
                "Docker 镜像预检未确认远端可用性，但已继续创建；若本地已缓存该镜像可直接忽略，否则待网络恢复后再执行 `sealantern docker pull {}`。",
                image_ref,
            );
            Ok(())
        }
    }
}

pub(super) fn preflight_docker_command_mode_support(
    image: &str,
    image_tag: &str,
    command_mode: &DockerCommandMode,
) -> Result<(), String> {
    if *command_mode != DockerCommandMode::DockerStdio {
        return Ok(());
    }

    let image_ref = format_docker_image_reference(image, image_tag);
    match inspect_docker_image_reference_with_soft_failures(&image_ref)? {
        DockerImageInspectOutcome::Available(DockerImageAvailability::LocalCached) => {
            ensure_docker_stdio_image_support(&image_ref)
        }
        DockerImageInspectOutcome::Available(DockerImageAvailability::RemoteResolvable) => {
            trace_cli_action(
                "docker_stdio_preflight_remote_only",
                &format!("image_ref={}", image_ref),
            );
            logger::log_warn(&format!(
                "docker_stdio 预检跳过镜像内部探测: image_ref={} availability=remote_resolvable",
                image_ref
            ));
            Ok(())
        }
        DockerImageInspectOutcome::SoftFailure { failure_kind, message } => {
            trace_cli_action(
                "docker_stdio_preflight_soft_failure",
                &format!("image_ref={} failure_kind={:?}", image_ref, failure_kind),
            );
            logger::log_warn(&format!(
                "docker_stdio 预检未确认镜像缓存状态，将跳过内部命令探测: image_ref={} failure_kind={:?} detail={}",
                image_ref, failure_kind, message
            ));
            Ok(())
        }
    }
}

fn ensure_docker_stdio_image_support(image_ref: &str) -> Result<(), String> {
    let docker_path = docker_executable_path()?;
    let output = std::process::Command::new(docker_path)
        .arg("run")
        .arg("--rm")
        .arg("--entrypoint")
        .arg("sh")
        .arg(image_ref)
        .arg("-lc")
        .arg("command -v mc-send-to-console >/dev/null 2>&1")
        .output()
        .map_err(|err| format!("执行 docker run 检查 docker_stdio 镜像能力失败: {}", err))?;

    if output.status.success() {
        trace_cli_action("docker_stdio_preflight_supported", &format!("image_ref={}", image_ref));
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let raw = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("退出码: {:?}", output.status.code())
    };

    Err(format!(
        "当前镜像不支持 --command-mode docker_stdio: image={} 未检测到 mc-send-to-console。请改用 --command-mode rcon，或改用兼容 itzg 语义且内置 mc-send-to-console 的镜像。原始输出: {}",
        image_ref, raw
    ))
}

pub(super) fn resolve_requested_docker_image(
    command: &CliServerCommand,
) -> Result<(String, String), String> {
    resolve_docker_image_and_tag(
        command.image.as_deref(),
        command.image_tag.as_deref(),
        DEFAULT_DOCKER_IMAGE,
        DEFAULT_DOCKER_IMAGE_TAG,
    )
}

pub(super) fn build_docker_create_request(
    command: &CliServerCommand,
    resolved_name: &str,
    ports: &PreparedPorts,
    defaults: DockerCreateDefaults,
) -> Result<CreateDockerItzgServerRequest, String> {
    let docker_folder_path = command.folder.as_deref().map(Path::new);
    let mc_version = command
        .mc_version
        .clone()
        .or_else(|| {
            docker_folder_path.and_then(|folder| infer_mc_version_from_folder(folder, None))
        })
        .or_else(|| {
            command
                .folder
                .as_deref()
                .and_then(|folder| infer_mc_version_hint(&[folder]))
        })
        .or_else(|| infer_mc_version_hint(&[resolved_name]))
        .ok_or_else(|| "docker server 缺少 --mc".to_string())?;
    let core_type = command
        .core_type
        .clone()
        .or_else(|| {
            docker_folder_path.and_then(|folder| infer_core_type_from_local_inputs(folder, None))
        })
        .map(|value| normalize_core_type(Some(&value)))
        .transpose()?
        .unwrap_or_else(|| "paper".to_string());
    let api_core = CoreType::normalize_to_api_core_key(&core_type).unwrap_or(core_type.clone());
    let data_dir = resolve_docker_data_dir(command, resolved_name)?;
    let (image, image_tag) = resolve_requested_docker_image(command)?;
    let container_name = command
        .container_name
        .clone()
        .unwrap_or_else(|| format!("sealantern-{}", sanitize_container_name(resolved_name)));
    let docker_backend_kind = parse_docker_backend(command.docker_backend.as_deref())?;
    let command_mode = parse_command_mode(command.command_mode.as_deref())?;
    let mut env = default_docker_env();
    apply_docker_memory_env(command, &mut env, defaults);
    apply_custom_docker_env(command, &mut env);
    let volume_mounts = parse_docker_volume_mounts(&command.docker_mounts)?;
    let mut reserved_ports = vec![ports.game_port];
    if let Some(web_port) = ports.web_port {
        reserved_ports.push(web_port);
    }
    let (mut extra_ports, rcon) =
        build_docker_command_transport(&command_mode, &container_name, &reserved_ports, &mut env)?;
    reserved_ports.extend(extra_ports.iter().map(|port| port.host_port));
    let published_ports = parse_extra_published_ports(&command.docker_publishes, &reserved_ports)?;
    extra_ports.extend(published_ports);

    trace_cli_action(
        "docker_request_built",
        &format!(
            "name={} image={}:{} container={} backend={} command_mode={} data_dir={} game_port= {}",
            resolved_name,
            image,
            image_tag,
            container_name,
            docker_backend_kind.as_str(),
            command_mode.as_str(),
            data_dir,
            ports.game_port
        ),
    );

    Ok(CreateDockerItzgServerRequest {
        name: resolved_name.to_string(),
        aliases: command.aliases.clone(),
        core_type,
        mc_version: mc_version.clone(),
        port: ports.game_port,
        runtime: DockerItzgRuntimeConfig {
            image,
            image_tag,
            container_name,
            type_value: api_core.to_ascii_uppercase().replace('-', "_"),
            version: mc_version,
            data_dir_mount: data_dir,
            published_game_port: ports.game_port,
            env,
            extra_ports,
            volume_mounts,
            docker_backend_kind,
            command_mode,
            rcon,
        },
    })
}

pub(super) fn parse_docker_backend(value: Option<&str>) -> Result<DockerBackendKind, String> {
    match value.unwrap_or("cli").trim().to_ascii_lowercase().as_str() {
        "cli" => Ok(DockerBackendKind::Cli),
        "engine_api" | "engine-api" => Ok(DockerBackendKind::EngineApi),
        other => Err(format!("不支持的 docker backend: {}", other)),
    }
}

pub(super) fn parse_command_mode(value: Option<&str>) -> Result<DockerCommandMode, String> {
    match value.unwrap_or("rcon").trim().to_ascii_lowercase().as_str() {
        "rcon" => Ok(DockerCommandMode::Rcon),
        "docker_stdio" | "docker-stdio" | "stdio" => Ok(DockerCommandMode::DockerStdio),
        other => Err(format!("不支持的 docker command mode: {}", other)),
    }
}

pub(super) fn format_memory_env_value(memory_mb: u32) -> String {
    if memory_mb.is_multiple_of(1024) {
        format!("{}G", memory_mb / 1024)
    } else {
        format!("{}M", memory_mb)
    }
}

pub(super) fn resolve_docker_data_dir(
    command: &CliServerCommand,
    resolved_name: &str,
) -> Result<String, String> {
    if let Some(explicit_data_dir) = command.data_dir.clone().or_else(|| command.folder.clone()) {
        return Ok(explicit_data_dir);
    }

    if is_container_like_environment() && !has_docker_host_path_mapping() {
        return Err(
            "当前 Sea Lantern 运行在容器可见路径下，且未配置 SEALANTERN_SERVERS_CONTAINER_ROOT / SEALANTERN_SERVERS_HOST_ROOT；请显式传入 --data-dir，或配置宿主路径映射"
                .to_string(),
        );
    }

    Ok(default_docker_server_dir(resolved_name))
}

pub(super) fn map_container_visible_path_to_docker_host_path(path: &Path) -> Option<String> {
    let host_root = std::env::var("SEALANTERN_SERVERS_HOST_ROOT").ok()?;
    let container_root = std::env::var("SEALANTERN_SERVERS_CONTAINER_ROOT").ok()?;

    let host_root = PathBuf::from(host_root);
    let container_root = PathBuf::from(container_root);
    let relative = strip_path_prefix_for_compare(path, &container_root)?;

    if relative.is_empty() {
        return Some(host_root.to_string_lossy().to_string());
    }

    Some(host_root.join(relative).to_string_lossy().to_string())
}

fn apply_docker_memory_env(
    command: &CliServerCommand,
    env: &mut BTreeMap<String, String>,
    defaults: DockerCreateDefaults,
) {
    let max_memory_mb = command
        .max_memory_mb
        .unwrap_or(defaults.default_max_memory_mb);
    let min_memory_mb = command
        .min_memory_mb
        .unwrap_or(defaults.default_min_memory_mb);

    env.insert("MEMORY".to_string(), format_memory_env_value(max_memory_mb));
    env.insert("MAX_MEMORY".to_string(), format_memory_env_value(max_memory_mb));
    env.insert("INIT_MEMORY".to_string(), format_memory_env_value(min_memory_mb));
}

fn apply_custom_docker_env(command: &CliServerCommand, env: &mut BTreeMap<String, String>) {
    for (key, value) in &command.docker_env {
        env.insert(key.clone(), value.clone());
    }
}

fn parse_docker_volume_mounts(values: &[String]) -> Result<Vec<VolumeMount>, String> {
    values
        .iter()
        .map(|value| parse_docker_volume_mount(value))
        .collect()
}

fn parse_docker_volume_mount(value: &str) -> Result<VolumeMount, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("--mount 不能为空".to_string());
    }

    let (without_mode, read_only) = if let Some(prefix) = trimmed.strip_suffix(":ro") {
        (prefix, true)
    } else if let Some(prefix) = trimmed.strip_suffix(":rw") {
        (prefix, false)
    } else {
        (trimmed, false)
    };

    let (source, target) = split_mount_source_target(without_mode, value)?;
    if source.trim().is_empty() || target.trim().is_empty() {
        return Err(format!("--mount 需要非空 source 与 target: {}", value));
    }

    Ok(VolumeMount {
        source: source.trim().to_string(),
        target: target.trim().to_string(),
        read_only,
    })
}

fn parse_extra_published_ports(
    values: &[String],
    reserved_host_ports: &[u16],
) -> Result<Vec<PublishedPort>, String> {
    let mut ports = Vec::with_capacity(values.len());
    for value in values {
        let port = parse_published_port(value)?;
        if reserved_host_ports.contains(&port.host_port) {
            return Err(format!(
                "--publish 宿主端口 {} 与当前已保留端口冲突，请改用其他宿主端口: {}",
                port.host_port, value
            ));
        }
        if ports
            .iter()
            .any(|existing: &PublishedPort| existing.host_port == port.host_port)
        {
            return Err(format!(
                "--publish 宿主端口 {} 重复定义，请检查参数: {}",
                port.host_port, value
            ));
        }
        ports.push(port);
    }
    Ok(ports)
}

fn parse_published_port(value: &str) -> Result<PublishedPort, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("--publish 不能为空".to_string());
    }

    let (port_pair, protocol) = if let Some((pair, protocol)) = trimmed.rsplit_once('/') {
        let protocol = protocol.trim().to_ascii_lowercase();
        if !matches!(protocol.as_str(), "tcp" | "udp") {
            return Err(format!("--publish 仅支持 tcp/udp 协议: {}", value));
        }
        (pair, protocol)
    } else {
        (trimmed, "tcp".to_string())
    };

    let Some((host_port, container_port)) = port_pair.split_once(':') else {
        return Err(format!("--publish 需要 host:container[/tcp|udp] 形式: {}", value));
    };

    Ok(PublishedPort {
        host_port: parse_non_zero_port(host_port.trim(), "--publish host")?,
        container_port: parse_non_zero_port(container_port.trim(), "--publish container")?,
        protocol,
    })
}

fn parse_non_zero_port(value: &str, label: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| format!("{} 需要有效的非零端口号: {}", label, value))
}

fn split_mount_source_target(value: &str, original: &str) -> Result<(String, String), String> {
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    for (position, ch) in chars.iter().rev() {
        if *ch != ':' {
            continue;
        }

        let source = &value[..*position];
        let target = &value[*position + 1..];
        if target.starts_with('/') || target.starts_with("./") || target.starts_with("../") {
            return Ok((source.to_string(), target.to_string()));
        }
    }

    Err(format!("--mount 需要 source:target[:ro|rw] 形式: {}", original))
}

fn default_docker_server_dir(name: &str) -> String {
    let path = default_container_visible_server_dir(name);
    map_container_visible_path_to_docker_host_path(&path)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn default_container_visible_server_dir(name: &str) -> PathBuf {
    let mut path = configured_servers_container_root().unwrap_or_else(|| {
        let mut app_data_dir = PathBuf::from(crate::utils::path::get_or_create_app_data_dir());
        app_data_dir.push("servers");
        app_data_dir
    });
    path.push(sanitize_container_name(name));
    path
}

fn sanitize_container_name(name: &str) -> String {
    sanitize_name_like(name)
}

fn normalize_core_type(value: Option<&str>) -> Result<String, String> {
    let raw = value.unwrap_or("paper").trim();
    if raw.is_empty() {
        return Err("--core 不能为空".to_string());
    }
    Ok(CoreType::normalize_to_api_core_key(raw).unwrap_or_else(|| raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        build_docker_create_request, format_memory_env_value,
        map_container_visible_path_to_docker_host_path, parse_command_mode, parse_docker_backend,
        parse_docker_volume_mount, parse_published_port, resolve_docker_data_dir,
        resolve_requested_docker_image, validate_docker_itzg_image_compatibility,
        DockerCreateDefaults, DEFAULT_DOCKER_IMAGE_TAG,
    };
    use crate::models::server::{DockerBackendKind, DockerCommandMode};
    use crate::utils::cli::server_args::CliServerCommand;
    use crate::utils::cli::server_ports::PreparedPorts;
    use once_cell::sync::Lazy;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    struct EnvVarGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    fn sample_defaults() -> DockerCreateDefaults {
        DockerCreateDefaults {
            default_max_memory_mb: 4096,
            default_min_memory_mb: 2048,
        }
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let original = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, original }
        }

        fn remove(key: &'static str) -> Self {
            let original = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn parse_server_command_defaults_docker_command_mode_to_rcon() {
        let command = CliServerCommand::default();
        assert_eq!(
            parse_command_mode(command.command_mode.as_deref()).unwrap(),
            DockerCommandMode::Rcon
        );
    }

    #[test]
    fn parse_docker_backend_accepts_engine_api_aliases() {
        assert_eq!(parse_docker_backend(Some("engine_api")).unwrap(), DockerBackendKind::EngineApi);
        assert_eq!(parse_docker_backend(Some("engine-api")).unwrap(), DockerBackendKind::EngineApi);
    }

    #[test]
    fn parse_docker_backend_rejects_unknown_value() {
        let err = parse_docker_backend(Some("podman")).expect_err("unknown backend should fail");
        assert!(err.contains("docker backend"));
        assert!(err.contains("podman"));
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
    fn build_docker_create_request_defaults_to_rcon_transport() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-prod".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("docker request should build");

        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert!(request.runtime.rcon.is_some());
        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("2G"));
        assert_eq!(request.runtime.env.get("ENABLE_RCON").map(String::as_str), Some("true"));
        assert_eq!(request.runtime.env.get("RCON_PORT").map(String::as_str), Some("25575"));
        assert!(request.runtime.env.get("RCON_PASSWORD").is_some());
        assert_eq!(request.runtime.extra_ports.len(), 1);
        assert_eq!(request.runtime.extra_ports[0].container_port, 25575);
    }

    #[test]
    fn build_docker_create_request_avoids_game_and_web_ports_for_default_rcon_port() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-rcon-avoid".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25575, web_port: Some(25576) };

        let request =
            build_docker_create_request(&command, "paper-rcon-avoid", &ports, sample_defaults())
                .expect("docker request should avoid reserved game/web ports for rcon");

        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert_eq!(request.runtime.extra_ports.len(), 1);
        assert!(request.runtime.extra_ports[0].host_port >= 25577);
        assert_eq!(
            request.runtime.rcon.as_ref().map(|rcon| rcon.port),
            Some(request.runtime.extra_ports[0].host_port)
        );
    }

    #[test]
    fn build_docker_create_request_can_infer_core_and_mc_from_folder_name() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");

        let command = CliServerCommand {
            runtime: Some("docker".to_string()),
            folder: Some(folder.to_string_lossy().to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("docker request should infer metadata from folder");

        assert_eq!(request.core_type, "paper");
        assert_eq!(request.mc_version, "1.21.1");
        assert_eq!(request.runtime.version, "1.21.1");
        assert_eq!(request.runtime.data_dir_mount, folder.to_string_lossy().to_string());
    }

    #[test]
    fn build_docker_create_request_prefers_explicit_data_dir_over_folder_mount_source() {
        let command = CliServerCommand {
            runtime: Some("docker".to_string()),
            folder: Some("E:/servers/paper-prod-1.21.1".to_string()),
            data_dir: Some("E:/docker/explicit".to_string()),
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("explicit data dir should still win");

        assert_eq!(request.runtime.data_dir_mount, "E:/docker/explicit");
    }

    #[test]
    fn build_docker_create_request_builds_rcon_ready_runtime_shape() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("itzg/minecraft-server".to_string()),
            image_tag: Some("java21".to_string()),
            data_dir: Some("E:/docker/paper".to_string()),
            container_name: Some("Sea Lantern Docker".to_string()),
            command_mode: Some("rcon".to_string()),
            aliases: vec!["paper_prod".to_string()],
            min_memory_mb: Some(2048),
            max_memory_mb: Some(4096),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-prod", &ports, sample_defaults())
                .expect("docker request should build");

        assert_eq!(request.runtime.image_tag, "java21");
        assert_eq!(request.runtime.data_dir_mount, "E:/docker/paper");
        assert_eq!(request.runtime.command_mode, DockerCommandMode::Rcon);
        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("2G"));
        assert!(request.runtime.rcon.is_some());
    }

    #[test]
    fn build_docker_create_request_skips_rcon_side_effects_for_stdio_mode() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            command_mode: Some("docker_stdio".to_string()),
            data_dir: Some("E:/docker/paper-stdio".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-stdio", &ports, sample_defaults())
                .expect("docker stdio request should build");

        assert_eq!(request.runtime.command_mode, DockerCommandMode::DockerStdio);
        assert!(request.runtime.extra_ports.is_empty());
        assert!(request.runtime.rcon.is_none());
        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("4G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("2G"));
        assert!(request.runtime.env.get("ENABLE_RCON").is_none());
        assert!(request.runtime.env.get("RCON_PASSWORD").is_none());
        assert_eq!(
            request
                .runtime
                .env
                .get("CREATE_CONSOLE_IN_PIPE")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn build_docker_create_request_applies_custom_env_and_mounts() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-custom".to_string()),
            docker_env: vec![
                ("STOP_DURATION".to_string(), "180".to_string()),
                ("DISABLE_HEALTHCHECK".to_string(), "true".to_string()),
            ],
            docker_mounts: vec!["E:/plugins:/data/plugins:ro".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-custom", &ports, sample_defaults())
                .expect("custom docker request should build");

        assert_eq!(request.runtime.env.get("STOP_DURATION").map(String::as_str), Some("180"));
        assert_eq!(
            request
                .runtime
                .env
                .get("DISABLE_HEALTHCHECK")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(request.runtime.volume_mounts.len(), 1);
        assert_eq!(request.runtime.volume_mounts[0].source, "E:/plugins");
        assert_eq!(request.runtime.volume_mounts[0].target, "/data/plugins");
        assert!(request.runtime.volume_mounts[0].read_only);
    }

    #[test]
    fn build_docker_create_request_supports_extra_publish_ports() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-publish".to_string()),
            docker_publishes: vec!["24454:24454/udp".to_string(), "8123:8123".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-publish", &ports, sample_defaults())
                .expect("custom publish ports should build");

        assert!(request.runtime.extra_ports.iter().any(|port| {
            port.host_port == 24454 && port.container_port == 24454 && port.protocol == "udp"
        }));
        assert!(request.runtime.extra_ports.iter().any(|port| {
            port.host_port == 8123 && port.container_port == 8123 && port.protocol == "tcp"
        }));
    }

    #[test]
    fn build_docker_create_request_rejects_publish_port_conflicting_with_game_port() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-publish-conflict".to_string()),
            docker_publishes: vec!["25565:24454/udp".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err = build_docker_create_request(
            &command,
            "paper-publish-conflict",
            &ports,
            sample_defaults(),
        )
        .expect_err("game port conflict should fail");

        assert!(err.contains("宿主端口 25565"));
        assert!(err.contains("冲突"));
    }

    #[test]
    fn build_docker_create_request_rejects_publish_port_conflicting_with_rcon_port() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-publish-rcon-conflict".to_string()),
            docker_publishes: vec!["25575:24454/tcp".to_string()],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err = build_docker_create_request(
            &command,
            "paper-publish-rcon-conflict",
            &ports,
            sample_defaults(),
        )
        .expect_err("rcon port conflict should fail");

        assert!(err.contains("宿主端口 25575"));
    }

    #[test]
    fn build_docker_create_request_rejects_rcon_password_file_without_explicit_password() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-rcon-secret".to_string()),
            docker_env: vec![(
                "RCON_PASSWORD_FILE".to_string(),
                "/run/secrets/rcon_pass".to_string(),
            )],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let err =
            build_docker_create_request(&command, "paper-rcon-secret", &ports, sample_defaults())
                .expect_err("rcon password file without explicit password should fail");

        assert!(err.contains("RCON_PASSWORD_FILE"));
        assert!(err.contains("RCON_PASSWORD"));
    }

    #[test]
    fn build_docker_create_request_respects_explicit_rcon_env_values() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/paper-rcon-custom".to_string()),
            docker_env: vec![
                ("RCON_PORT".to_string(), "28016".to_string()),
                ("RCON_PASSWORD".to_string(), "top-secret".to_string()),
            ],
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-rcon-custom", &ports, sample_defaults())
                .expect("custom rcon env should build");

        assert_eq!(request.runtime.extra_ports[0].container_port, 28016);
        assert_eq!(request.runtime.env.get("RCON_PORT").map(String::as_str), Some("28016"));
        assert_eq!(
            request
                .runtime
                .rcon
                .as_ref()
                .map(|rcon| rcon.password.as_str()),
            Some("top-secret")
        );
    }

    #[test]
    fn parse_docker_volume_mount_supports_windows_host_paths() {
        let mount = parse_docker_volume_mount("E:/plugins:/data/plugins:ro")
            .expect("windows-style docker mount should parse");

        assert_eq!(mount.source, "E:/plugins");
        assert_eq!(mount.target, "/data/plugins");
        assert!(mount.read_only);
    }

    #[test]
    fn parse_published_port_supports_tcp_and_udp() {
        let udp = parse_published_port("24454:24454/udp").expect("udp publish should parse");
        let tcp = parse_published_port("8123:8123").expect("tcp publish should parse");

        assert_eq!(udp.host_port, 24454);
        assert_eq!(udp.container_port, 24454);
        assert_eq!(udp.protocol, "udp");
        assert_eq!(tcp.protocol, "tcp");
    }

    #[test]
    fn resolve_requested_docker_image_supports_embedded_tag() {
        let command = CliServerCommand {
            image: Some("itzg/minecraft-server:java21".to_string()),
            ..Default::default()
        };

        let (image, tag) =
            resolve_requested_docker_image(&command).expect("embedded tag should resolve");

        assert_eq!(image, "itzg/minecraft-server");
        assert_eq!(tag, "java21");
    }

    #[test]
    fn build_docker_create_request_supports_full_image_reference_without_separate_image_tag() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            image: Some("registry.local:5000/itzg/minecraft-server:java21".to_string()),
            data_dir: Some("E:/docker/full-image-ref".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request =
            build_docker_create_request(&command, "paper-full-image", &ports, sample_defaults())
                .expect("full image ref should resolve for docker request");

        assert_eq!(request.runtime.image, "registry.local:5000/itzg/minecraft-server");
        assert_eq!(request.runtime.image_tag, "java21");
    }

    #[test]
    fn validate_docker_itzg_image_compatibility_accepts_default_and_private_mirror_shapes() {
        validate_docker_itzg_image_compatibility("itzg/minecraft-server")
            .expect("official image should be accepted");
        validate_docker_itzg_image_compatibility("registry.local:5000/itzg/minecraft-server")
            .expect("private mirror with minecraft-server suffix should be accepted");
        validate_docker_itzg_image_compatibility("minecraft-server")
            .expect("bare minecraft-server image name should be accepted");
    }

    #[test]
    fn validate_docker_itzg_image_compatibility_rejects_non_minecraft_runtime_images() {
        let err = validate_docker_itzg_image_compatibility("naloveyuki/liteyukibot-web")
            .expect_err("non minecraft runtime image should be rejected");

        assert!(err.contains("liteyukibot-web"));
        assert!(err.contains("minecraft-server"));
    }

    #[test]
    fn resolve_docker_data_dir_prefers_explicit_data_dir() {
        let command = CliServerCommand {
            data_dir: Some("E:/docker/explicit".to_string()),
            ..Default::default()
        };

        let resolved = resolve_docker_data_dir(&command, "paper-docker")
            .expect("explicit data dir should win");
        assert_eq!(resolved, "E:/docker/explicit");
    }

    #[test]
    fn resolve_docker_data_dir_errors_for_container_like_default_without_mapping() {
        let _env_lock = ENV_LOCK.lock().expect("env lock should acquire");
        let _headless_guard = EnvVarGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _host_guard = EnvVarGuard::remove("SEALANTERN_SERVERS_HOST_ROOT");
        let _container_guard = EnvVarGuard::remove("SEALANTERN_SERVERS_CONTAINER_ROOT");

        let command = CliServerCommand::default();
        let err = resolve_docker_data_dir(&command, "paper-docker")
            .expect_err("container-like env without mapping should fail fast");

        assert!(err.contains("SEALANTERN_SERVERS_CONTAINER_ROOT"));
        assert!(err.contains("--data-dir"));
    }

    #[test]
    fn resolve_docker_data_dir_uses_mapped_default_when_container_mapping_exists() {
        let _env_lock = ENV_LOCK.lock().expect("env lock should acquire");
        let _headless_guard = EnvVarGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _host_guard =
            EnvVarGuard::set("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        let _container_guard =
            EnvVarGuard::set("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");

        let command = CliServerCommand::default();
        let resolved = resolve_docker_data_dir(&command, "paper-docker")
            .expect("mapped container env should resolve host path");

        assert!(resolved.replace('\\', "/").ends_with("/paper-docker"));
        assert!(resolved
            .replace('\\', "/")
            .starts_with("E:/srv/sealantern/servers/"));
    }

    #[test]
    fn default_docker_image_tag_matches_latest() {
        assert_eq!(DEFAULT_DOCKER_IMAGE_TAG, "latest");
    }

    #[test]
    fn map_container_visible_path_to_docker_host_path_translates_when_roots_match() {
        std::env::set_var("SEALANTERN_SERVERS_HOST_ROOT", "E:/srv/sealantern/servers");
        std::env::set_var("SEALANTERN_SERVERS_CONTAINER_ROOT", "/app/data/servers");

        let mapped = map_container_visible_path_to_docker_host_path(Path::new(
            "/app/data/servers/paper-docker",
        ))
        .expect("path should translate when roots match");

        assert_eq!(mapped.replace('\\', "/"), "E:/srv/sealantern/servers/paper-docker");

        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");
    }

    #[test]
    fn map_container_visible_path_to_docker_host_path_returns_none_without_mapping() {
        std::env::remove_var("SEALANTERN_SERVERS_HOST_ROOT");
        std::env::remove_var("SEALANTERN_SERVERS_CONTAINER_ROOT");

        assert!(map_container_visible_path_to_docker_host_path(Path::new(
            "/app/data/servers/paper"
        ))
        .is_none());
    }

    #[test]
    fn format_memory_env_value_prefers_gigabytes_when_evenly_divisible() {
        assert_eq!(format_memory_env_value(2048), "2G");
        assert_eq!(format_memory_env_value(1536), "1536M");
    }

    #[test]
    fn build_docker_create_request_applies_default_memory_envs_when_cli_omits_memory_flags() {
        let command = CliServerCommand {
            mc_version: Some("1.21.1".to_string()),
            core_type: Some("paper".to_string()),
            data_dir: Some("E:/docker/default-memory".to_string()),
            ..Default::default()
        };
        let ports = PreparedPorts { game_port: 25565, web_port: None };

        let request = build_docker_create_request(
            &command,
            "paper-default-memory",
            &ports,
            DockerCreateDefaults {
                default_max_memory_mb: 3072,
                default_min_memory_mb: 1536,
            },
        )
        .expect("default docker memory envs should be applied");

        assert_eq!(request.runtime.env.get("MEMORY").map(String::as_str), Some("3G"));
        assert_eq!(request.runtime.env.get("MAX_MEMORY").map(String::as_str), Some("3G"));
        assert_eq!(request.runtime.env.get("INIT_MEMORY").map(String::as_str), Some("1536M"));
    }
}
