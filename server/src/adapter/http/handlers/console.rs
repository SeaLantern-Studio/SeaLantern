//! 服务器控制台日志 REST handler。
//!
//! 提供服务器控制台日志的增量读取接口，薄转发到
//! [`CoreConsoleService`](sealantern_application::service::CoreConsoleService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use sealantern_core::instance::InstanceId;
use sealantern_interface::console::ConsoleLogLine;
use sealantern_interface::ConsoleService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 日志读取的查询参数。
#[derive(Debug, Default, Deserialize)]
pub struct ConsoleLogQuery {
    /// 起始行号游标（默认 0）。
    pub since: Option<i64>,
    /// 最近 N 行窗口（缺省返回全部）。
    pub limit: Option<i64>,
}

/// 解析路径参数中的实例 ID，非法输入视为客户端错误。
fn parse_id(raw: &str) -> Result<InstanceId, HttpError> {
    InstanceId::new(raw.to_owned())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// `GET /api/instances/{id}/logs?since=&limit=` — 读取服务器控制台日志。
pub async fn console_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ConsoleLogQuery>,
) -> Result<Json<Vec<ConsoleLogLine>>, HttpError> {
    let id = parse_id(&id)?;
    state
        .console()
        .logs(&id, query.since.unwrap_or(0), query.limit)
        .await
        .map(Json)
        .map_err(HttpError::from)
}
