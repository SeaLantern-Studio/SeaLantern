use crate::models::mcs_plugin::*;
use crate::services::global;

fn m_manager() -> &'static crate::services::mcs_plugin_manager::m_PluginManager {
    global::m_plugin_manager()
}

#[tauri::command]
pub fn m_get_plugins(server_id: String) -> Result<Vec<m_PluginInfo>, String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    m_manager().m_get_plugins(&server.path)
}

#[tauri::command]
pub fn m_toggle_plugin(server_id: String, file_name: String, enabled: bool) -> Result<(), String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    m_manager().m_toggle_plugin(&server.path, &file_name, enabled)
}

#[tauri::command]
pub fn m_delete_plugin(server_id: String, file_name: String) -> Result<(), String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    m_manager().m_delete_plugin(&server.path, &file_name)
}

#[tauri::command]
pub async fn m_install_plugin(
    server_id: String,
    file_data: Vec<u8>,
    file_name: String,
) -> Result<(), String> {
    let server_path = {
        let server_manager = global::server_manager();
        let servers = server_manager.servers.lock().unwrap();
        let server = servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or("Server not found")?;
        server.path.clone()
    };

    m_manager()
        .m_install_plugin(&server_path, file_data, &file_name)
        .await
}
