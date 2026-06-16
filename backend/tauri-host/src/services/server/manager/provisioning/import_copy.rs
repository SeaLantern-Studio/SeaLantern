use std::path::Path;

use crate::models::server::{
    ImportServerRequest, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
use sea_lantern_server_config_core::startup::{
    create_server_properties_if_missing, read_server_port, write_server_startup_config_for_dir,
};
use sea_lantern_server_installer_core::resolve_imported_server_core_key;
use sea_lantern_server_local_setup_core::normalize_cli_startup_mode;

use super::super::common::{current_timestamp_secs, validate_server_name};
use super::super::fs::copy_dir_recursive;
use super::super::ServerManager;
use super::i18n::{provisioning_t, provisioning_t1, provisioning_t2};

pub(super) fn import_server(
    manager: &ServerManager,
    req: ImportServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let startup_mode =
        normalize_cli_startup_mode(Some(&req.startup_mode)).unwrap_or_else(|_| "jar".to_string());
    let source_startup_file = Path::new(&req.jar_path);
    if !source_startup_file.exists() {
        return Err(provisioning_t1(
            "server.provisioning.startup_file_missing",
            req.jar_path.clone(),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let data_dir = manager.data_dir_value()?;
    let servers_dir = Path::new(&data_dir).join("servers");
    let server_dir = servers_dir.join(&id);

    std::fs::create_dir_all(&server_dir).map_err(|e| {
        provisioning_t1("server.provisioning.create_server_dir_failed", e.to_string())
    })?;

    let startup_filename = source_startup_file
        .file_name()
        .ok_or_else(|| provisioning_t("server.provisioning.startup_filename_missing"))?;
    let source_server_dir = source_startup_file
        .parent()
        .ok_or_else(|| provisioning_t("server.provisioning.startup_parent_missing"))?;

    println!(
        "{}",
        provisioning_t2(
            "server.provisioning.import_copying_source_dir",
            source_server_dir.display().to_string(),
            server_dir.display().to_string(),
        )
    );
    copy_dir_recursive(source_server_dir, &server_dir).map_err(|e| {
        provisioning_t1("server.provisioning.copy_server_dir_failed", e.to_string())
    })?;

    let dest_startup = server_dir.join(startup_filename);
    if !dest_startup.exists() {
        return Err(provisioning_t1(
            "server.provisioning.copied_startup_missing",
            dest_startup.display().to_string(),
        ));
    }

    create_server_properties_if_missing(&server_dir, req.port, req.online_mode)?;
    let port = read_server_port(&server_dir, req.port);
    write_server_startup_config_for_dir(
        &server_dir,
        req.max_memory,
        req.min_memory,
        req.jvm_args.clone(),
        req.cpu_policy.clone(),
        req.jvm_preset.clone(),
    )?;

    let now = current_timestamp_secs();
    let core_type =
        resolve_imported_server_core_key(&startup_mode, &dest_startup.to_string_lossy());
    println!(
        "{}",
        provisioning_t1("server.provisioning.detected_core_type", core_type.clone())
    );

    let server = ServerInstance {
        id: id.clone(),
        name: server_name,
        aliases: req.aliases,
        core_type,
        core_version: String::new(),
        mc_version: "unknown".into(),
        path: server_dir.to_string_lossy().to_string(),
        port,
        max_memory: req.max_memory,
        min_memory: req.min_memory,
        created_at: now,
        last_started_at: None,
        runtime_kind: "local".to_string(),
        runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
            jar_path: dest_startup.to_string_lossy().to_string(),
            startup_mode,
            custom_command: req.custom_command,
            java_path: req.java_path,
            jvm_args: req.jvm_args,
            terminal_mode: req.terminal_mode,
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
    use super::import_server;
    use crate::models::server::{CpuPolicyConfig, ImportServerRequest, JvmPresetConfig};
    use crate::services::server::manager::ServerManager;
    use crate::test_support::{lock_env, EnvGuard};
    use tempfile::tempdir;

    #[test]
    fn import_server_reuses_shared_imported_core_resolution() {
        let _env_lock = lock_env();
        let temp_dir = tempdir().expect("temp dir should exist");
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());
        let source_dir = temp_dir.path().join("source");
        std::fs::create_dir_all(&source_dir).expect("source dir should create");
        let startup_path = source_dir.join("pumpkin-launcher.exe");
        std::fs::write(&startup_path, b"placeholder").expect("startup file should write");

        let manager = ServerManager::new();
        let req = ImportServerRequest {
            name: "Pumpkin Import".to_string(),
            aliases: Vec::new(),
            jar_path: startup_path.to_string_lossy().to_string(),
            java_path: "C:/Java/bin/java.exe".to_string(),
            startup_mode: "custom".to_string(),
            custom_command: Some(startup_path.to_string_lossy().to_string()),
            max_memory: 4096,
            min_memory: 2048,
            port: 25565,
            online_mode: false,
            jvm_args: Vec::new(),
            terminal_mode: crate::models::server::LocalTerminalMode::PipeManaged,
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };

        let server = import_server(&manager, req).expect("import server should succeed");

        assert_eq!(server.core_type, "pumpkin");
        assert_eq!(server.startup_mode_str(), "custom");
    }
}
