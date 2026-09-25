//! 实例启动配置（SeaLantern/config.toml）REST handler。
//!
//! 提供实例级启动配置覆盖的读取与写入，薄转发到
//! [`ServerStartupService`](sealantern_application::port::ServerStartupService)。
//!
//! 端点以实例标识定位（`/instances/{id}/startup-config`），服务器目录由应用层
//! 解析，因此 HTTP 调用方无法指定任意路径。

use axum::Json;
use axum::extract::{Path, State};

use sealantern_application::port::ServerStartupService;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_core::instance::InstanceId;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 解析路径参数中的实例 ID，非法输入视为客户端错误。
fn parse_id(raw: &str) -> Result<InstanceId, HttpError> {
    InstanceId::new(raw.to_owned())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// `GET /api/instances/{id}/startup-config` — 读取启动配置覆盖。
pub async fn read_startup_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SLStartupConfig>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_startup()
        .read(&id)
        .await
        .map(Json)
        .map_err(HttpError::from_server_startup_error)
}

/// `PUT /api/instances/{id}/startup-config` — 写入启动配置覆盖。
pub async fn write_startup_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(config): Json<SLStartupConfig>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server_startup()
        .write(&id, &config)
        .await
        .map(Json)
        .map_err(HttpError::from_server_startup_error)
}
