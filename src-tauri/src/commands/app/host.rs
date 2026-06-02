mod common;
mod dialogs;
mod host_io;
mod resources;
mod system_info;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateServerDefaults {
    pub default_run_path: String,
    pub suggested_run_path: String,
    pub default_max_memory: u32,
    pub default_min_memory: u32,
    pub default_port: u16,
    pub cached_java_list: Vec<crate::services::java_detector::JavaInfo>,
    pub preferred_java_path: String,
}

#[tauri::command]
pub fn get_system_info() -> Result<serde_json::Value, String> {
    system_info::get_system_info()
}

#[tauri::command]
pub fn get_server_resource_usage(server_id: String) -> Result<serde_json::Value, String> {
    resources::get_server_resource_usage(server_id)
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

    Ok(CreateServerDefaults {
        default_run_path,
        suggested_run_path,
        default_max_memory: settings.default_max_memory,
        default_min_memory: settings.default_min_memory,
        default_port: settings.default_port,
        cached_java_list: settings.cached_java_list,
        preferred_java_path,
    })
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
pub async fn test_ipv6_connectivity() -> Result<serde_json::Value, String> {
    system_info::test_ipv6_connectivity().await
}
