mod common;
mod dialogs;
mod host_io;
mod resources;
mod system_info;

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
