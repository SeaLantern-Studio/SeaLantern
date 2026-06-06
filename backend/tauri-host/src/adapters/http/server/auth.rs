use super::state::{ApiErrorDetail, ApiResponse};
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
    if extract_bearer_token_for_config(&config, request.headers()).is_some() {
        next.run(request).await
    } else {
        unauthorized_response()
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = header_value.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then_some(token)
}

fn extract_bearer_token_for_config<'a>(
    config: &'a HeadlessHttpConfig,
    headers: &'a HeaderMap,
) -> Option<&'a str> {
    extract_bearer_token(headers).filter(|token| *token == config.auth_token.as_str())
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
