//! 系统资源信息 REST handler。
//!
//! 提供系统资源快照、默认运行路径与服务器资源占用接口，薄转发到
//! [`CoreSystemService`](sealantern_application::service::CoreSystemService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::Json;
use axum::extract::{Path, State};

use sealantern_interface::SystemService;
use sealantern_interface::system::{ServerResourceUsage, SystemSnapshot};

use super::super::error::HttpError;
use super::super::state::AppState;

/// `GET /api/system` — 采集整机系统资源快照。
pub async fn system_snapshot(
    State(state): State<AppState>,
) -> Result<Json<SystemSnapshot>, HttpError> {
    state
        .system()
        .system_snapshot()
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `GET /api/system/default-run-path` — 获取默认运行路径。
pub async fn default_run_path(State(state): State<AppState>) -> Result<Json<String>, HttpError> {
    state
        .system()
        .default_run_path()
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `GET /api/system/servers/{instance_id}/usage` — 按实例标识采集服务器资源占用。
pub async fn server_resource_usage(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
) -> Result<Json<ServerResourceUsage>, HttpError> {
    state
        .system()
        .server_resource_usage(&instance_id)
        .await
        .map(Json)
        .map_err(HttpError::from)
}
