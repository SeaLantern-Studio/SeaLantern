use super::super::super::common::{
    resolve_managed_console_encoding, ManagedConsoleEncoding, StartupMode,
};
use crate::models::server::ServerInstance;
use crate::services::server::manager::i18n::manager_t1;
use sea_lantern_server_installer_core::{detect_core_type, CoreType};
use sea_lantern_server_local_setup_core::{
    resolve_java_paths as resolve_shared_java_paths,
    startup_filename as resolve_shared_startup_filename,
};
use std::path::Path;

pub(in crate::services::server::manager::runtime_start) struct LaunchContext<'a> {
    pub server: &'a ServerInstance,
    pub settings: &'a crate::models::settings::AppSettings,
    pub startup_mode: StartupMode,
    pub managed_console_encoding: ManagedConsoleEncoding,
    pub java_bin_dir_str: String,
    pub java_home_dir_str: String,
    pub startup_filename: String,
    pub starter_core_key: String,
}

pub(in crate::services::server::manager::runtime_start) fn resolve_managed_encoding(
    startup_mode: StartupMode,
    startup_path: &Path,
) -> ManagedConsoleEncoding {
    if startup_mode.is_custom() {
        ManagedConsoleEncoding::Utf8
    } else {
        resolve_managed_console_encoding(startup_mode, startup_path)
    }
}

pub(in crate::services::server::manager::runtime_start) fn resolve_java_paths(
    java_path: &str,
) -> Result<(String, String), String> {
    resolve_shared_java_paths(java_path)
}

pub(in crate::services::server::manager::runtime_start) fn startup_filename(
    startup_path: &str,
) -> String {
    resolve_shared_startup_filename(startup_path)
}

pub(in crate::services::server::manager::runtime_start) fn resolve_starter_core_key(
    server: &ServerInstance,
) -> Result<String, String> {
    let startup_mode = StartupMode::from_raw(server.startup_mode_str());
    if !startup_mode.is_starter() {
        return Ok(String::new());
    }

    let detected_core_type = detect_core_type(server.jar_path().unwrap_or_default());
    CoreType::normalize_to_api_core_key(&server.core_type)
        .or_else(|| CoreType::normalize_to_api_core_key(&detected_core_type))
        .ok_or_else(|| {
            manager_t1(
                "server.manager.starter_core_type_unrecognized",
                if server.core_type.trim().is_empty() {
                    detected_core_type
                } else {
                    server.core_type.clone()
                },
            )
        })
}

#[cfg(test)]
mod tests {
    use super::resolve_starter_core_key;
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };

    fn test_server(startup_mode: &str, core_type: &str, jar_path: &str) -> ServerInstance {
        ServerInstance {
            id: format!("context-{}", startup_mode),
            name: "Context".to_string(),
            aliases: Vec::new(),
            core_type: core_type.to_string(),
            core_version: core_type.to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/context".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: jar_path.to_string(),
                startup_mode: startup_mode.to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn resolve_starter_core_key_returns_empty_string_for_non_starter_modes() {
        let server = test_server("jar", "paper", "server.jar");

        let core_key = resolve_starter_core_key(&server).expect("jar mode should not fail");

        assert_eq!(core_key, "");
    }

    #[test]
    fn resolve_starter_core_key_prefers_normalized_server_core_type_for_starter() {
        let server = test_server(
            "starter",
            "Neoforge",
            "E:/minecraft/neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar",
        );

        let core_key = resolve_starter_core_key(&server).expect("starter core type should resolve");

        assert_eq!(core_key, "neoforge");
    }
}
