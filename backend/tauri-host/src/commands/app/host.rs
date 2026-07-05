mod common;
mod dialogs;
mod host_io;
mod resources;
mod system_info;

use crate::services::event_consumer_registry::{
    EventConsumerRegistryEntryDto, EventConsumerRegistryFilterUpdateRequest,
    EventConsumerRegistryMetadataUpdateRequest,
};
use crate::utils::app_version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum HostBuildFlavor {
    #[serde(rename = "desktop-full")]
    DesktopFull,
    #[serde(rename = "desktop-min")]
    DesktopMin,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct HostPluginRuntimeCapabilities {
    pub available: bool,
    pub local_runtime: bool,
    pub ui_bridge: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct HostCapabilities {
    pub build_flavor: HostBuildFlavor,
    pub plugin_runtime: HostPluginRuntimeCapabilities,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateServerDefaults {
    pub default_run_path: String,
    pub suggested_run_path: String,
    pub default_max_memory: u32,
    pub default_min_memory: u32,
    pub default_port: u16,
    pub cached_java_list: Vec<crate::services::java_detector::JavaInfo>,
    pub preferred_java_path: String,
    pub default_jvm_args: Vec<String>,
    pub default_cpu_policy: crate::models::server::CpuPolicyConfig,
    pub default_jvm_preset: crate::models::server::JvmPresetConfig,
}

fn build_create_server_defaults(
    settings: crate::models::settings::AppSettings,
    default_run_path: String,
) -> CreateServerDefaults {
    let suggested_base_path = if settings.last_run_path.trim().is_empty() {
        default_run_path.clone()
    } else {
        settings.last_run_path.trim().to_string()
    };

    let suggested_run_path = host_io::append_generated_server_dir(&suggested_base_path);

    let preferred_java_path = if !settings.default_java_path.trim().is_empty() {
        settings.default_java_path.trim().to_string()
    } else {
        settings
            .cached_java_list
            .iter()
            .find(|java| java.is_64bit && java.major_version >= 17)
            .or_else(|| settings.cached_java_list.first())
            .map(|java| java.path.clone())
            .unwrap_or_default()
    };

    CreateServerDefaults {
        default_run_path,
        suggested_run_path,
        default_max_memory: settings.default_max_memory,
        default_min_memory: settings.default_min_memory,
        default_port: settings.default_port,
        cached_java_list: settings.cached_java_list,
        preferred_java_path,
        default_jvm_args: settings.default_jvm_args,
        default_cpu_policy: settings.default_cpu_policy,
        default_jvm_preset: settings.default_jvm_preset,
    }
}

fn build_host_capabilities_from_flags(
    local_runtime_enabled: bool,
    ui_bridge_enabled: bool,
) -> HostCapabilities {
    let build_flavor = match (local_runtime_enabled, ui_bridge_enabled) {
        (true, true) => HostBuildFlavor::DesktopFull,
        (false, false) => HostBuildFlavor::DesktopMin,
        _ => HostBuildFlavor::Custom,
    };

    HostCapabilities {
        build_flavor,
        plugin_runtime: HostPluginRuntimeCapabilities {
            available: local_runtime_enabled || ui_bridge_enabled,
            local_runtime: local_runtime_enabled,
            ui_bridge: ui_bridge_enabled,
        },
    }
}

fn build_host_capabilities() -> HostCapabilities {
    build_host_capabilities_from_flags(
        cfg!(feature = "plugin-local-runtime"),
        cfg!(feature = "plugin-runtime-bridge"),
    )
}

#[tauri::command]
pub fn get_system_info() -> Result<serde_json::Value, String> {
    system_info::get_system_info()
}

#[tauri::command]
pub fn get_host_capabilities() -> Result<HostCapabilities, String> {
    Ok(build_host_capabilities())
}

#[tauri::command]
pub fn get_server_resource_usage(server_id: String) -> Result<serde_json::Value, String> {
    resources::get_server_resource_usage(server_id)
}

#[tauri::command]
pub fn list_event_consumers() -> Result<Vec<EventConsumerRegistryEntryDto>, String> {
    Ok(crate::services::global::event_consumer_registry_service().list())
}

#[tauri::command]
pub fn get_event_consumer(name: String) -> Result<Option<EventConsumerRegistryEntryDto>, String> {
    Ok(crate::services::global::event_consumer_registry_service().get(&name))
}

#[tauri::command]
pub fn set_event_consumer_enabled(
    name: String,
    enabled: bool,
) -> Result<EventConsumerRegistryEntryDto, String> {
    crate::services::global::event_consumer_registry_service().set_enabled(&name, enabled)
}

#[tauri::command]
pub fn update_event_consumer_filters(
    name: String,
    request: EventConsumerRegistryFilterUpdateRequest,
) -> Result<EventConsumerRegistryEntryDto, String> {
    crate::services::global::event_consumer_registry_service().update_filters(&name, request)
}

#[tauri::command]
pub fn update_event_consumer_metadata(
    name: String,
    request: EventConsumerRegistryMetadataUpdateRequest,
) -> Result<EventConsumerRegistryEntryDto, String> {
    crate::services::global::event_consumer_registry_service().update_metadata(&name, request)
}

#[tauri::command]
pub async fn pick_jar_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_jar_file(app).await
}

#[tauri::command]
pub async fn pick_archive_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_archive_file(app).await
}

#[tauri::command]
pub async fn pick_startup_file(
    app: tauri::AppHandle,
    mode: String,
) -> Result<Option<String>, String> {
    dialogs::pick_startup_file(app, mode).await
}

#[tauri::command]
pub async fn pick_server_executable(
    app: tauri::AppHandle,
) -> Result<Option<(String, String)>, String> {
    dialogs::pick_server_executable(app).await
}

#[tauri::command]
pub async fn pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_java_file(app).await
}

#[tauri::command]
pub async fn pick_save_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_save_file(app).await
}

#[tauri::command]
pub async fn pick_personalization_export_file(
    app: tauri::AppHandle,
    suggested_name: String,
) -> Result<Option<String>, String> {
    dialogs::pick_personalization_export_file(app, suggested_name).await
}

#[tauri::command]
pub async fn pick_personalization_import_file(
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    dialogs::pick_personalization_import_file(app).await
}

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_folder(app).await
}

#[tauri::command]
pub async fn pick_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_file(app).await
}

#[tauri::command]
pub async fn pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    dialogs::pick_image_file(app).await
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    host_io::open_file(path)
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    host_io::open_folder(path)
}

#[tauri::command]
pub fn get_default_run_path() -> Result<String, String> {
    host_io::get_default_run_path()
}

#[tauri::command]
pub fn get_create_server_defaults() -> Result<CreateServerDefaults, String> {
    let settings = crate::services::global::settings_manager().get();
    let default_run_path = host_io::get_default_run_path()?;

    Ok(build_create_server_defaults(settings, default_run_path))
}

#[tauri::command]
pub fn get_safe_mode_status() -> Result<bool, String> {
    host_io::get_safe_mode_status()
}

#[tauri::command]
pub fn frontend_heartbeat() -> Result<(), String> {
    host_io::frontend_heartbeat()
}

#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    Ok(app_version::display_version())
}

#[tauri::command]
pub async fn test_ipv6_connectivity() -> Result<serde_json::Value, String> {
    system_info::test_ipv6_connectivity().await
}

#[cfg(test)]
mod tests {
    use super::{
        build_create_server_defaults, build_host_capabilities_from_flags, HostBuildFlavor,
    };
    use crate::models::server::{CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId};
    use crate::models::settings::AppSettings;
    use crate::services::java_detector::JavaInfo;

    fn sample_java(path: &str, major_version: u32, is_64bit: bool) -> JavaInfo {
        JavaInfo {
            path: path.to_string(),
            version: format!("{}", major_version),
            major_version,
            is_64bit,
            vendor: "TestVendor".to_string(),
        }
    }

    #[test]
    fn create_server_defaults_include_startup_related_defaults() {
        let settings = AppSettings {
            default_max_memory: 6144,
            default_min_memory: 2048,
            default_port: 25570,
            last_run_path: "E:/servers".to_string(),
            default_jvm_args: vec!["-XX:+UseG1GC".to_string(), "-Dfoo=bar".to_string()],
            default_cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(4),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            default_jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
            cached_java_list: vec![
                sample_java("C:/Java/jre8/bin/java.exe", 8, true),
                sample_java("C:/Java/jdk21/bin/java.exe", 21, true),
            ],
            ..AppSettings::default()
        };

        let defaults = build_create_server_defaults(settings, "E:/default-run".to_string());

        assert_eq!(defaults.default_max_memory, 6144);
        assert_eq!(defaults.default_min_memory, 2048);
        assert_eq!(defaults.default_port, 25570);
        assert_eq!(defaults.default_jvm_args, vec!["-XX:+UseG1GC", "-Dfoo=bar"]);
        assert_eq!(defaults.default_cpu_policy.mode, CpuPolicyMode::Count);
        assert_eq!(defaults.default_cpu_policy.count, Some(4));
        assert_eq!(defaults.default_jvm_preset.preset, JvmPresetId::AikarG1);
        assert_eq!(defaults.preferred_java_path, "C:/Java/jdk21/bin/java.exe");
        assert!(defaults.suggested_run_path.starts_with("E:/servers"));
    }

    #[test]
    fn host_capabilities_mark_desktop_full_when_runtime_and_bridge_are_enabled() {
        let capabilities = build_host_capabilities_from_flags(true, true);

        assert_eq!(capabilities.build_flavor, HostBuildFlavor::DesktopFull);
        assert!(capabilities.plugin_runtime.available);
        assert!(capabilities.plugin_runtime.local_runtime);
        assert!(capabilities.plugin_runtime.ui_bridge);
    }

    #[test]
    fn host_capabilities_mark_desktop_min_when_runtime_and_bridge_are_disabled() {
        let capabilities = build_host_capabilities_from_flags(false, false);

        assert_eq!(capabilities.build_flavor, HostBuildFlavor::DesktopMin);
        assert!(!capabilities.plugin_runtime.available);
        assert!(!capabilities.plugin_runtime.local_runtime);
        assert!(!capabilities.plugin_runtime.ui_bridge);
    }

    #[test]
    fn host_capabilities_mark_custom_for_mixed_runtime_flags() {
        let capabilities = build_host_capabilities_from_flags(false, true);

        assert_eq!(capabilities.build_flavor, HostBuildFlavor::Custom);
        assert!(capabilities.plugin_runtime.available);
        assert!(!capabilities.plugin_runtime.local_runtime);
        assert!(capabilities.plugin_runtime.ui_bridge);
    }
}
