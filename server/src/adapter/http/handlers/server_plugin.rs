//! 服务器插件（plugins 目录）REST handler。
//!
//! 提供插件列举、配置读取、启用/禁用、删除与安装，薄转发到
//! [`ServerPluginService`](sealantern_application::port::ServerPluginService)。
//!
//! 所有端点都以实例标识定位（`/instances/{id}/plugins[...]`），服务器目录由
//! 应用层解析，因此 HTTP 调用方无法借文件名参数访问实例目录之外的位置。

use axum::Json;
use axum::extract::{Path, Query, State};

use sealantern_application::port::ServerPluginService;
use sealantern_contract::server_plugin::{PluginConfigFile, PluginSummary};
use sealantern_core::instance::InstanceId;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 解析路径参数中的实例 ID，非法输入视为客户端错误。
fn parse_id(raw: &str) -> Result<InstanceId, HttpError> {
    InstanceId::new(raw.to_owned())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// `GET /api/instances/{id}/plugins` — 列出实例的全部服务器插件。
pub async fn list_server_plugins(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<PluginSummary>>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_plugin()
        .list(&id)
        .await
        .map(Json)
        .map_err(HttpError::from_server_plugin_error)
}

/// 读取插件配置文件的查询参数。
#[derive(Debug, serde::Deserialize)]
pub struct ConfigFilesQuery {
    /// jar 文件名；与 Tauri 命令契约保持一致，当前仅用于定位调用。
    pub file_name: String,
    /// 插件声明的名称，即 `plugins/<name>/` 配置目录名。
    pub plugin_name: String,
}

/// `GET /api/instances/{id}/plugins/config-files` — 读取插件配置目录下的文本文件。
pub async fn read_server_plugin_config_files(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ConfigFilesQuery>,
) -> Result<Json<Vec<PluginConfigFile>>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_plugin()
        .read_config_files(&id, &query.file_name, &query.plugin_name)
        .await
        .map(Json)
        .map_err(HttpError::from_server_plugin_error)
}

/// 启用或禁用插件的请求体。
#[derive(Debug, serde::Deserialize)]
pub struct SetEnabledRequest {
    /// jar 文件名，可带 `.disabled` 后缀。
    pub file_name: String,
    /// 目标状态：`true` 启用，`false` 禁用。
    pub enabled: bool,
}

/// `PUT /api/instances/{id}/plugins/enabled` — 启用或禁用一个插件。
pub async fn set_server_plugin_enabled(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<SetEnabledRequest>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_plugin()
        .set_enabled(&id, &request.file_name, request.enabled)
        .await
        .map(Json)
        .map_err(HttpError::from_server_plugin_error)
}

/// 删除插件的查询参数。
#[derive(Debug, serde::Deserialize)]
pub struct DeletePluginQuery {
    /// jar 文件名，可带 `.disabled` 后缀。
    pub file_name: String,
}

/// `DELETE /api/instances/{id}/plugins` — 永久删除一个插件。
pub async fn delete_server_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DeletePluginQuery>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_plugin()
        .delete(&id, &query.file_name)
        .await
        .map(Json)
        .map_err(HttpError::from_server_plugin_error)
}

/// 安装插件的请求体。
#[derive(Debug, serde::Deserialize)]
pub struct InstallPluginRequest {
    /// jar 文件名。
    pub file_name: String,
    /// jar 字节内容，由前端一次性给出。
    pub file_data: Vec<u8>,
}

/// `POST /api/instances/{id}/plugins` — 安装一个插件。
pub async fn install_server_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<InstallPluginRequest>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_plugin()
        .install(&id, &request.file_name, &request.file_data)
        .await
        .map(Json)
        .map_err(HttpError::from_server_plugin_error)
}
