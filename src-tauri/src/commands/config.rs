use crate::models::config::ServerProperties;
use crate::services::config_parser;
use std::collections::HashMap;
use std::path::Path;

fn validate_config_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path traversal not allowed".to_string());
        }
    }

    if path.to_string_lossy().contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    Ok(())
}

fn validate_path_within_server(server_path: &str, file_path: &str) -> Result<(), String> {
    let canonical_server =
        std::fs::canonicalize(server_path).map_err(|e| format!("无效的服务器目录: {}", e))?;

    let fp = Path::new(file_path);
    let parent = fp.parent().unwrap_or(fp);
    let canonical_parent =
        std::fs::canonicalize(parent).map_err(|e| format!("无效的配置路径: {}", e))?;

    if !canonical_parent.starts_with(&canonical_server) {
        return Err("配置路径必须在服务器目录内".to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn read_config(server_path: String, path: String) -> Result<HashMap<String, String>, String> {
    validate_config_path(&path)?;
    validate_path_within_server(&server_path, &path)?;
    config_parser::read_properties(&path)
}

#[tauri::command]
pub fn write_config(
    server_path: String,
    path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    validate_config_path(&path)?;
    validate_path_within_server(&server_path, &path)?;
    config_parser::write_properties(&path, &values)
}

#[tauri::command]
pub fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    validate_config_path(&server_path)?;
    let props_path = format!("{}/server.properties", server_path);
    validate_path_within_server(&server_path, &props_path)?;
    config_parser::parse_server_properties(&props_path)
}

#[tauri::command]
pub fn write_server_properties(
    server_path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    validate_config_path(&server_path)?;
    let props_path = format!("{}/server.properties", server_path);
    validate_path_within_server(&server_path, &props_path)?;
    config_parser::write_properties(&props_path, &values)
}
