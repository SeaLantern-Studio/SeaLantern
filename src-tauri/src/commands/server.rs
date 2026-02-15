use crate::models::server::*;
use crate::services::global;

fn manager() -> &'static crate::services::server_manager::ServerManager {
    global::server_manager()
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_server(
    name: String,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
) -> Result<ServerInstance, String> {
    let req = CreateServerRequest {
        name,
        core_type,
        mc_version,
        max_memory,
        min_memory,
        port,
        java_path,
        jar_path,
    };
    manager().create_server(req)
}

#[tauri::command]
pub fn import_server(
    name: String,
    jar_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    online_mode: bool,
) -> Result<ServerInstance, String> {
    let req = ImportServerRequest {
        name,
        jar_path,
        java_path,
        max_memory,
        min_memory,
        port,
        online_mode,
    };
    manager().import_server(req)
}

#[tauri::command]
pub fn import_modpack(
    name: String,
    modpack_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
) -> Result<ServerInstance, String> {
    let req = ImportModpackRequest {
        name,
        modpack_path,
        java_path,
        max_memory,
        min_memory,
        port,
    };
    manager().import_modpack(req)
}

#[tauri::command]
pub fn start_server(id: String) -> Result<(), String> {
    manager().start_server(&id)
}

#[tauri::command]
pub fn stop_server(id: String) -> Result<(), String> {
    manager().stop_server(&id)
}

#[tauri::command]
pub fn send_command(id: String, command: String) -> Result<(), String> {
    manager().send_command(&id, &command)
}

#[tauri::command]
pub fn get_server_list() -> Vec<ServerInstance> {
    manager().get_server_list()
}

#[tauri::command]
pub fn get_server_status(id: String) -> ServerStatusInfo {
    manager().get_server_status(&id)
}

#[tauri::command]
pub fn delete_server(id: String) -> Result<(), String> {
    manager().delete_server(&id)
}

#[tauri::command]
pub fn get_server_logs(id: String, since: usize) -> Vec<String> {
    manager().get_logs(&id, since)
}

#[tauri::command]
pub fn add_server_command(id: String, name: String, command: String) -> Result<(), String> {
    manager().add_server_command(&id, &name, &command)
}

#[tauri::command]
pub fn update_server_command(
    id: String,
    command_id: String,
    name: String,
    command: String,
) -> Result<(), String> {
    manager().update_server_command(&id, &command_id, &name, &command)
}

#[tauri::command]
pub fn delete_server_command(id: String, command_id: String) -> Result<(), String> {
    manager().delete_server_command(&id, &command_id)
}

#[tauri::command]
pub fn update_server_name(id: String, name: String) -> Result<(), String> {
    manager().update_server_name(&id, &name)
}
