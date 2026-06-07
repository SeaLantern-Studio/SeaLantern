mod build;
mod files;
mod mapping;
mod properties;
mod run_dir;
mod startup;

use std::path::Path;

use crate::models::server::{ImportModpackRequest, ServerInstance};
use sea_lantern_server_config_core::startup::write_server_startup_config_for_dir;

use super::super::common::validate_server_name;
use super::super::ServerManager;
use super::i18n::{provisioning_t1, provisioning_t3};

fn should_delay_starter_runtime_file_writes(
    req: &ImportModpackRequest,
    source_path: &Path,
) -> bool {
    req.startup_mode.eq_ignore_ascii_case("starter")
        && source_path.is_file()
        && source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
}

pub(super) fn import_modpack(
    manager: &ServerManager,
    req: ImportModpackRequest,
) -> Result<ServerInstance, String> {
    let source_path = Path::new(&req.modpack_path);
    if !source_path.exists() {
        return Err(provisioning_t1(
            "server.provisioning.modpack_path_missing",
            req.modpack_path.clone(),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let server_name = validate_server_name(&req.name)?;
    let run_dir = run_dir::resolve_modpack_run_dir(&req.run_path)?;
    files::prepare_modpack_files(source_path, &run_dir)?;
    let startup = startup::resolve_modpack_startup_selection(&req, source_path, &run_dir)?;

    let data_dir = manager.data_dir_value()?;
    mapping::save_modpack_run_mapping(&data_dir, &id, &server_name, &req, &run_dir, &startup)?;

    let delay_runtime_file_writes = should_delay_starter_runtime_file_writes(&req, source_path);
    let port = if delay_runtime_file_writes {
        req.port
    } else {
        properties::ensure_server_properties_for_import(&run_dir, req.port, req.online_mode)?
    };

    if !delay_runtime_file_writes {
        write_server_startup_config_for_dir(
            &run_dir,
            req.max_memory,
            req.min_memory,
            req.jvm_args.clone(),
            req.cpu_policy.clone(),
            req.jvm_preset.clone(),
        )?;
    }
    let server =
        build::build_modpack_server_instance(id, server_name, req, &run_dir, startup, port);

    println!(
        "{}",
        provisioning_t3(
            "server.provisioning.created_server_instance",
            server.id.clone(),
            server.path.clone(),
            server.jar_path().unwrap_or_default(),
        )
    );

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::should_delay_starter_runtime_file_writes;
    use crate::models::server::ImportModpackRequest;
    use tempfile::tempdir;

    fn sample_request() -> ImportModpackRequest {
        ImportModpackRequest {
            name: "NeoForge".to_string(),
            aliases: Vec::new(),
            modpack_path: String::new(),
            java_path: "java".to_string(),
            max_memory: 2048,
            min_memory: 512,
            port: 25565,
            startup_mode: "starter".to_string(),
            online_mode: true,
            custom_command: None,
            run_path: String::new(),
            startup_file_path: None,
            core_type: None,
            mc_version: None,
            jvm_args: Vec::new(),
            cpu_policy: crate::models::server::CpuPolicyConfig::default(),
            jvm_preset: crate::models::server::JvmPresetConfig::default(),
        }
    }

    #[test]
    fn starter_jar_import_delays_runtime_file_writes() {
        let dir = tempdir().expect("temp dir should exist");
        let jar_path = dir.path().join("neoforge-installer.jar");
        std::fs::write(&jar_path, b"jar").expect("jar fixture should be created");

        let req = sample_request();

        assert!(should_delay_starter_runtime_file_writes(&req, &jar_path));
    }

    #[test]
    fn non_starter_import_keeps_runtime_file_writes() {
        let dir = tempdir().expect("temp dir should exist");
        let jar_path = dir.path().join("server.jar");
        std::fs::write(&jar_path, b"jar").expect("jar fixture should be created");

        let mut req = sample_request();
        req.startup_mode = "jar".to_string();

        assert!(!should_delay_starter_runtime_file_writes(&req, &jar_path));
    }
}
