//! HTTP 服务入口
//!
//! 这里负责启动 Axum 服务，并把认证、路由、上传、日志流等细节委托给子模块。

#[path = "server/api.rs"]
mod api;
#[path = "server/auth.rs"]
mod auth;
#[path = "server/event_stream.rs"]
mod event_stream;
#[path = "server/log_stream.rs"]
mod log_stream;
#[path = "server/next_bridge.rs"]
mod next_bridge;
#[path = "server/router.rs"]
mod router;
#[path = "server/state.rs"]
mod state;
#[path = "server/upload.rs"]
mod upload;

use super::command_registry::CommandRegistry;
use crate::services::web_auth::WebAuthService;
use crate::utils::logger::log_error_ctx;
use router::build_http_app;
use sea_lantern_runtime::{
    default_headless_http_config_checked, log_headless_http_ready, prepare_headless_http_listener,
    HeadlessHttpConfig,
};
use state::AppState;
use std::sync::Arc;

pub use state::{get_log_sender, LogEvent};

#[cfg(test)]
pub(crate) use router::build_test_http_app;

#[cfg(test)]
use state::ApiResponse;

fn default_http_server_config() -> Result<HeadlessHttpConfig, String> {
    default_headless_http_config_checked()
}

/// 启动 HTTP 服务
pub async fn run_http_server(
    addr: &str,
    static_dir: Option<String>,
    startup_notifier: Option<std::sync::mpsc::Sender<Result<(), String>>>,
) -> Result<(), String> {
    let config = Arc::new(default_http_server_config()?);
    let state = AppState {
        command_registry: Arc::new(CommandRegistry::new()),
        config: config.clone(),
        web_auth: Arc::new(WebAuthService::new()),
    };
    let listener = prepare_headless_http_listener(addr, config.as_ref(), startup_notifier).await?;

    let app = build_http_app(state, static_dir);
    log_headless_http_ready(addr);

    if let Err(error) = axum::serve(listener, app).await {
        let message = format!("SeaLantern HTTP server error on {}: {}", addr, error);
        log_error_ctx("http.server", "run_http_server", &message);
        return Err(message);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_http_app, build_test_http_app, default_http_server_config, ApiResponse, AppState,
        CommandRegistry,
    };
    use crate::services::web_auth::WebAuthService;
    use crate::test_support::{lock_env, EnvGuard};
    use crate::utils::logger::GLOBAL_LOG_COLLECTOR;
    use axum::{
        body::Body,
        http::{self, header, Request, StatusCode},
    };
    use sea_lantern_runtime::HeadlessHttpConfig;
    use serde_json::Value;
    use std::{
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tower::ServiceExt;

    use super::upload::{
        build_unique_saved_name, build_upload_reference, build_upload_target_path,
        sanitize_upload_basename,
    };

    fn unique_web_auth_state_path(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}.json", prefix, unique))
    }

    fn test_state(upload_dir: PathBuf) -> AppState {
        test_state_with_password(upload_dir, Some("test-password"))
    }

    fn test_state_with_password(upload_dir: PathBuf, seeded_password: Option<&str>) -> AppState {
        let web_auth_path = unique_web_auth_state_path("server-web-auth-state");
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
            web_auth: Arc::new(WebAuthService::new_for_test(web_auth_path, seeded_password)),
        }
    }

    fn test_app(upload_dir: PathBuf) -> axum::Router {
        build_http_app(test_state(upload_dir), None)
    }

    fn test_app_with_password(upload_dir: PathBuf, seeded_password: Option<&str>) -> axum::Router {
        build_http_app(test_state_with_password(upload_dir, seeded_password), None)
    }

    fn test_app_with_web_auth(upload_dir: PathBuf, web_auth: Arc<WebAuthService>) -> axum::Router {
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
                web_auth,
            },
            None,
        )
    }

    fn test_app_with_limits(
        upload_dir: PathBuf,
        max_upload_bytes: usize,
        max_upload_files: usize,
    ) -> axum::Router {
        let web_auth_path = unique_web_auth_state_path("server-web-auth-state");
        build_http_app(
            AppState {
                command_registry: Arc::new(CommandRegistry::new()),
                config: Arc::new(HeadlessHttpConfig {
                    auth_token: "test-token".to_string(),
                    upload_dir,
                    cors_allowed_origins: Vec::new(),
                    max_upload_bytes,
                    max_upload_file_bytes: max_upload_bytes,
                    max_upload_files,
                }),
                web_auth: Arc::new(WebAuthService::new_for_test(
                    web_auth_path,
                    Some("test-password"),
                )),
            },
            None,
        )
    }

    fn test_app_with_upload_caps(
        upload_dir: PathBuf,
        max_upload_bytes: usize,
        max_upload_file_bytes: usize,
        max_upload_files: usize,
    ) -> axum::Router {
        let web_auth_path = unique_web_auth_state_path("server-web-auth-state");
        build_http_app(
            AppState {
                command_registry: Arc::new(CommandRegistry::new()),
                config: Arc::new(HeadlessHttpConfig {
                    auth_token: "test-token".to_string(),
                    upload_dir,
                    cors_allowed_origins: Vec::new(),
                    max_upload_bytes,
                    max_upload_file_bytes,
                    max_upload_files,
                }),
                web_auth: Arc::new(WebAuthService::new_for_test(
                    web_auth_path,
                    Some("test-password"),
                )),
            },
            None,
        )
    }

    fn auth_request(builder: http::request::Builder, token: &str) -> http::request::Builder {
        builder.header(header::AUTHORIZATION, format!("Bearer {}", token))
    }

    async fn login_session_token(app: axum::Router) -> String {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"password":"test-password"}"#))
                    .unwrap(),
            )
            .await
            .expect("login response");

        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_payload(response).await;
        payload
            .data
            .and_then(|value| value.get("session_token").cloned())
            .and_then(|value| value.as_str().map(ToString::to_string))
            .expect("session token")
    }

    fn multipart_body(files: &[(&str, &[u8])], boundary: &str) -> Vec<u8> {
        let mut body = Vec::new();
        for (index, (filename, bytes)) in files.iter().enumerate() {
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"file{}\"; filename=\"{}\"\r\n",
                    index, filename
                )
                .as_bytes(),
            );
            body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
            body.extend_from_slice(bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
        body
    }

    async fn response_json(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: ApiResponse = serde_json::from_slice(&body).expect("json payload");
        payload
            .data
            .or_else(|| payload.error.map(Value::String))
            .unwrap_or(Value::Null)
    }

    async fn response_payload(response: axum::response::Response) -> ApiResponse {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        serde_json::from_slice(&body).expect("json payload")
    }

    #[tokio::test]
    async fn protected_routes_require_token_but_health_is_public() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());

        let health = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("health response");
        assert_eq!(health.status(), StatusCode::OK);

        let api = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/does-not-exist")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"params":{}}"#))
                    .unwrap(),
            )
            .await
            .expect("api response");
        assert_eq!(api.status(), StatusCode::UNAUTHORIZED);
        let body = axum::body::to_bytes(api.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: ApiResponse = serde_json::from_slice(&body).expect("json payload");
        assert_eq!(payload.error.as_deref(), Some("Unauthorized"));
        let error_detail = payload.error_detail.expect("structured error detail");
        assert_eq!(error_detail.code, "common.message_unauthorized");
        assert_eq!(error_detail.error_kind.as_deref(), Some("unauthorized"));

        let upload = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/upload")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("upload response");
        assert_eq!(upload.status(), StatusCode::UNAUTHORIZED);

        let logs = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/logs/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("logs response");
        assert_eq!(logs.status(), StatusCode::UNAUTHORIZED);

        let runtime_events = app
            .oneshot(
                Request::builder()
                    .uri("/api/events/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("runtime events response");
        assert_eq!(runtime_events.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn authenticated_api_requests_reach_command_dispatch() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());
        let token = login_session_token(app.clone()).await;

        let response = app
            .oneshot(
                auth_request(
                    Request::builder()
                        .method("POST")
                        .uri("/api/does-not-exist")
                        .header(header::CONTENT_TYPE, "application/json"),
                    &token,
                )
                .body(Body::from(r#"{"params":{}}"#))
                .unwrap(),
            )
            .await
            .expect("api response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: ApiResponse = serde_json::from_slice(&body).expect("json payload");
        assert_eq!(
            payload.error.as_deref(),
            Some(
                "Command 'does-not-exist' not found. Use GET /api/list to see available commands."
            )
        );
        let error_detail = payload.error_detail.expect("structured error detail");
        assert_eq!(error_detail.code, "common.message_server_not_found");
        assert_eq!(error_detail.error_kind.as_deref(), Some("not_found"));
    }

    #[tokio::test]
    async fn authenticated_runtime_event_stream_is_exposed() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());
        let token = login_session_token(app.clone()).await;

        let response = app
            .oneshot(
                auth_request(Request::builder().uri("/api/events/stream"), &token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("runtime event stream response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn auth_status_setup_initialize_and_login_flow_work_over_http() {
        let _lock = lock_env();
        let _recovery_guard = EnvGuard::remove(sea_lantern_runtime::WEB_AUTH_RECOVERY_TOKEN_ENV);
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let web_auth_path = unique_web_auth_state_path("server-web-auth-setup-flow");
        let web_auth = Arc::new(WebAuthService::new_for_test(web_auth_path, None));
        web_auth.seed_setup_token_for_test("setup-secret", 60);
        let app = test_app_with_web_auth(upload_dir.path().to_path_buf(), web_auth);

        let initial_status_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/auth/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("initial auth status response");

        assert_eq!(initial_status_response.status(), StatusCode::OK);
        let initial_status_payload = response_payload(initial_status_response).await;
        let initial_status = initial_status_payload
            .data
            .expect("initial auth status data");
        assert_eq!(
            initial_status.get("state").and_then(|value| value.as_str()),
            Some("setup_pending")
        );
        assert_eq!(
            initial_status
                .get("base_state")
                .and_then(|value| value.as_str()),
            Some("setup_pending")
        );
        assert_eq!(
            initial_status
                .get("setup_required")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            initial_status
                .get("password_login_enabled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );

        let initialize_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/setup/initialize")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"setup_token":"setup-secret","password":"browser-password"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .expect("setup initialize response");

        assert_eq!(initialize_response.status(), StatusCode::OK);
        let initialize_payload = response_payload(initialize_response).await;
        let initialized_session = initialize_payload.data.expect("setup initialize data");
        let initialized_session_token = initialized_session
            .get("session_token")
            .and_then(|value| value.as_str())
            .expect("initialized session token")
            .to_string();
        assert_eq!(
            initialized_session
                .get("token")
                .and_then(|value| value.as_str()),
            Some(initialized_session_token.as_str())
        );
        assert_eq!(
            initialized_session
                .get("purpose")
                .and_then(|value| value.as_str()),
            Some("browser_session")
        );
        assert_eq!(
            initialized_session
                .get("state")
                .and_then(|value| value.as_str()),
            Some("initialized")
        );

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/list")
                    .header(header::AUTHORIZATION, format!("Bearer {}", initialized_session_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list response after setup initialize");

        assert_eq!(list_response.status(), StatusCode::OK);

        let initialized_status_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/auth/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("initialized auth status response");

        assert_eq!(initialized_status_response.status(), StatusCode::OK);
        let initialized_status_payload = response_payload(initialized_status_response).await;
        let initialized_status = initialized_status_payload
            .data
            .expect("initialized auth status data");
        assert_eq!(
            initialized_status
                .get("state")
                .and_then(|value| value.as_str()),
            Some("initialized")
        );
        assert_eq!(
            initialized_status
                .get("base_state")
                .and_then(|value| value.as_str()),
            Some("initialized")
        );
        assert_eq!(
            initialized_status
                .get("setup_required")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            initialized_status
                .get("password_login_enabled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );

        let login_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"password":"browser-password"}"#))
                    .unwrap(),
            )
            .await
            .expect("password login response");

        assert_eq!(login_response.status(), StatusCode::OK);
        let login_payload = response_payload(login_response).await;
        let login_session = login_payload.data.expect("password login data");
        assert_eq!(
            login_session
                .get("purpose")
                .and_then(|value| value.as_str()),
            Some("browser_session")
        );
        assert!(login_session
            .get("session_token")
            .and_then(|value| value.as_str())
            .is_some());
    }

    #[tokio::test]
    async fn auth_status_recovery_reset_and_new_login_flow_work_over_http() {
        let _lock = lock_env();
        let _recovery_guard =
            EnvGuard::set(sea_lantern_runtime::WEB_AUTH_RECOVERY_TOKEN_ENV, "recovery-secret");
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app_with_password(upload_dir.path().to_path_buf(), Some("old-password"));

        let initial_status_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/auth/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("recovery auth status response");

        assert_eq!(initial_status_response.status(), StatusCode::OK);
        let initial_status_payload = response_payload(initial_status_response).await;
        let initial_status = initial_status_payload
            .data
            .expect("recovery auth status data");
        assert_eq!(
            initial_status.get("state").and_then(|value| value.as_str()),
            Some("recovery_active")
        );
        assert_eq!(
            initial_status
                .get("base_state")
                .and_then(|value| value.as_str()),
            Some("initialized")
        );
        assert_eq!(
            initial_status
                .get("password_login_enabled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );

        let recovery_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/recovery/reset")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"recovery_token":"recovery-secret","new_password":"new-password"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .expect("recovery reset response");

        assert_eq!(recovery_response.status(), StatusCode::OK);
        let recovery_payload = response_payload(recovery_response).await;
        let recovery_session = recovery_payload.data.expect("recovery reset data");
        assert_eq!(
            recovery_session
                .get("purpose")
                .and_then(|value| value.as_str()),
            Some("browser_session")
        );
        assert_eq!(
            recovery_session
                .get("state")
                .and_then(|value| value.as_str()),
            Some("initialized")
        );

        let old_login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"password":"old-password"}"#))
                    .unwrap(),
            )
            .await
            .expect("old password login response");

        assert_eq!(old_login_response.status(), StatusCode::UNAUTHORIZED);

        let new_login_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"password":"new-password"}"#))
                    .unwrap(),
            )
            .await
            .expect("new password login response");

        assert_eq!(new_login_response.status(), StatusCode::OK);
        let new_login_payload = response_payload(new_login_response).await;
        let new_login_session = new_login_payload.data.expect("new password login data");
        assert_eq!(
            new_login_session
                .get("purpose")
                .and_then(|value| value.as_str()),
            Some("browser_session")
        );
        assert!(new_login_session
            .get("session_token")
            .and_then(|value| value.as_str())
            .is_some());
    }

    #[tokio::test]
    async fn next_bridge_issue_and_exchange_grant_one_time_browser_session() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());
        let token = login_session_token(app.clone()).await;

        let issue_response = app
            .clone()
            .oneshot(
                auth_request(
                    Request::builder()
                        .method("POST")
                        .uri("/api/auth/next-bridge/issue")
                        .header(header::CONTENT_TYPE, "application/json"),
                    &token,
                )
                .body(Body::from(r#"{"target_path":"/settings"}"#))
                .unwrap(),
            )
            .await
            .expect("issue response");

        assert_eq!(issue_response.status(), StatusCode::OK);
        let issue_payload = response_payload(issue_response).await;
        let issued = issue_payload.data.expect("issue data");
        assert_eq!(issued.get("purpose").and_then(|value| value.as_str()), Some("next_bridge"));
        assert_eq!(issued.get("target_path").and_then(|value| value.as_str()), Some("/settings"));
        let bridge_token = issued
            .get("bridge_token")
            .and_then(|value| value.as_str())
            .expect("bridge token");

        let exchange_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/next-bridge/exchange")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(r#"{{"bridge_token":"{}"}}"#, bridge_token)))
                    .unwrap(),
            )
            .await
            .expect("exchange response");

        assert_eq!(exchange_response.status(), StatusCode::OK);
        let exchange_payload = response_payload(exchange_response).await;
        let exchanged = exchange_payload.data.expect("exchange data");
        assert_eq!(
            exchanged.get("purpose").and_then(|value| value.as_str()),
            Some("browser_session")
        );
        let session_token = exchanged
            .get("token")
            .and_then(|value| value.as_str())
            .expect("session token");
        assert_eq!(
            exchanged
                .get("session_token")
                .and_then(|value| value.as_str()),
            Some(session_token)
        );

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/list")
                    .header(header::AUTHORIZATION, format!("Bearer {}", session_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list response");

        assert_eq!(list_response.status(), StatusCode::OK);

        let replay_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/next-bridge/exchange")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(format!(r#"{{"bridge_token":"{}"}}"#, bridge_token)))
                    .unwrap(),
            )
            .await
            .expect("replay response");

        assert_eq!(replay_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn sanitize_upload_filename_normalizes_and_rejects_unsafe_values() {
        assert_eq!(sanitize_upload_basename("../x.jar").unwrap(), "x.jar");
        assert_eq!(sanitize_upload_basename("a/../b.jar").unwrap(), "b.jar");
        assert_eq!(sanitize_upload_basename("a\\b.jar").unwrap(), "b.jar");

        assert!(sanitize_upload_basename("").is_err());
        assert!(sanitize_upload_basename("\u{0000}bad.jar").is_err());
        assert!(sanitize_upload_basename("CON").is_err());
        assert!(sanitize_upload_basename("NUL.txt").is_err());
    }

    #[test]
    fn upload_target_path_stays_within_upload_directory() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(upload_dir.path()).expect("canonical root");

        for input in ["../x.jar", "a/../b.jar", "a\\b.jar", "nested/../../c.jar"] {
            let target = build_upload_target_path(&root, input).expect("safe target path");
            assert!(target.starts_with(&root));
        }
    }

    #[test]
    fn unique_saved_names_do_not_collide_for_same_input() {
        let first = build_unique_saved_name("plugin.jar");
        let second = build_unique_saved_name("plugin.jar");

        assert_ne!(first, second);
        assert!(first.ends_with(".jar"));
        assert!(second.ends_with(".jar"));
    }

    #[test]
    fn token_log_reference_uses_prefix_and_fingerprint_without_full_value() {
        let token = "12345678-abcdef-full-secret-token";
        let reference = sea_lantern_runtime::format_token_reference(token);

        assert!(reference.contains("prefix=12345678"));
        assert!(reference.contains("fingerprint="));
        assert!(!reference.contains(token));
    }

    #[test]
    fn generated_token_logging_does_not_leak_full_token() {
        let _lock = lock_env();
        let _auth_guard = EnvGuard::remove(sea_lantern_runtime::HTTP_AUTH_TOKEN_ENV);
        let _cors_guard = EnvGuard::remove(sea_lantern_runtime::HTTP_CORS_ORIGINS_ENV);
        GLOBAL_LOG_COLLECTOR.clear();

        let config = default_http_server_config().expect("default config should build");
        for message in sea_lantern_runtime::describe_http_security_configuration(&config) {
            crate::utils::logger::capture_println(message);
        }

        let logs = GLOBAL_LOG_COLLECTOR.get_logs(None);
        assert!(!logs.is_empty());
        assert!(logs
            .iter()
            .all(|entry| !entry.message.contains(config.auth_token.as_str())));
        assert!(logs
            .iter()
            .any(|entry| entry.message.contains("process-local token prefix=")));
    }

    #[tokio::test]
    async fn malformed_multipart_returns_failure_status() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());
        let token = login_session_token(app.clone()).await;

        let response = app
            .oneshot(
                auth_request(
                    Request::builder()
                        .method("POST")
                        .uri("/upload")
                        .header(header::CONTENT_TYPE, "multipart/form-data"),
                    &token,
                )
                .body(Body::from("not a valid multipart body"))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        assert!(!body.is_empty(), "bad multipart response should include an error body");
    }

    #[tokio::test]
    async fn upload_internal_failures_are_classified_as_runtime_errors() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let blocked_root = upload_dir.path().join("blocked");
        std::fs::write(&blocked_root, b"not-a-directory").expect("block upload dir with file");
        let app = test_app(blocked_root);
        let token = login_session_token(app.clone()).await;
        let boundary = "runtime-boundary";
        let body = multipart_body(&[("plugin.jar", b"plugin")], boundary);

        let response = app
            .oneshot(
                auth_request(
                    Request::builder().method("POST").uri("/upload").header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", boundary),
                    ),
                    &token,
                )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: ApiResponse = serde_json::from_slice(&body).expect("json payload");
        let error_detail = payload.error_detail.expect("structured error detail");
        assert_eq!(error_detail.code, "common.message_unknown_error");
        assert_eq!(error_detail.error_kind.as_deref(), Some("runtime"));
    }

    #[tokio::test]
    async fn truncated_multipart_returns_failure_status() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());
        let token = login_session_token(app.clone()).await;
        let boundary = "broken-boundary";
        let body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jar\"\r\nContent-Type: application/octet-stream\r\n\r\nabc",
            boundary
        );

        let response = app
            .oneshot(
                auth_request(
                    Request::builder().method("POST").uri("/upload").header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", boundary),
                    ),
                    &token,
                )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn upload_body_limit_is_enforced() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app_with_limits(upload_dir.path().to_path_buf(), 32, 16);
        let token = login_session_token(app.clone()).await;
        let boundary = "limit-boundary";
        let body = multipart_body(&[("big.bin", &[1u8; 64])], boundary);

        let response = app
            .oneshot(
                auth_request(
                    Request::builder().method("POST").uri("/upload").header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", boundary),
                    ),
                    &token,
                )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn single_file_size_limit_is_enforced_explicitly() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app_with_upload_caps(upload_dir.path().to_path_buf(), 1024 * 1024, 8, 16);
        let token = login_session_token(app.clone()).await;
        let boundary = "single-file-boundary";
        let body = multipart_body(&[("big.jar", &[7u8; 32])], boundary);

        let response = app
            .oneshot(
                auth_request(
                    Request::builder().method("POST").uri("/upload").header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", boundary),
                    ),
                    &token,
                )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let payload = response_json(response).await;
        let message = payload.as_str().expect("error string");
        assert!(message.contains("single-file size limit"));
        assert_eq!(std::fs::read_dir(upload_dir.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn upload_file_count_limit_is_enforced() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app_with_limits(upload_dir.path().to_path_buf(), 1024 * 1024, 1);
        let token = login_session_token(app.clone()).await;
        let boundary = "count-boundary";
        let body = multipart_body(&[("a.jar", b"a"), ("b.jar", b"b")], boundary);

        let response = app
            .oneshot(
                auth_request(
                    Request::builder().method("POST").uri("/upload").header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", boundary),
                    ),
                    &token,
                )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(std::fs::read_dir(upload_dir.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn same_name_uploads_do_not_overwrite_each_other() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = build_test_http_app(upload_dir.path().to_path_buf());
        let token = login_session_token(app.clone()).await;
        let boundary = "unique-boundary";
        let body = multipart_body(&[("plugin.jar", b"first"), ("plugin.jar", b"second")], boundary);

        let response = app
            .oneshot(
                auth_request(
                    Request::builder().method("POST").uri("/upload").header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", boundary),
                    ),
                    &token,
                )
                .body(Body::from(body))
                .unwrap(),
            )
            .await
            .expect("upload response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(std::fs::read_dir(upload_dir.path()).unwrap().count(), 2);

        let payload = response_json(response).await;
        let files = payload
            .get("files")
            .and_then(|value| value.as_array())
            .expect("files array");
        assert_eq!(files.len(), 2);
        assert_ne!(files[0].get("saved_name"), files[1].get("saved_name"));
        for file in files {
            let saved_name = file
                .get("saved_name")
                .and_then(|value| value.as_str())
                .expect("saved_name string");
            let saved_path = file
                .get("saved_path")
                .and_then(|value| value.as_str())
                .expect("saved_path string");
            assert_eq!(saved_path, build_upload_reference(saved_name));
            assert!(!saved_path.contains(upload_dir.path().to_string_lossy().as_ref()));
        }
    }

    #[tokio::test]
    async fn cors_is_disabled_by_default_and_whitelist_is_explicit() {
        let app = {
            let _lock = lock_env();
            let _auth_guard = EnvGuard::remove(sea_lantern_runtime::HTTP_AUTH_TOKEN_ENV);
            let _cors_guard = EnvGuard::remove(sea_lantern_runtime::HTTP_CORS_ORIGINS_ENV);

            let upload_dir = tempfile::tempdir().expect("tempdir");
            test_app(upload_dir.keep())
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/health")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("cors response");
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());

        let app = {
            let _lock = lock_env();
            let _cors_guard =
                EnvGuard::set(sea_lantern_runtime::HTTP_CORS_ORIGINS_ENV, "https://example.com");
            build_http_app(
                AppState {
                    command_registry: Arc::new(CommandRegistry::new()),
                    config: Arc::new(
                        default_http_server_config().expect("default config should build"),
                    ),
                    web_auth: Arc::new(WebAuthService::new_for_test(
                        unique_web_auth_state_path("cors-web-auth-state"),
                        Some("test-password"),
                    )),
                },
                None,
            )
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/health")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("cors response");
        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&header::HeaderValue::from_static("https://example.com"))
        );
    }

    #[test]
    fn default_http_server_config_rejects_invalid_cors_env() {
        let _lock = lock_env();
        let _cors_guard = EnvGuard::set(
            sea_lantern_runtime::HTTP_CORS_ORIGINS_ENV,
            "https://ok.example, bad\nvalue",
        );

        let error = default_http_server_config()
            .expect_err("HTTP server startup config should reject invalid CORS env");

        assert!(error.contains("CORS origin 无效"), "unexpected error: {}", error);
    }
}
