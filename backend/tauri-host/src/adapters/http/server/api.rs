use super::state::{ApiErrorDetail, ApiRequest, ApiResponse, AppState};
use super::upload::resolve_uploaded_value_references;
use crate::utils::logger;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_lantern_runtime::{is_supported_http_command, DispatchResult};

fn upload_reference_error_kind_for_status(status: StatusCode) -> &'static str {
    if status.is_server_error() {
        "runtime"
    } else {
        "invalid_request"
    }
}

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
    logger::log_debug_ctx(
        "http.api",
        "handle_api_command",
        &format!("received command={}", command),
    );

    let params =
        match resolve_uploaded_value_references(&state.config.upload_dir, payload.params).await {
            Ok(params) => params,
            Err(error) => {
                logger::log_warn_ctx(
                    "http.api",
                    "handle_api_command",
                    &format!("command={} request rejected: {}", command, error.message),
                );
                return (
                    error.status,
                    Json(ApiResponse::error_with_detail(
                        error.message.clone(),
                        ApiErrorDetail {
                            code: "common.message_unknown_error".to_string(),
                            message: error.message,
                            args: None,
                            error_kind: Some(
                                upload_reference_error_kind_for_status(error.status).to_string(),
                            ),
                        },
                    )),
                )
                    .into_response();
            }
        };

    match state.command_registry.dispatch(&command, params).await {
        DispatchResult::NotFound(message) => {
            logger::log_warn_ctx(
                "http.api",
                "handle_api_command",
                &format!("command={} not found: {}", command, message),
            );
            (StatusCode::NOT_FOUND, Json(ApiResponse::error(message))).into_response()
        }
        DispatchResult::Success(data) => {
            logger::log_info_ctx(
                "http.api",
                "handle_api_command",
                &format!("command={} succeeded", command),
            );
            (StatusCode::OK, Json(ApiResponse::success(data))).into_response()
        }
        DispatchResult::Failure(error) => {
            logger::log_error_ctx(
                "http.api",
                "handle_api_command",
                &format!("command={} failed: {}", command, error),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(error))).into_response()
        }
    }
}
