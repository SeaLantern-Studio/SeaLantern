use super::state::ApiResponse;
use axum::{
    Json,
    extract::{Request, State},
    http::{
        HeaderMap, HeaderValue, Method, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use sea_lantern_runtime::HeadlessHttpConfig;
use std::sync::Arc;
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

pub(super) async fn require_bearer_auth(
    State(config): State<Arc<HeadlessHttpConfig>>,
    request: Request,
    next: Next,
) -> Response {
    match extract_bearer_token(request.headers()) {
        Some(token) if token == config.auth_token.as_str() => next.run(request).await,
        _ => unauthorized_response(),
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = header_value.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then_some(token)
}

fn unauthorized_response() -> Response {
    (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("Unauthorized".to_string()))).into_response()
}
