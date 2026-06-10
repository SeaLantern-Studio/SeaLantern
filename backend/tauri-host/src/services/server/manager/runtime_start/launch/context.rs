use super::super::super::common::StartupMode;
use crate::models::server::ServerInstance;
use crate::services::server::manager::i18n::manager_t1;
use sea_lantern_server_installer_core::resolve_starter_core_key_checked as resolve_shared_starter_core_key;
use sea_lantern_server_local_setup_core::ManagedConsoleEncoding;

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

pub(in crate::services::server::manager::runtime_start) fn resolve_starter_core_key(
    server: &ServerInstance,
) -> Result<String, String> {
    let resolution = resolve_shared_starter_core_key(
        server.startup_mode_str(),
        Some(&server.core_type),
        server.jar_path(),
    );
    if resolution.needs_unrecognized_error(server.startup_mode_str()) {
        return Err(manager_t1(
            "server.manager.starter_core_type_unrecognized",
            resolution.unresolved_display_hint(Some(&server.core_type)),
        ));
    }

    Ok(resolution.starter_core_key)
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

    #[test]
    fn resolve_starter_core_key_falls_back_to_detected_launch_target() {
        let server = test_server(
            "starter",
            "   ",
            "E:/minecraft/neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar",
        );

        let core_key = resolve_starter_core_key(&server)
            .expect("starter core type should fall back to detected launch target");

        assert_eq!(core_key, "neoforge");
    }
}
