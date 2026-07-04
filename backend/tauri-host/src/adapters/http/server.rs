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
    use crate::test_support::{lock_env, EnvGuard};
    use crate::utils::logger::GLOBAL_LOG_COLLECTOR;
    use axum::{
        body::Body,
        http::{self, header, Request, StatusCode},
    };
    use sea_lantern_runtime::HeadlessHttpConfig;
    use serde_json::Value;
    use std::{path::PathBuf, sync::Arc};
    use tower::ServiceExt;

    use super::upload::{
        build_unique_saved_name, build_upload_reference, build_upload_target_path,
        sanitize_upload_basename,
    };

    fn test_state(upload_dir: PathBuf) -> AppState {
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
        }
    }

    fn test_app(upload_dir: PathBuf) -> axum::Router {
        build_http_app(test_state(upload_dir), None)
    }

    fn test_app_with_limits(
        upload_dir: PathBuf,
        max_upload_bytes: usize,
        max_upload_files: usize,
    ) -> axum::Router {
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
            },
            None,
        )
    }

    fn bearer_request(builder: http::request::Builder) -> http::request::Builder {
        builder.header(header::AUTHORIZATION, "Bearer test-token")
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

        let response = app
            .oneshot(
                bearer_request(
                    Request::builder()
                        .method("POST")
                        .uri("/api/does-not-exist")
                        .header(header::CONTENT_TYPE, "application/json"),
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

        let response = app
            .oneshot(
                bearer_request(Request::builder().uri("/api/events/stream"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("runtime event stream response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn next_bridge_issue_and_exchange_grant_one_time_browser_session() {
        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());

        let issue_response = app
            .clone()
            .oneshot(
                bearer_request(
                    Request::builder()
                        .method("POST")
                        .uri("/api/auth/next-bridge/issue")
                        .header(header::CONTENT_TYPE, "application/json"),
                )
                .body(Body::from(r#"{"target_path":"/settings"}"#))
                .unwrap(),
            )
            .await
            .expect("issue response");

        assert_eq!(issue_response.status(), StatusCode::OK);
        let issue_payload = response_payload(issue_response).await;
        let issued = issue_payload.data.expect("issue data");
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
        let session_token = exchanged
            .get("token")
            .and_then(|value| value.as_str())
            .expect("session token");

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

        let response = app
            .oneshot(
                bearer_request(
                    Request::builder()
                        .method("POST")
                        .uri("/upload")
                        .header(header::CONTENT_TYPE, "multipart/form-data"),
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
        let boundary = "runtime-boundary";
        let body = multipart_body(&[("plugin.jar", b"plugin")], boundary);

        let response = app
            .oneshot(
                bearer_request(Request::builder().method("POST").uri("/upload").header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                ))
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
        let boundary = "broken-boundary";
        let body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jar\"\r\nContent-Type: application/octet-stream\r\n\r\nabc",
            boundary
        );

        let response = app
            .oneshot(
                bearer_request(Request::builder().method("POST").uri("/upload").header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                ))
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
        let boundary = "limit-boundary";
        let body = multipart_body(&[("big.bin", &[1u8; 64])], boundary);

        let response = app
            .oneshot(
                bearer_request(Request::builder().method("POST").uri("/upload").header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                ))
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
        let boundary = "single-file-boundary";
        let body = multipart_body(&[("big.jar", &[7u8; 32])], boundary);

        let response = app
            .oneshot(
                bearer_request(Request::builder().method("POST").uri("/upload").header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                ))
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
        let boundary = "count-boundary";
        let body = multipart_body(&[("a.jar", b"a"), ("b.jar", b"b")], boundary);

        let response = app
            .oneshot(
                bearer_request(Request::builder().method("POST").uri("/upload").header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                ))
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
        let boundary = "unique-boundary";
        let body = multipart_body(&[("plugin.jar", b"first"), ("plugin.jar", b"second")], boundary);

        let response = app
            .oneshot(
                bearer_request(Request::builder().method("POST").uri("/upload").header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={}", boundary),
                ))
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
