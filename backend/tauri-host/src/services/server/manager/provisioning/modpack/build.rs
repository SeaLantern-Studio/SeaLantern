use std::path::Path;

use crate::models::server::{
    ImportModpackRequest, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
use server_installer::resolve_imported_server_core_key;
use server_local_setup::{canonical_core_type, ModpackStartupSelection};

pub(super) fn build_modpack_server_instance(
    id: String,
    server_name: String,
    req: ImportModpackRequest,
    run_dir: &Path,
    startup: ModpackStartupSelection,
    port: u16,
) -> ServerInstance {
    let startup_path = startup.startup_file_path.clone().unwrap_or_default();
    let detected_core_type = resolve_imported_server_core_key(&startup.startup_mode, &startup_path);
    let core_type = startup
        .selected_core_type
        .as_deref()
        .map(canonical_core_type)
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

#[cfg(test)]
mod tests {
    use super::build_modpack_server_instance;
    use crate::models::server::{CpuPolicyConfig, ImportModpackRequest, JvmPresetConfig};
    use std::path::Path;

    use server_local_setup::ModpackStartupSelection;

    fn sample_request() -> ImportModpackRequest {
        ImportModpackRequest {
            name: "Imported Server".to_string(),
            aliases: vec!["imported_alias".to_string()],
            modpack_path: "E:/packs/sample.zip".to_string(),
            java_path: "C:/Java/bin/java.exe".to_string(),
            max_memory: 4096,
            min_memory: 2048,
            port: 25565,
            startup_mode: "jar".to_string(),
            online_mode: true,
            custom_command: None,
            run_path: "E:/servers/imported".to_string(),
            startup_file_path: None,
            core_type: None,
            mc_version: None,
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        }
    }

    #[test]
    fn build_modpack_server_instance_canonicalizes_selected_core_type() {
        let req = sample_request();
        let startup = ModpackStartupSelection {
            startup_mode: "jar".to_string(),
            custom_command: None,
            startup_file_path: Some("E:/servers/imported/server.jar".to_string()),
            selected_core_type: Some("AllayMC".to_string()),
            selected_mc_version: Some("1.21.1".to_string()),
        };

        let server = build_modpack_server_instance(
            "server-1".to_string(),
            "Imported Server".to_string(),
            req,
            Path::new("E:/servers/imported"),
            startup,
            25565,
        );

        assert_eq!(server.core_type, "allay");
        assert_eq!(server.mc_version, "1.21.1");
    }

    #[test]
    fn build_modpack_server_instance_canonicalizes_detected_core_type() {
        let req = sample_request();
        let startup = ModpackStartupSelection {
            startup_mode: "jar".to_string(),
            custom_command: None,
            startup_file_path: Some("E:/servers/imported/paper-server.jar".to_string()),
            selected_core_type: None,
            selected_mc_version: None,
        };

        let server = build_modpack_server_instance(
            "server-2".to_string(),
            "Imported Server".to_string(),
            req,
            Path::new("E:/servers/imported"),
            startup,
            25565,
        );

        assert_eq!(server.core_type, "paper");
        assert_eq!(server.mc_version, "unknown");
    }

    #[test]
    fn build_modpack_server_instance_canonicalizes_detected_legacy_core_alias() {
        let req = sample_request();
        let startup = ModpackStartupSelection {
            startup_mode: "jar".to_string(),
            custom_command: None,
            startup_file_path: Some("E:/servers/imported/nukkit.jar".to_string()),
            selected_core_type: None,
            selected_mc_version: None,
        };

        let server = build_modpack_server_instance(
            "server-3".to_string(),
            "Imported Server".to_string(),
            req,
            Path::new("E:/servers/imported"),
            startup,
            25565,
        );

        assert_eq!(server.core_type, "nukkit");
    }

    #[test]
    fn build_modpack_server_instance_keeps_custom_non_pumpkin_entries_as_custom() {
        let req = sample_request();
        let startup = ModpackStartupSelection {
            startup_mode: "custom".to_string(),
            custom_command: Some("./start-custom.sh".to_string()),
            startup_file_path: Some("E:/servers/imported/start-custom.sh".to_string()),
            selected_core_type: None,
            selected_mc_version: None,
        };

        let server = build_modpack_server_instance(
            "server-4".to_string(),
            "Imported Server".to_string(),
            req,
            Path::new("E:/servers/imported"),
            startup,
            25565,
        );

        assert_eq!(server.core_type, "custom");
    }

    #[test]
    fn build_modpack_server_instance_keeps_custom_pumpkin_entries_detectable() {
        let req = sample_request();
        let startup = ModpackStartupSelection {
            startup_mode: "custom".to_string(),
            custom_command: Some("./pumpkin-launcher.exe".to_string()),
            startup_file_path: Some("E:/servers/imported/pumpkin-launcher.exe".to_string()),
            selected_core_type: None,
            selected_mc_version: None,
        };

        let server = build_modpack_server_instance(
            "server-5".to_string(),
            "Imported Server".to_string(),
            req,
            Path::new("E:/servers/imported"),
            startup,
            25565,
        );

        assert_eq!(server.core_type, "pumpkin");
    }
}
