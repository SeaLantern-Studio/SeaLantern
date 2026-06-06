use super::{
    api::{handle_api_command, list_api_endpoints},
    auth::{build_cors_layer, require_bearer_auth},
    log_stream::handle_log_stream,
    state::AppState,
    upload::handle_file_upload,
};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};
use sea_lantern_runtime::log_headless_http_static_dir;
use tower_http::services::{ServeDir, ServeFile};

#[cfg(test)]
use crate::adapters::http::command_registry::CommandRegistry;

#[cfg(test)]
use sea_lantern_runtime::HeadlessHttpConfig;

#[cfg(test)]
use std::{path::PathBuf, sync::Arc};

pub(super) fn build_http_app(state: AppState, static_dir: Option<String>) -> Router {
    let auth_config = state.config.clone();
    let upload_limit = state.config.max_upload_bytes;

    let protected_routes = Router::new()
        .route("/api/{command}", post(handle_api_command))
        .route("/api/list", get(list_api_endpoints))
        .route("/upload", post(handle_file_upload).layer(DefaultBodyLimit::max(upload_limit)))
        .route("/api/logs/stream", get(handle_log_stream))
        .route_layer(middleware::from_fn_with_state(auth_config, require_bearer_auth));

    let mut app = Router::new()
        .route("/health", get(|| async { "OK" }))
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
        },
        None,
    )
}
