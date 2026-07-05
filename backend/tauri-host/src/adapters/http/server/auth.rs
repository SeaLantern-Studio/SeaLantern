use super::state::{ApiErrorDetail, ApiResponse, AppState};
use axum::{
    extract::{Request, State},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderMap, HeaderValue, Method, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

pub(super) fn build_cors_layer(allowed_origins: &[HeaderValue]) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    if allowed_origins.is_empty() {
        cors
    } else {
        cors.allow_origin(allowed_origins.to_vec())
    }
}

#[derive(Deserialize)]
pub(super) struct InitializeBrowserAuthRequest {
    setup_token: String,
    password: String,
}

#[derive(Deserialize)]
pub(super) struct LoginBrowserAuthRequest {
    password: String,
}

#[derive(Deserialize)]
pub(super) struct RecoveryResetRequest {
    recovery_token: String,
    new_password: String,
}

#[derive(Serialize)]
struct BrowserAuthStatusEnvelope {
    state: &'static str,
    base_state: &'static str,
    recovery_active: bool,
    setup_required: bool,
    password_login_enabled: bool,
    session: BrowserSessionMeta,
    next_bridge: NextBridgeMeta,
    totp: TotpStatusEnvelope,
}

#[derive(Serialize)]
struct BrowserSessionMeta {
    ttl_seconds: u64,
}

#[derive(Serialize)]
struct NextBridgeMeta {
    enabled: bool,
    exchange_ttl_seconds: u64,
}

#[derive(Serialize)]
pub(super) struct TotpStatusEnvelope {
    state: &'static str,
    required_on_login: bool,
    can_setup: bool,
    can_disable: bool,
}

#[derive(Serialize)]
pub(super) struct BrowserSessionResponse {
    pub session_token: String,
    pub token: String,
    pub expires_at: u64,
    pub purpose: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<&'static str>,
}

pub(super) async fn require_browser_session_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(token) = extract_bearer_token(request.headers()) {
        if state.web_auth.is_valid_browser_session_token(token) {
            return next.run(request).await;
        }
    }

    unauthorized_response()
}

pub(super) async fn get_auth_status(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.web_auth.auth_status();
    let payload = BrowserAuthStatusEnvelope {
        state: snapshot.state,
        base_state: snapshot.base_state,
        recovery_active: snapshot.recovery_active,
        setup_required: snapshot.setup_required,
        password_login_enabled: snapshot.password_login_enabled,
        session: BrowserSessionMeta {
            ttl_seconds: snapshot.session_ttl_seconds,
        },
        next_bridge: NextBridgeMeta {
            enabled: true,
            exchange_ttl_seconds: snapshot.next_bridge_exchange_ttl_seconds,
        },
        totp: reserved_totp_status(),
    };

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(payload)))).into_response()
}

pub(super) async fn get_totp_status() -> impl IntoResponse {
    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(reserved_totp_status()))))
        .into_response()
}

pub(super) async fn begin_totp_setup() -> impl IntoResponse {
    reserved_totp_not_implemented_response()
}

pub(super) async fn confirm_totp_setup() -> impl IntoResponse {
    reserved_totp_not_implemented_response()
}

pub(super) async fn disable_totp() -> impl IntoResponse {
    reserved_totp_not_implemented_response()
}

pub(super) async fn initialize_browser_auth(
    State(state): State<AppState>,
    Json(payload): Json<InitializeBrowserAuthRequest>,
) -> impl IntoResponse {
    match state
        .web_auth
        .initialize_password(&payload.setup_token, &payload.password)
    {
        Ok(session) => success_session_response(session, Some("initialized")),
        Err(error) => auth_error_response(error.kind, &error.message),
    }
}

pub(super) async fn login_browser_auth(
    State(state): State<AppState>,
    Json(payload): Json<LoginBrowserAuthRequest>,
) -> impl IntoResponse {
    match state.web_auth.login(&payload.password) {
        Ok(session) => success_session_response(session, None),
        Err(error) => auth_error_response(error.kind, &error.message),
    }
}

pub(super) async fn recovery_reset_browser_auth(
    State(state): State<AppState>,
    Json(payload): Json<RecoveryResetRequest>,
) -> impl IntoResponse {
    match state
        .web_auth
        .recovery_reset(&payload.recovery_token, &payload.new_password)
    {
        Ok(session) => success_session_response(session, Some("initialized")),
        Err(error) => auth_error_response(error.kind, &error.message),
    }
}

fn success_session_response(
    session: crate::services::web_auth::BrowserSessionIssue,
    state: Option<&'static str>,
) -> Response {
    let payload = BrowserSessionResponse {
        token: session.session_token.clone(),
        session_token: session.session_token,
        expires_at: session.expires_at,
        purpose: session.purpose,
        state,
    };
    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(payload)))).into_response()
}

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = header_value.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then_some(token)
}

fn auth_error_response(kind: crate::services::web_auth::WebAuthError, message: &str) -> Response {
    let (status, code, error_kind) = match kind {
        crate::services::web_auth::WebAuthError::Unauthorized => {
            (StatusCode::UNAUTHORIZED, "common.message_unauthorized", "unauthorized")
        }
        crate::services::web_auth::WebAuthError::InvalidRequest => {
            (StatusCode::BAD_REQUEST, "common.message_unknown_error", "invalid_request")
        }
        crate::services::web_auth::WebAuthError::Conflict => {
            (StatusCode::CONFLICT, "common.message_unknown_error", "invalid_request")
        }
        crate::services::web_auth::WebAuthError::Unavailable => {
            (StatusCode::INTERNAL_SERVER_ERROR, "common.message_unknown_error", "runtime")
        }
    };

    (
        status,
        Json(ApiResponse::error_with_detail(
            message.to_string(),
            ApiErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                args: None,
                error_kind: Some(error_kind.to_string()),
            },
        )),
    )
        .into_response()
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiResponse::error_with_detail(
            "Unauthorized".to_string(),
            ApiErrorDetail {
                code: "common.message_unauthorized".to_string(),
                message: "Unauthorized".to_string(),
                args: None,
                error_kind: Some("unauthorized".to_string()),
            },
        )),
    )
        .into_response()
}

fn reserved_totp_status() -> TotpStatusEnvelope {
    TotpStatusEnvelope {
        state: "reserved",
        required_on_login: false,
        can_setup: false,
        can_disable: false,
    }
}

fn reserved_totp_not_implemented_response() -> Response {
    let message = "TOTP endpoint is reserved and not implemented";
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiResponse::error_with_detail(
            message.to_string(),
            ApiErrorDetail {
                code: "common.message_unknown_error".to_string(),
                message: message.to_string(),
                args: None,
                error_kind: Some("not_implemented".to_string()),
            },
        )),
    )
        .into_response()
}
