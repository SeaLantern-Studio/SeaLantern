use crate::services::global;
use crate::services::server::plugin_manager::{common, ops};
use sea_lantern_server_plugin_core::{m_PluginConfigFile, m_PluginInfo};

fn server_path_by_id(server_id: &str) -> Result<String, String> {
    let server_manager = global::server_manager();
    let servers = server_manager
        .servers
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let server = servers
        .iter()
        .find(|server| server.id == server_id)
        .ok_or("Server not found")?;
    Ok(server.path.clone())
}

#[tauri::command]
pub async fn m_get_plugins(server_id: String) -> Result<Vec<m_PluginInfo>, String> {
    let server_path = server_path_by_id(&server_id)?;
    common::ensure_plugins_dir(&server_path)?;
    sea_lantern_server_plugin_core::get_plugins_checked(&server_path)
}

#[tauri::command]
pub fn m_get_plugin_config_files(
    server_id: String,
    _file_name: String,
    plugin_name: String,
) -> Result<Vec<m_PluginConfigFile>, String> {
    let server_path = server_path_by_id(&server_id)?;
    sea_lantern_server_plugin_core::get_plugin_config_files(&server_path, &plugin_name)
}

#[tauri::command]
pub fn m_toggle_plugin(server_id: String, file_name: String, enabled: bool) -> Result<(), String> {
    let server_path = server_path_by_id(&server_id)?;
    ops::toggle_plugin(&server_path, &file_name, enabled)
}

#[tauri::command]
pub fn m_delete_plugin(server_id: String, file_name: String) -> Result<(), String> {
    let server_path = server_path_by_id(&server_id)?;
    ops::delete_plugin(&server_path, &file_name)
}

#[tauri::command]
pub async fn m_install_plugin(
    server_id: String,
    file_data: Vec<u8>,
    file_name: String,
) -> Result<(), String> {
    let server_path = server_path_by_id(&server_id)?;
    ops::install_plugin(&server_path, file_data, &file_name).await
}
