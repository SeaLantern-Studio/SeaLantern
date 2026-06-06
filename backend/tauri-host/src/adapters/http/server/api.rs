use super::state::{ApiRequest, ApiResponse, AppState};
use crate::utils::logger::capture_eprintln;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_lantern_runtime::{DispatchResult, is_supported_http_command};

/// 返回当前可用的命令列表
pub(super) async fn list_api_endpoints(State(state): State<AppState>) -> impl IntoResponse {
    let endpoints = state.command_registry.list_commands();

    let supported: Vec<&String> = endpoints
        .iter()
        .filter(|cmd: &&String| is_supported_http_command(cmd))
        .collect();

    Json(ApiResponse::success(serde_json::json!({
        "endpoints": endpoints,
        "supported_count": supported.len(),
        "note": "Some plugin commands are supported in HTTP mode, but install and enable flows are still limited",
        "usage": "POST /api/{command} with JSON body {\"params\": {...}}"
    })))
}

/// 处理 `/api/{command}` 调用
pub(super) async fn handle_api_command(
    Path(command): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ApiRequest>,
) -> impl IntoResponse {
    capture_eprintln(format!("[HTTP API] Received command: {}", command));

    match state.command_registry.dispatch(&command, payload.params).await {
        DispatchResult::NotFound(message) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(message)),
        )
            .into_response(),
        DispatchResult::Success(data) => {
            capture_eprintln(format!("[HTTP API] Command '{}' succeeded", command));
            (StatusCode::OK, Json(ApiResponse::success(data))).into_response()
        }
        DispatchResult::Failure(error) => {
            capture_eprintln(format!("[HTTP API] Command '{}' failed: {}", command, error));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(error))).into_response()
        }
    }
}
