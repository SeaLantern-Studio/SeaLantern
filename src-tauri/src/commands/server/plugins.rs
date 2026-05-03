use crate::models::mcs_plugin::{m_PluginConfigFile, m_PluginInfo};
use crate::services::global;

fn plugin_manager() -> &'static crate::services::server::plugin_manager::ServerPluginManager {
    global::server_plugin_manager()
}

fn server_path_by_id(server_id: &str) -> Result<String, String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|server| server.id == server_id)
        .ok_or("Server not found")?;
    Ok(server.path.clone())
}

#[tauri::command]
pub async fn m_get_plugins(server_id: String) -> Result<Vec<m_PluginInfo>, String> {
    let server_path = server_path_by_id(&server_id)?;
    plugin_manager().get_plugins(&server_path).await
}

#[tauri::command]
pub fn m_get_plugin_config_files(
    server_id: String,
    _file_name: String,
    plugin_name: String,
) -> Result<Vec<m_PluginConfigFile>, String> {
    let server_path = server_path_by_id(&server_id)?;
    plugin_manager().get_plugin_config_files(&server_path, &plugin_name)
}

#[tauri::command]
pub fn m_toggle_plugin(server_id: String, file_name: String, enabled: bool) -> Result<(), String> {
    let server_path = server_path_by_id(&server_id)?;
    plugin_manager().toggle_plugin(&server_path, &file_name, enabled)
}

#[tauri::command]
pub fn m_delete_plugin(server_id: String, file_name: String) -> Result<(), String> {
    let server_path = server_path_by_id(&server_id)?;
    plugin_manager().delete_plugin(&server_path, &file_name)
}

#[tauri::command]
pub async fn m_install_plugin(
    server_id: String,
    file_data: Vec<u8>,
    file_name: String,
) -> Result<(), String> {
    let server_path = server_path_by_id(&server_id)?;
    plugin_manager()
        .install_plugin(&server_path, file_data, &file_name)
        .await
}
