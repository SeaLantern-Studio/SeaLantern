use std::path::Path;

use crate::models::server::{
    AddExistingServerRequest, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
use sea_lantern_server_config_core::startup::{
    ensure_server_path_writable, read_server_port, write_server_startup_config_for_dir,
};
use sea_lantern_server_installer_core::detect_core_type;

use super::super::common::{
    current_timestamp_secs, detect_startup_mode_from_path, ensure_server_identity_available,
    validate_server_name, StartupMode,
};
use super::super::fs::find_server_executable;
use super::super::ServerManager;
use super::i18n::{provisioning_t, provisioning_t1};

pub(super) fn add_existing_server(
    manager: &ServerManager,
    req: AddExistingServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let server_path = Path::new(&req.server_path);
    if !server_path.exists() {
        return Err(provisioning_t1(
            "server.provisioning.server_dir_missing",
            req.server_path.clone(),
        ));
    }
    if !server_path.is_dir() {
        return Err(provisioning_t("server.provisioning.selected_path_not_directory"));
    }

    {
        let servers = manager.lock_servers()?;
        ensure_server_identity_available(
            &servers,
            &server_name,
            &req.aliases,
            &req.server_path,
            None,
        )?;
    }

    ensure_server_path_writable(server_path)?;

    let requested_mode = StartupMode::from_raw(&req.startup_mode);
    let (jar_path, startup_mode, custom_command) = if requested_mode.is_custom() {
        let command = req
            .custom_command
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| provisioning_t("server.provisioning.custom_command_empty"))?;
        (String::new(), requested_mode.as_str().to_string(), Some(command))
    } else if let Some(ref exec_path) = req.executable_path {
        let path = Path::new(exec_path);
        if !path.exists() {
            return Err(provisioning_t1(
                "server.provisioning.executable_missing",
                exec_path.clone(),
            ));
        }
        (exec_path.clone(), detect_startup_mode_from_path(path), None)
    } else {
        let (path, mode) = find_server_executable(server_path)?;
        (path, mode, None)
    };

    let port = read_server_port(server_path, req.port);
    write_server_startup_config_for_dir(
        server_path,
        req.max_memory,
        req.min_memory,
        req.jvm_args.clone(),
        req.cpu_policy.clone(),
        req.jvm_preset.clone(),
    )?;
    let core_type = req
        .core_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            if StartupMode::from_raw(&startup_mode).is_custom() {
                "Unknown".to_string()
            } else {
                detect_core_type(&jar_path)
            }
        });
    let mc_version = req
        .mc_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "unknown".to_string());
    println!(
        "{}",
        provisioning_t1("server.provisioning.detected_core_type", core_type.clone())
    );

    let now = current_timestamp_secs();
    let id = uuid::Uuid::new_v4().to_string();

    let server = ServerInstance {
        id: id.clone(),
        name: server_name,
        aliases: req.aliases,
        core_type,
        core_version: String::new(),
        mc_version,
        path: req.server_path,
        port,
        max_memory: req.max_memory,
        min_memory: req.min_memory,
        created_at: now,
        last_started_at: None,
        runtime_kind: "local".to_string(),
        runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
            jar_path,
            startup_mode,
            custom_command,
            java_path: req.java_path,
            jvm_args: req.jvm_args,
            cpu_policy: req.cpu_policy,
            jvm_preset: req.jvm_preset,
        }),
    };

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::add_existing_server;
    use crate::models::server::{AddExistingServerRequest, CpuPolicyConfig, JvmPresetConfig};
    use crate::services::server::manager::ServerManager;
    use crate::test_support::{lock_env, EnvGuard};
    use tempfile::tempdir;

    fn unique_name(prefix: &str) -> String {
        format!("{}-{}", prefix, uuid::Uuid::new_v4())
    }

    fn unique_alias(prefix: &str) -> String {
        format!("{}-{}", prefix, uuid::Uuid::new_v4())
    }

    #[test]
    fn add_existing_server_prefers_request_core_and_mc_version() {
        let _env_lock = lock_env();
        let temp_dir = tempdir().expect("temp dir should exist");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());
        let server_dir = temp_dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&server_dir).expect("server dir should create");
        std::fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let manager = ServerManager::new();
        let req = AddExistingServerRequest {
            name: unique_name("Paper Existing"),
            aliases: vec![unique_alias("paper_prod")],
            server_path: server_dir.to_string_lossy().to_string(),
            java_path: "C:/Java/bin/java.exe".to_string(),
            max_memory: 4096,
            min_memory: 2048,
            port: 25565,
            startup_mode: "sh".to_string(),
            executable_path: Some(server_dir.join("start.sh").to_string_lossy().to_string()),
            custom_command: None,
            core_type: Some("Paper".to_string()),
            mc_version: Some("1.21.1".to_string()),
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };

        let server = add_existing_server(&manager, req).expect("existing server should be added");

        assert_eq!(server.core_type, "Paper");
        assert_eq!(server.mc_version, "1.21.1");
        assert_eq!(server.aliases.len(), 1);
        assert!(server.aliases[0].starts_with("paper_prod-"));
        assert_eq!(server.startup_mode_str(), "sh");
    }
}
