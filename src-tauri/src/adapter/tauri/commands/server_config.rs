//! 服务器配置管理 Tauri 命令。

use std::collections::BTreeMap;

use sealantern_extra::config::{ServerProperties, ServerPropertiesManager};

/// 读取服务器配置文件 (server.properties)
#[tauri::command]
pub async fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    let manager = ServerPropertiesManager::new(&server_path);
    manager.read().map_err(|e| e.to_string())
}

/// 写入服务器配置文件
#[tauri::command]
pub async fn write_server_properties(
    server_path: String,
    values: BTreeMap<String, String>,
) -> Result<(), String> {
    let manager = ServerPropertiesManager::new(&server_path);
    manager.write(&values).map_err(|e| e.to_string())
}

/// 读取 server.properties 原始文本
#[tauri::command]
pub async fn read_server_properties_source(server_path: String) -> Result<String, String> {
    let manager = ServerPropertiesManager::new(&server_path);
    manager.read_source().map_err(|e| e.to_string())
}

/// 直接写入 server.properties 原始文本
#[tauri::command]
pub async fn write_server_properties_source(
    server_path: String,
    source: String,
) -> Result<(), String> {
    let manager = ServerPropertiesManager::new(&server_path);
    manager.write_source(&source).map_err(|e| e.to_string())
}

/// 将原始文本解析为可视化配置结构
#[tauri::command]
pub async fn parse_server_properties_source(source: String) -> Result<ServerProperties, String> {
    ServerPropertiesManager::parse_source(&source).map_err(|e| e.to_string())
}

/// 预览可视化配置写回后最终文本
#[tauri::command]
pub async fn preview_server_properties_write(
    server_path: String,
    values: BTreeMap<String, String>,
) -> Result<String, String> {
    let manager = ServerPropertiesManager::new(&server_path);
    manager.preview_write(&values).map_err(|e| e.to_string())
}

/// 基于给定源码预览可视化配置写回后的最终文本
#[tauri::command]
pub async fn preview_server_properties_write_from_source(
    source: String,
    values: BTreeMap<String, String>,
) -> Result<String, String> {
    ServerPropertiesManager::preview_write_from_source(&source, &values).map_err(|e| e.to_string())
}
