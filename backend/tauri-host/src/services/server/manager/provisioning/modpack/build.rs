use std::path::Path;
use std::str::FromStr;

use crate::models::server::{
    ImportModpackRequest, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
use sea_lantern_server_installer_core::{detect_core_type, CoreType};

use super::startup::ModpackStartupSelection;

pub(super) fn build_modpack_server_instance(
    id: String,
    server_name: String,
    req: ImportModpackRequest,
    run_dir: &Path,
    startup: ModpackStartupSelection,
    port: u16,
) -> ServerInstance {
    let startup_path = startup.startup_file_path.clone().unwrap_or_default();
    let detected_core_type = if startup.startup_mode == "custom" {
        if startup_path.to_ascii_lowercase().contains("pumpkin") {
            "Pumpkin".to_string()
        } else {
            "custom".to_string()
        }
    } else {
        detect_core_type(&startup_path)
    };
    let core_type = startup
        .selected_core_type
        .and_then(|value| {
            CoreType::from_str(&value)
                .ok()
                .map(|core| core.to_string())
                .or(Some(value))
        })
        .unwrap_or(detected_core_type);
    let mc_version = startup
        .selected_mc_version
        .unwrap_or_else(|| "unknown".to_string());

    ServerInstance {
        id,
        name: server_name,
        aliases: req.aliases,
        core_type,
        core_version: String::new(),
        mc_version,
        path: run_dir.to_string_lossy().to_string(),
        port,
        max_memory: req.max_memory,
        min_memory: req.min_memory,
        created_at: super::super::super::common::current_timestamp_secs(),
        last_started_at: None,
        runtime_kind: "local".to_string(),
        runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
            jar_path: startup_path,
            startup_mode: startup.startup_mode,
            custom_command: startup.custom_command,
            java_path: req.java_path,
            jvm_args: req.jvm_args,
            cpu_policy: req.cpu_policy,
            jvm_preset: req.jvm_preset,
        }),
    }
}
