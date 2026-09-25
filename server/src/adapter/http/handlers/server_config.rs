//! 服务器配置（server.properties）REST handler。
//!
//! 提供 `server.properties` 的可视化结构读写、原始文本读写、解析与写入预览接口，
//! 薄转发到 [`ServerConfigService`](sealantern_application::port::ServerConfigService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。
//!
//! 按实例定位的端点只接受实例标识：服务器目录由应用层从实例注册表解析，
//! 因此调用方无法借路径参数访问实例目录之外的文件。不涉及实例的解析与预览
//! 是纯文本变换，直接接受调用方给出的源码。

use std::collections::BTreeMap;
use std::path::PathBuf;

use axum::Json;
use axum::extract::{Path, State};

use sealantern_application::port::ServerConfigService;
use sealantern_application::service::resolve_instance_directory;
use sealantern_contract::server_config::ServerProperties;
use sealantern_core::instance::InstanceId;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 解析路径参数中的实例 ID，非法输入视为客户端错误。
fn parse_id(raw: &str) -> Result<InstanceId, HttpError> {
    InstanceId::new(raw.to_owned())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// 将实例标识解析为服务器目录。
///
/// 返回 [`PathBuf`] 而不是字符串：实例目录在类 Unix 系统上允许包含非 UTF-8 字节，
/// 转成字符串会有损，后续拼接出的路径将指向别处。
async fn resolve_directory(state: &AppState, id: &InstanceId) -> Result<PathBuf, HttpError> {
    resolve_instance_directory(state.instance().as_ref(), id)
        .await
        .map_err(HttpError::from)?
        .ok_or_else(|| HttpError::not_found("instance_not_found", "server instance not found"))
}

/// 按键值对写入配置的请求体。
#[derive(Debug, serde::Deserialize)]
pub struct WritePropertiesRequest {
    /// 待写入的键值对；仅这些键被替换，注释与其余行保持不变。
    pub values: BTreeMap<String, String>,
}

/// 仅携带原始文本的请求体。
#[derive(Debug, serde::Deserialize)]
pub struct SourceRequest {
    /// `server.properties` 原始全文。
    pub source: String,
}

/// 基于给定源码预览写入的请求体。
#[derive(Debug, serde::Deserialize)]
pub struct PreviewFromSourceRequest {
    /// 作为预览基准的草稿全文。
    pub source: String,
    /// 待写入的键值对。
    pub values: BTreeMap<String, String>,
}

/// `GET /api/instances/{id}/server-properties` — 读取服务器配置文件（可视化结构）。
pub async fn read_server_properties(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServerProperties>, HttpError> {
    let id = parse_id(&id)?;
    let directory = resolve_directory(&state, &id).await?;
    state
        .server_config()
        .read(&directory)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}

/// `PUT /api/instances/{id}/server-properties` — 按键值对写入配置（保留注释与顺序）。
pub async fn write_server_properties(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<WritePropertiesRequest>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    let directory = resolve_directory(&state, &id).await?;
    state
        .server_config()
        .write(&directory, &request.values)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}

/// `GET /api/instances/{id}/server-properties/source` — 读取配置文件原始文本。
pub async fn read_server_properties_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<String>, HttpError> {
    let id = parse_id(&id)?;
    let directory = resolve_directory(&state, &id).await?;
    state
        .server_config()
        .read_source(&directory)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}

/// `PUT /api/instances/{id}/server-properties/source` — 写入配置文件原始文本。
pub async fn write_server_properties_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<SourceRequest>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    let directory = resolve_directory(&state, &id).await?;
    state
        .server_config()
        .write_source(&directory, &request.source)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}

/// `POST /api/instances/{id}/server-properties/preview` — 预览按键值写入后的最终文本。
pub async fn preview_server_properties_write(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<WritePropertiesRequest>,
) -> Result<Json<String>, HttpError> {
    let id = parse_id(&id)?;
    let directory = resolve_directory(&state, &id).await?;
    state
        .server_config()
        .preview_write(&directory, &request.values)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}

/// `POST /api/server-properties/parse` — 将原始文本解析为可视化结构（纯函数）。
pub async fn parse_server_properties_source(
    State(state): State<AppState>,
    Json(request): Json<SourceRequest>,
) -> Result<Json<ServerProperties>, HttpError> {
    state
        .server_config()
        .parse_source(&request.source)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}

/// `POST /api/server-properties/preview` — 基于给定源码预览写入结果（纯函数）。
pub async fn preview_server_properties_write_from_source(
    State(state): State<AppState>,
    Json(request): Json<PreviewFromSourceRequest>,
) -> Result<Json<String>, HttpError> {
    state
        .server_config()
        .preview_write_from_source(&request.source, &request.values)
        .await
        .map(Json)
        .map_err(HttpError::from_server_config_error)
}
