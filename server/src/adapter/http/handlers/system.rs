//! 系统资源信息 REST handler。
//!
//! 提供系统资源快照、进程资源与目录磁盘占用接口，薄转发到
//! [`CoreSystemService`](sealantern_application::service::CoreSystemService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::extract::{Path, State};
use axum::Json;

use sealantern_interface::system::{DirectoryUsage, ProcessResourceUsage, SystemSnapshot};
use sealantern_interface::SystemService;

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

/// `GET /api/system/process/{pid}` — 采集指定进程的资源使用。
pub async fn process_usage(
    State(state): State<AppState>,
    Path(pid): Path<u32>,
) -> Result<Json<ProcessResourceUsage>, HttpError> {
    state
        .system()
        .process_usage(pid)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `GET /api/system/directory/*path` — 计算指定目录的磁盘占用。
///
/// 路径使用通配符捕获，支持任意多段路径
/// （如 `/api/system/directory/var/log`、`/api/system/directory/C:%5Cservers%5Cserver-a`）。
pub async fn directory_usage(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<DirectoryUsage>, HttpError> {
    let path = std::path::Path::new(&path);
    state
        .system()
        .directory_usage(path)
        .await
        .map(Json)
        .map_err(HttpError::from)
}
