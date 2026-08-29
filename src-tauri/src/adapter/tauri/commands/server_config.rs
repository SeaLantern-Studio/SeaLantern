//! 服务器配置管理 Tauri 命令。

use std::collections::BTreeMap;

use sealantern_feature::config::{
    ServerProperties, ServerPropertiesError, ServerPropertiesManager,
};

async fn run_blocking<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ServerPropertiesError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| format!("server properties task failed: {error}"))?
        .map_err(|error| error.to_string())
}

/// 读取服务器配置文件 (server.properties)
#[tauri::command]
pub async fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    run_blocking(move || ServerPropertiesManager::new(server_path).read()).await
}

/// 写入服务器配置文件
#[tauri::command]
pub async fn write_server_properties(
    server_path: String,
    values: BTreeMap<String, String>,
) -> Result<(), String> {
    run_blocking(move || ServerPropertiesManager::new(server_path).write(&values)).await
}

/// 读取 server.properties 原始文本
#[tauri::command]
pub async fn read_server_properties_source(server_path: String) -> Result<String, String> {
    run_blocking(move || ServerPropertiesManager::new(server_path).read_source()).await
}

/// 直接写入 server.properties 原始文本
#[tauri::command]
pub async fn write_server_properties_source(
    server_path: String,
    source: String,
) -> Result<(), String> {
    run_blocking(move || ServerPropertiesManager::new(server_path).write_source(&source)).await
}

/// 将原始文本解析为可视化配置结构
#[tauri::command]
pub async fn parse_server_properties_source(source: String) -> Result<ServerProperties, String> {
    run_blocking(move || ServerPropertiesManager::parse_source(&source)).await
}

/// 预览可视化配置写回后最终文本
#[tauri::command]
pub async fn preview_server_properties_write(
    server_path: String,
    values: BTreeMap<String, String>,
) -> Result<String, String> {
    run_blocking(move || ServerPropertiesManager::new(server_path).preview_write(&values)).await
}

/// 基于给定源码预览可视化配置写回后的最终文本
#[tauri::command]
pub async fn preview_server_properties_write_from_source(
    source: String,
    values: BTreeMap<String, String>,
) -> Result<String, String> {
    run_blocking(move || ServerPropertiesManager::preview_write_from_source(&source, &values)).await
}
