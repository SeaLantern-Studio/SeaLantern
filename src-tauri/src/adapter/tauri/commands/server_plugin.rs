//! 服务器插件（plugins 目录）管理 Tauri 命令。
//!
//! 命令只接受实例标识：服务器目录由应用层从实例发现结果解析，因此前端
//! 无法借文件名参数访问实例目录之外的位置。
//!
//! 命令名与参数保持前端既有的 `m_*` / snake_case 形态——前端插件管理尚未
//! 迁移到统一契约，这里按最小改动补齐后端；后续重构再统一命名。

use sealantern_application::port::ServerPluginService;
use sealantern_application::services::AppServices;
use sealantern_contract::ServerPluginServiceError;
use sealantern_contract::server_plugin::{PluginConfigFile, PluginSummary};
use sealantern_core::instance::InstanceId;
use tauri::State;

/// 解析前端传入的实例标识，非法输入视为客户端错误。
fn parse_id(raw: String) -> Result<InstanceId, ServerPluginServiceError> {
    InstanceId::new(raw).map_err(|_| ServerPluginServiceError::InvalidInput)
}

/// 列出实例的全部服务器插件（含已禁用）。
#[tauri::command(rename_all = "snake_case")]
pub async fn m_get_plugins(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<Vec<PluginSummary>, ServerPluginServiceError> {
    let id = parse_id(server_id)?;
    services.server_plugin().list(&id).await
}

/// 读取某个插件配置目录下的文本文件。
#[tauri::command(rename_all = "snake_case")]
pub async fn m_get_plugin_config_files(
    services: State<'_, AppServices>,
    server_id: String,
    file_name: String,
    plugin_name: String,
) -> Result<Vec<PluginConfigFile>, ServerPluginServiceError> {
    let id = parse_id(server_id)?;
    services
        .server_plugin()
        .read_config_files(&id, &file_name, &plugin_name)
        .await
}

/// 启用或禁用一个插件。
#[tauri::command(rename_all = "snake_case")]
pub async fn m_toggle_plugin(
    services: State<'_, AppServices>,
    server_id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), ServerPluginServiceError> {
    let id = parse_id(server_id)?;
    services
        .server_plugin()
        .set_enabled(&id, &file_name, enabled)
        .await
}

/// 永久删除一个插件。
#[tauri::command(rename_all = "snake_case")]
pub async fn m_delete_plugin(
    services: State<'_, AppServices>,
    server_id: String,
    file_name: String,
) -> Result<(), ServerPluginServiceError> {
    let id = parse_id(server_id)?;
    services.server_plugin().delete(&id, &file_name).await
}

/// 安装一个插件：字节内容由前端一次性给出。
#[tauri::command(rename_all = "snake_case")]
pub async fn m_install_plugin(
    services: State<'_, AppServices>,
    server_id: String,
    file_name: String,
    file_data: Vec<u8>,
) -> Result<(), ServerPluginServiceError> {
    let id = parse_id(server_id)?;
    services
        .server_plugin()
        .install(&id, &file_name, &file_data)
        .await
}
