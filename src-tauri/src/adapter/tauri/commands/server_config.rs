//! 服务器配置管理 Tauri 命令。

use std::collections::BTreeMap;

use sealantern_application::port::ServerConfigService;
use sealantern_application::services::AppServices;
use sealantern_contract::ServerConfigServiceError;
use sealantern_contract::server_config::ServerProperties;
use tauri::State;

/// 读取服务器配置文件 (server.properties)
#[tauri::command]
pub async fn read_server_properties(
    services: State<'_, AppServices>,
    server_path: String,
) -> Result<ServerProperties, ServerConfigServiceError> {
    services.server_config().read(&server_path).await
}

/// 写入服务器配置文件
#[tauri::command]
pub async fn write_server_properties(
    services: State<'_, AppServices>,
    server_path: String,
    values: BTreeMap<String, String>,
) -> Result<(), ServerConfigServiceError> {
    services.server_config().write(&server_path, &values).await
}

/// 读取 server.properties 原始文本
#[tauri::command]
pub async fn read_server_properties_source(
    services: State<'_, AppServices>,
    server_path: String,
) -> Result<String, ServerConfigServiceError> {
    services.server_config().read_source(&server_path).await
}

/// 直接写入 server.properties 原始文本
#[tauri::command]
pub async fn write_server_properties_source(
    services: State<'_, AppServices>,
    server_path: String,
    source: String,
) -> Result<(), ServerConfigServiceError> {
    services
        .server_config()
        .write_source(&server_path, &source)
        .await
}

/// 将原始文本解析为可视化配置结构
#[tauri::command]
pub async fn parse_server_properties_source(
    services: State<'_, AppServices>,
    source: String,
) -> Result<ServerProperties, ServerConfigServiceError> {
    services.server_config().parse_source(&source).await
}

/// 预览可视化配置写回后最终文本
#[tauri::command]
pub async fn preview_server_properties_write(
    services: State<'_, AppServices>,
    server_path: String,
    values: BTreeMap<String, String>,
) -> Result<String, ServerConfigServiceError> {
    services
        .server_config()
        .preview_write(&server_path, &values)
        .await
}

/// 基于给定源码预览可视化配置写回后的最终文本
#[tauri::command]
pub async fn preview_server_properties_write_from_source(
    services: State<'_, AppServices>,
    source: String,
    values: BTreeMap<String, String>,
) -> Result<String, ServerConfigServiceError> {
    services
        .server_config()
        .preview_write_from_source(&source, &values)
        .await
}
