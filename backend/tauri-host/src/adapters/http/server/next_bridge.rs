use super::{
    auth::BrowserSessionResponse,
    state::{ApiErrorDetail, ApiResponse, AppState},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

const NEXT_BRIDGE_PURPOSE: &str = "next_bridge";
const DEFAULT_NEXT_TARGET_PATH: &str = "/settings";

#[derive(Deserialize)]
pub(super) struct IssueNextBridgeTokenRequest {
    #[serde(default)]
    target_path: Option<String>,
}

#[derive(Serialize)]
struct IssueNextBridgeTokenResponse {
    bridge_token: String,
    expires_at: u64,
    purpose: &'static str,
    target_path: String,
}

#[derive(Deserialize)]
pub(super) struct ExchangeNextBridgeTokenRequest {
    bridge_token: String,
}

fn sanitize_next_target_path(raw: Option<&str>) -> Result<String, String> {
    let trimmed = raw.map(str::trim).filter(|value| !value.is_empty());
    let path = trimmed.unwrap_or(DEFAULT_NEXT_TARGET_PATH);

    if !path.starts_with('/') || path.starts_with("//") {
        return Err("next bridge target path must stay within the current origin".to_string());
    }

    Ok(path.to_string())
}

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
    code: &str,
    error_kind: &str,
) -> Response {
    let message = message.into();
    (
        status,
        Json(ApiResponse::error_with_detail(
            message.clone(),
            ApiErrorDetail {
                code: code.to_string(),
                message,
                args: None,
                error_kind: Some(error_kind.to_string()),
            },
        )),
    )
        .into_response()
}

pub(super) async fn issue_next_bridge_token(
    State(state): State<AppState>,
    Json(payload): Json<IssueNextBridgeTokenRequest>,
) -> impl IntoResponse {
    let target_path = match sanitize_next_target_path(payload.target_path.as_deref()) {
        Ok(path) => path,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                error,
                "common.message_unknown_error",
                "invalid_request",
            )
        }
    };

    let issued = state.web_auth.issue_next_bridge_token();
    let payload = IssueNextBridgeTokenResponse {
        bridge_token: issued.bridge_token,
        expires_at: issued.expires_at,
        purpose: NEXT_BRIDGE_PURPOSE,
        target_path,
    };

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(payload)))).into_response()
}

pub(super) async fn exchange_next_bridge_token(
    State(state): State<AppState>,
    Json(payload): Json<ExchangeNextBridgeTokenRequest>,
) -> impl IntoResponse {
    match state
        .web_auth
        .exchange_next_bridge_token(&payload.bridge_token)
    {
        Ok(session) => {
            let payload = BrowserSessionResponse {
                token: session.session_token.clone(),
                session_token: session.session_token,
                expires_at: session.expires_at,
                purpose: session.purpose,
                state: None,
            };
            (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(payload)))).into_response()
        }
        Err(error) => match error.kind {
            crate::services::web_auth::WebAuthError::InvalidRequest => error_response(
                StatusCode::BAD_REQUEST,
                error.message,
                "common.message_unknown_error",
                "invalid_request",
            ),
            crate::services::web_auth::WebAuthError::Unauthorized => error_response(
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                "common.message_unauthorized",
                "unauthorized",
            ),
            crate::services::web_auth::WebAuthError::Conflict => error_response(
                StatusCode::CONFLICT,
                error.message,
                "common.message_unknown_error",
                "invalid_request",
            ),
            crate::services::web_auth::WebAuthError::Unavailable => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                error.message,
                "common.message_unknown_error",
                "runtime",
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_next_target_path;

    #[test]
    fn sanitize_next_target_path_defaults_to_settings() {
        assert_eq!(sanitize_next_target_path(None).unwrap(), "/settings");
        assert!(sanitize_next_target_path(Some("https://example.com/settings")).is_err());
        assert!(sanitize_next_target_path(Some("//example.com/settings")).is_err());
    }
}
