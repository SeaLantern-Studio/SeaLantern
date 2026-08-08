//! 服务器进程管理 REST handler。
//!
//! 提供服务器进程生命周期（状态/启动/停止/强制停止/控制台命令）接口，
//! 薄转发到 [`CoreServerService`](sealantern_application::service::CoreServerService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::extract::{Path, State};
use axum::Json;

use sealantern_core::instance::InstanceId;
use sealantern_interface::server::ServerSnapshot;
use sealantern_interface::ServerService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 解析路径参数中的实例 ID，非法输入视为客户端错误。
fn parse_id(raw: &str) -> Result<InstanceId, HttpError> {
    InstanceId::new(raw.to_owned())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// `GET /api/instances/{id}/status` — 查询服务器进程状态。
pub async fn server_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServerSnapshot>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server()
        .status(&id)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/instances/{id}/start` — 启动服务器进程。
pub async fn start_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServerSnapshot>, HttpError> {
    let id = parse_id(&id)?;
    state.server().start(&id).await?;
    state
        .server()
        .status(&id)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/instances/{id}/stop` — 优雅停止服务器进程。
pub async fn stop_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServerSnapshot>, HttpError> {
    let id = parse_id(&id)?;
    state.server().stop(&id).await?;
    state
        .server()
        .status(&id)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/instances/{id}/force-stop` — 强制停止服务器进程。
pub async fn force_stop_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServerSnapshot>, HttpError> {
    let id = parse_id(&id)?;
    state.server().force_stop(&id).await?;
    state
        .server()
        .status(&id)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// 控制台命令请求体。
#[derive(Debug, serde::Deserialize)]
pub struct SendCommandRequest {
    /// 要发送的控制台命令。
    pub command: String,
}

/// `POST /api/instances/{id}/command` — 向服务器控制台发送命令。
pub async fn send_server_command(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<SendCommandRequest>,
) -> Result<Json<()>, HttpError> {
    let id = parse_id(&id)?;
    state
        .server()
        .send_command(&id, &request.command)
        .await
        .map(Json)
        .map_err(HttpError::from)
}
