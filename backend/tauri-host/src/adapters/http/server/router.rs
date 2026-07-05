use super::{
    api::{handle_api_command, list_api_endpoints},
    auth::{
        begin_totp_setup, build_cors_layer, confirm_totp_setup, disable_totp, get_auth_status,
        get_totp_status, initialize_browser_auth, login_browser_auth, recovery_reset_browser_auth,
        require_browser_session_auth,
    },
    event_stream::handle_runtime_event_stream,
    log_stream::handle_log_stream,
    next_bridge::{exchange_next_bridge_token, issue_next_bridge_token},
    state::{ensure_runtime_event_bridge, AppState},
    upload::handle_file_upload,
};
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
    Router,
};
use sea_lantern_runtime::log_headless_http_static_dir;
use tower_http::services::{ServeDir, ServeFile};

#[cfg(test)]
use crate::services::web_auth::WebAuthService;

#[cfg(test)]
use crate::adapters::http::command_registry::CommandRegistry;

#[cfg(test)]
use sea_lantern_runtime::HeadlessHttpConfig;

#[cfg(test)]
use std::{path::PathBuf, sync::Arc};

#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
fn unique_web_auth_state_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{}-{}.json", prefix, unique))
}

pub(super) fn build_http_app(state: AppState, static_dir: Option<String>) -> Router {
    ensure_runtime_event_bridge();
    let upload_limit = state.config.max_upload_bytes;

    let protected_routes = Router::new()
        .route("/api/auth/next-bridge/issue", post(issue_next_bridge_token))
        .route("/api/{command}", post(handle_api_command))
        .route("/api/list", get(list_api_endpoints))
        .route("/upload", post(handle_file_upload).layer(DefaultBodyLimit::max(upload_limit)))
        .route("/api/events/stream", get(handle_runtime_event_stream))
        .route("/api/logs/stream", get(handle_log_stream))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_browser_session_auth));

    let mut app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/auth/status", get(get_auth_status))
        .route("/api/auth/totp/status", get(get_totp_status))
        .route("/api/auth/setup/initialize", post(initialize_browser_auth))
        .route("/api/auth/login", post(login_browser_auth))
        .route("/api/auth/recovery/reset", post(recovery_reset_browser_auth))
        .route("/api/auth/totp/setup/begin", post(begin_totp_setup))
        .route("/api/auth/totp/setup/confirm", post(confirm_totp_setup))
        .route("/api/auth/totp/disable", post(disable_totp))
        .route("/api/auth/next-bridge/exchange", post(exchange_next_bridge_token))
        .merge(protected_routes)
        .layer(build_cors_layer(&state.config.cors_allowed_origins))
        .with_state(state);

    if let Some(dir) = static_dir {
        let index_path = format!("{}/index.html", dir);
        let serve_dir = ServeDir::new(&dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(&index_path));
        app = app.fallback_service(serve_dir);
        log_headless_http_static_dir(&dir);
    }

    app
}

#[cfg(test)]
pub(crate) fn build_test_http_app(upload_dir: PathBuf) -> Router {
    let web_auth_path = unique_web_auth_state_path("router-web-auth-state");
    build_http_app(
        AppState {
            command_registry: Arc::new(CommandRegistry::new()),
            config: Arc::new(HeadlessHttpConfig {
                auth_token: "test-token".to_string(),
                upload_dir,
                cors_allowed_origins: Vec::new(),
                max_upload_bytes: 1024 * 1024,
                max_upload_file_bytes: 512 * 1024,
                max_upload_files: 16,
            }),
            web_auth: Arc::new(WebAuthService::new_for_test(web_auth_path, Some("test-password"))),
        },
        None,
    )
}
