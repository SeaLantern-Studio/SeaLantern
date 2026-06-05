use std::path::Path;

use crate::models::server::{
    CreateServerRequest, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};

use super::super::common::{
    current_timestamp_secs, ensure_server_identity_available, normalize_startup_mode,
    validate_server_name,
};
use super::shared::{create_server_properties_if_missing, write_sl_startup_config};
use super::ServerManager;

pub(super) fn create_server(
    manager: &ServerManager,
    req: CreateServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = current_timestamp_secs();
    let server_dir = req
        .server_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            Path::new(&req.jar_path)
                .parent()
                .map(|path| path.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| ".".to_string());

    {
        let servers = manager.lock_servers()?;
        ensure_server_identity_available(&servers, &server_name, &req.aliases, &server_dir, None)?;
    }

    let server = ServerInstance {
        id: id.clone(),
        name: server_name,
        aliases: req.aliases,
        core_type: req.core_type,
        core_version: String::new(),
        mc_version: req.mc_version,
        path: server_dir.clone(),
        port: req.port,
        max_memory: req.max_memory,
        min_memory: req.min_memory,
        created_at: now,
        last_started_at: None,
        runtime_kind: "local".to_string(),
        runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
            jar_path: req.jar_path,
            startup_mode: normalize_startup_mode(&req.startup_mode).to_string(),
            custom_command: req.custom_command,
            java_path: req.java_path,
            jvm_args: req.jvm_args,
            cpu_policy: req.cpu_policy,
            jvm_preset: req.jvm_preset,
        }),
    };

    std::fs::create_dir_all(&server_dir).map_err(|e| format!("无法创建服务器目录: {}", e))?;
    create_server_properties_if_missing(Path::new(&server_dir), req.port, true)?;
    if let Some(runtime) = server.local_runtime() {
        write_sl_startup_config(
            Path::new(&server_dir),
            req.max_memory,
            req.min_memory,
            runtime.jvm_args.clone(),
            runtime.cpu_policy.clone(),
            runtime.jvm_preset.clone(),
        )?;
    }

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::create_server;
    use crate::models::server::CreateServerRequest;
    use crate::services::server::manager::ServerManager;
    use tempfile::tempdir;

    #[test]
    fn create_server_prefers_explicit_server_path_over_jar_parent() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server_dir = temp_dir.path().join("server-root");
        let jar_parent = temp_dir.path().join("elsewhere");
        std::fs::create_dir_all(&server_dir).expect("server dir should create");
        std::fs::create_dir_all(&jar_parent).expect("jar parent should create");

        let manager = ServerManager::new();
        let req = CreateServerRequest {
            name: "Custom Script".to_string(),
            aliases: vec![],
            core_type: "Paper".to_string(),
            mc_version: "1.21.1".to_string(),
            max_memory: 4096,
            min_memory: 2048,
            port: 25565,
            java_path: "C:/Java/bin/java.exe".to_string(),
            jar_path: jar_parent.join("server.jar").to_string_lossy().to_string(),
            server_path: Some(server_dir.to_string_lossy().to_string()),
            startup_mode: "custom".to_string(),
            custom_command: Some("powershell -File start.ps1".to_string()),
            jvm_args: Vec::new(),
            cpu_policy: crate::models::server::CpuPolicyConfig::default(),
            jvm_preset: crate::models::server::JvmPresetConfig::default(),
        };

        let server = create_server(&manager, req).expect("server should be created");

        assert_eq!(server.path, server_dir.to_string_lossy().to_string());
        assert!(server_dir.join("SeaLantern").join("config.toml").exists());
    }

    #[test]
    fn create_server_creates_missing_server_dir_and_server_properties() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server_dir = temp_dir.path().join("fresh-server-root");

        let manager = ServerManager::new();
        let req = CreateServerRequest {
            name: "Fresh Server".to_string(),
            aliases: vec![],
            core_type: "Fabric".to_string(),
            mc_version: "1.20.1".to_string(),
            max_memory: 4096,
            min_memory: 2048,
            port: 25570,
            java_path: "C:/Java/bin/java.exe".to_string(),
            jar_path: server_dir.join("server.jar").to_string_lossy().to_string(),
            server_path: Some(server_dir.to_string_lossy().to_string()),
            startup_mode: "jar".to_string(),
            custom_command: None,
            jvm_args: Vec::new(),
            cpu_policy: crate::models::server::CpuPolicyConfig::default(),
            jvm_preset: crate::models::server::JvmPresetConfig::default(),
        };

        let server = create_server(&manager, req).expect("server should be created");

        assert_eq!(server.path, server_dir.to_string_lossy().to_string());
        assert!(server_dir.exists());
        assert!(server_dir.join("SeaLantern").join("config.toml").exists());
        let server_properties = std::fs::read_to_string(server_dir.join("server.properties"))
            .expect("server.properties should exist");
        assert!(server_properties.contains("server-port=25570"));
        assert!(server_properties.contains("online-mode=true"));
    }
}
