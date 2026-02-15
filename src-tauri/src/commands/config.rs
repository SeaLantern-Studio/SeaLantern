use crate::models::config::ServerProperties;
use crate::services::config_parser;
use std::collections::HashMap;

#[tauri::command]
pub fn read_config(path: String) -> Result<HashMap<String, String>, String> {
    config_parser::read_properties(&path)
}

#[tauri::command]
pub fn write_config(path: String, values: HashMap<String, String>) -> Result<(), String> {
    config_parser::write_properties(&path, &values)
}

#[tauri::command]
pub fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    let props_path = format!("{}/server.properties", server_path);
    config_parser::parse_server_properties(&props_path)
}

#[tauri::command]
pub fn write_server_properties(
    server_path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    let props_path = format!("{}/server.properties", server_path);
    config_parser::write_properties(&props_path, &values)
}
