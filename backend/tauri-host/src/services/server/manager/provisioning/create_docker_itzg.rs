use crate::models::server::{CreateDockerItzgServerRequest, ServerInstance, ServerRuntimeConfig};
use sea_lantern_docker_core::parse_memory_env_value_checked;
use sea_lantern_server_config_core::startup::ensure_server_path_writable;

use super::super::common::{
    current_timestamp_secs, ensure_server_identity_available, validate_server_name,
};
use super::ServerManager;
use std::path::Path;

fn resolve_memory_env_mb(
    env: &std::collections::BTreeMap<String, String>,
    key: &str,
    default_mb: u32,
) -> Result<u32, String> {
    Ok(
        env.get(key)
            .map(|value| {
                parse_memory_env_value_checked(value)
                    .map_err(|err| format!("{} 配置无效: {}", key, err))
                    .map(|parsed| parsed.unwrap_or(default_mb))
            })
            .transpose()?
            .unwrap_or(default_mb),
    )
}

fn build_docker_itzg_server(
    id: String,
    now: u64,
    server_name: String,
    req: CreateDockerItzgServerRequest,
) -> Result<ServerInstance, String> {
    let data_dir_mount = req.runtime.data_dir_mount.trim().to_string();
    if data_dir_mount.is_empty() {
        return Err("docker_itzg data_dir_mount 不能为空".to_string());
    }

    let max_memory = resolve_memory_env_mb(&req.runtime.env, "MAX_MEMORY", 4096)?;
    let min_memory = resolve_memory_env_mb(&req.runtime.env, "INIT_MEMORY", 1024)?;
    let runtime = req.runtime;
    let server_dir = data_dir_mount;

    Ok(ServerInstance {
        id,
        name: server_name,
        aliases: req.aliases,
        core_type: req.core_type,
        core_version: String::new(),
        mc_version: req.mc_version,
        path: server_dir.clone(),
        port: req.port,
        max_memory,
        min_memory,
        created_at: now,
        last_started_at: None,
        runtime_kind: "docker_itzg".to_string(),
        runtime: ServerRuntimeConfig::DockerItzg(runtime),
    })
}

pub(super) fn create_docker_itzg_server(
    manager: &ServerManager,
    req: CreateDockerItzgServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = current_timestamp_secs();

    {
        let servers = manager.lock_servers()?;
        ensure_server_identity_available(
            &servers,
            &server_name,
            &req.aliases,
            &req.runtime.data_dir_mount,
            Some(&req.runtime.container_name),
        )?;
    }

    let server = build_docker_itzg_server(id, now, server_name, req)?;
    let server_path = Path::new(&server.path);
    std::fs::create_dir_all(server_path).map_err(|e| format!("无法创建 Docker 数据目录: {}", e))?;
    ensure_server_path_writable(server_path)?;

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    use super::build_docker_itzg_server;
    use sea_lantern_docker_core::{parse_memory_env_value, parse_memory_env_value_checked};
    use crate::models::server::{
        CpuPolicyConfig, CreateDockerItzgServerRequest, DockerBackendKind, DockerCommandMode,
        DockerItzgRuntimeConfig, JvmPresetConfig, ServerRuntimeConfig,
    };
    use crate::services::server::manager::ServerManager;

    fn isolated_manager(temp_dir: &std::path::Path) -> ServerManager {
        let manager = ServerManager::new();
        *manager.data_dir.lock().expect("data_dir lock should work") =
            temp_dir.to_string_lossy().to_string();
        manager
            .lock_servers()
            .expect("servers lock should work")
            .clear();
        manager
    }

    fn sample_request() -> CreateDockerItzgServerRequest {
        CreateDockerItzgServerRequest {
            name: "Docker Hidden".to_string(),
            aliases: vec!["docker_hidden".to_string()],
            core_type: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            port: 25565,
            runtime: DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "java21".to_string(),
                container_name: "sea-hidden-docker".to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: "E:/servers/docker-hidden".to_string(),
                published_game_port: 25565,
                env: BTreeMap::from([
                    ("MAX_MEMORY".to_string(), "4G".to_string()),
                    ("INIT_MEMORY".to_string(), "2G".to_string()),
                ]),
                extra_ports: Vec::new(),
                volume_mounts: Vec::new(),
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            },
        }
    }

    #[test]
    fn build_docker_itzg_server_sets_hidden_runtime_shape() {
        let server = build_docker_itzg_server(
            "docker-hidden-1".to_string(),
            123,
            "Docker Hidden".to_string(),
            sample_request(),
        )
        .expect("docker hidden server should build");

        assert_eq!(server.runtime_kind, "docker_itzg");
        assert_eq!(server.aliases, vec!["docker_hidden"]);
        assert_eq!(server.path, "E:/servers/docker-hidden");
        assert_eq!(server.port, 25565);
        assert_eq!(server.max_memory, 4096);
        assert_eq!(server.min_memory, 2048);

        match server.runtime {
            ServerRuntimeConfig::DockerItzg(runtime) => {
                assert_eq!(runtime.image, "itzg/minecraft-server");
                assert_eq!(runtime.image_tag, "java21");
                assert_eq!(runtime.container_name, "sea-hidden-docker");
                assert_eq!(runtime.docker_backend_kind, DockerBackendKind::Cli);
                assert_eq!(runtime.command_mode, DockerCommandMode::Rcon);
            }
            ServerRuntimeConfig::Local(_) => panic!("expected docker runtime"),
        }
    }

    #[test]
    fn build_docker_itzg_server_rejects_empty_data_dir_mount() {
        let mut req = sample_request();
        req.runtime.data_dir_mount = "   ".to_string();

        let err = build_docker_itzg_server(
            "docker-hidden-2".to_string(),
            123,
            "Docker Hidden".to_string(),
            req,
        )
        .expect_err("empty data dir mount should be rejected");

        assert!(err.contains("data_dir_mount"));
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
    fn build_docker_itzg_server_rejects_invalid_max_memory_env() {
        let mut req = sample_request();
        req.runtime
            .env
            .insert("MAX_MEMORY".to_string(), "4X".to_string());

        let err = build_docker_itzg_server(
            "docker-hidden-3".to_string(),
            123,
            "Docker Hidden".to_string(),
            req,
        )
        .expect_err("invalid MAX_MEMORY should be rejected");

        assert!(err.contains("MAX_MEMORY 配置无效"));
        assert!(err.contains("内存值无效 '4X'"));
    }

    #[test]
    fn build_docker_itzg_server_treats_blank_init_memory_as_absent() {
        let mut req = sample_request();
        req.runtime
            .env
            .insert("INIT_MEMORY".to_string(), "   ".to_string());

        let server = build_docker_itzg_server(
            "docker-hidden-4".to_string(),
            123,
            "Docker Hidden".to_string(),
            req,
        )
        .expect("blank INIT_MEMORY should fall back to default");

        assert_eq!(server.min_memory, 1024);
    }

    #[test]
    fn create_docker_itzg_server_creates_missing_data_dir_before_persisting() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let data_dir = temp_dir.path().join("docker-data").join("paper-prod");

        let manager = isolated_manager(temp_dir.path());
        let mut req = sample_request();
        req.name = "Docker Hidden Temp Create".to_string();
        req.aliases = vec!["docker_hidden_temp_create".to_string()];
        req.runtime.container_name = "sea-hidden-docker-temp-create".to_string();
        req.runtime.data_dir_mount = data_dir.to_string_lossy().to_string();

        let server = super::create_docker_itzg_server(&manager, req)
            .expect("docker server should be created with data dir");

        assert_eq!(server.path, data_dir.to_string_lossy().to_string());
        assert!(data_dir.exists());
        assert!(data_dir.is_dir());
    }

    #[test]
    fn create_docker_itzg_server_rejects_file_backed_data_dir_mount() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let data_file = temp_dir.path().join("docker-data-file");
        fs::write(&data_file, b"not a directory").expect("data file should exist");

        let manager = isolated_manager(temp_dir.path());
        let mut req = sample_request();
        req.name = "Docker Hidden Temp File".to_string();
        req.aliases = vec!["docker_hidden_temp_file".to_string()];
        req.runtime.container_name = "sea-hidden-docker-temp-file".to_string();
        req.runtime.data_dir_mount = data_file.to_string_lossy().to_string();

        let err = super::create_docker_itzg_server(&manager, req)
            .expect_err("file-backed data dir should fail");

        assert!(
            err.contains("无法创建 Docker 数据目录")
                || err.contains("无法写入服务器目录")
                || err.contains("权限")
        );
    }
}
