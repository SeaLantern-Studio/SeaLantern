use crate::models::plugin::*;
use crate::services::global;

fn manager() -> &'static crate::services::plugin_manager::PluginManager {
    global::plugin_manager()
}

#[tauri::command]
pub fn get_plugins(server_id: String) -> Result<Vec<PluginInfo>, String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    manager().get_plugins(&server.path)
}

#[tauri::command]
pub fn toggle_plugin(server_id: String, file_name: String, enabled: bool) -> Result<(), String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    manager().toggle_plugin(&server.path, &file_name, enabled)
}

#[tauri::command]
pub fn delete_plugin(server_id: String, file_name: String) -> Result<(), String> {
    let server_manager = global::server_manager();
    let servers = server_manager.servers.lock().unwrap();
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    manager().delete_plugin(&server.path, &file_name)
}

#[tauri::command]
pub async fn install_plugin(
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

    manager()
        .install_plugin(&server_path, file_data, &file_name)
        .await
}
