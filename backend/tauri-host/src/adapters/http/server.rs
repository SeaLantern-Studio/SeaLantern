//! HTTP 服务入口
//!
//! 这里负责启动 Axum 服务、处理命令请求、文件上传和日志 SSE 推送

use super::command_registry::CommandRegistry;
use crate::utils::logger::{capture_eprintln, capture_println};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Request, State},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderMap, HeaderValue, Method, StatusCode,
    },
    middleware::{self, Next},
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use tokio_stream::StreamExt as _;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

const DEFAULT_UPLOAD_DIR: &str = "/app/uploads";
const DEFAULT_MAX_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
const DEFAULT_MAX_UPLOAD_FILE_BYTES: usize = 100 * 1024 * 1024;
const DEFAULT_MAX_UPLOAD_FILES: usize = 16;
const HTTP_AUTH_TOKEN_ENV: &str = "SEALANTERN_HTTP_AUTH_TOKEN";
const HTTP_CORS_ORIGINS_ENV: &str = "SEALANTERN_HTTP_CORS_ORIGINS";

/// HTTP 日志流里的单条日志事件
#[derive(Clone, Serialize)]
pub struct LogEvent {
    /// 产生日志的服务器 ID
    pub server_id: String,
    /// 推送给订阅方的日志内容
    pub line: String,
}

static LOG_BROADCAST: once_cell::sync::Lazy<tokio::sync::broadcast::Sender<LogEvent>> =
    once_cell::sync::Lazy::new(|| {
        let (tx, _rx) = tokio::sync::broadcast::channel(1024);
        tx
    });

/// 读取日志广播发送器
pub fn get_log_sender() -> tokio::sync::broadcast::Sender<LogEvent> {
    LOG_BROADCAST.clone()
}

#[derive(Clone)]
struct HttpServerConfig {
    auth_token: Arc<str>,
    upload_dir: PathBuf,
    cors_allowed_origins: Arc<Vec<HeaderValue>>,
    max_upload_bytes: usize,
    max_upload_file_bytes: usize,
    max_upload_files: usize,
}

/// HTTP 路由共用状态
#[derive(Clone)]
pub struct AppState {
    /// 命令名到处理函数的注册表
    pub command_registry: Arc<CommandRegistry>,
    config: Arc<HttpServerConfig>,
}

/// `/api/{command}` 的请求体
#[derive(Serialize, Deserialize)]
pub struct ApiRequest {
    /// 透传给命令处理器的参数
    #[serde(default)]
    pub params: Value,
}

/// 统一的 HTTP 响应包裹结构
#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    /// 请求是否成功
    pub success: bool,
    /// 成功时的数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// 失败时的错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ApiResponse {
    /// 构建成功响应
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 构建失败响应
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

struct UploadFailure {
    status: StatusCode,
    message: String,
}

impl UploadFailure {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

/// 启动 HTTP 服务
pub async fn run_http_server(
    addr: &str,
    static_dir: Option<String>,
    startup_notifier: Option<std::sync::mpsc::Sender<Result<(), String>>>,
) -> Result<(), String> {
    let state = AppState {
        command_registry: Arc::new(CommandRegistry::new()),
        config: Arc::new(default_http_server_config()),
    };

    if let Err(e) = fs::create_dir_all(&state.config.upload_dir).await {
        capture_eprintln(format!(
            "Failed to create upload directory '{}': {}",
            state.config.upload_dir.display(),
            e
        ));
    }

    log_http_security_configuration(&state.config);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            let message = format!("SeaLantern HTTP server failed to bind at {}: {}", addr, e);
            capture_eprintln(message.clone());
            if let Some(notifier) = startup_notifier {
                let _ = notifier.send(Err(message.clone()));
            }
            return Err(message);
        }
    };

    if let Some(notifier) = startup_notifier {
        let _ = notifier.send(Ok(()));
    }

    let app = build_http_app(state, static_dir);

    capture_println(format!("SeaLantern HTTP server listening on {}", addr));
    capture_println(format!("API endpoints available at http://{}/api/<command>", addr));
    capture_println(format!("Health check at http://{}/health", addr));
    capture_println(format!("File upload available at http://{}/upload", addr));

    if let Err(e) = axum::serve(listener, app).await {
        let message = format!("SeaLantern HTTP server error on {}: {}", addr, e);
        capture_eprintln(message.clone());
        return Err(message);
    }

    Ok(())
}

fn build_http_app(state: AppState, static_dir: Option<String>) -> Router {
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
        capture_println(format!("Serving static files from: {} (SPA fallback enabled)", dir));
    }

    app
}

#[cfg(test)]
pub(crate) fn build_test_http_app(upload_dir: PathBuf) -> Router {
    build_http_app(
        AppState {
            command_registry: Arc::new(CommandRegistry::new()),
            config: Arc::new(HttpServerConfig {
                auth_token: Arc::<str>::from("test-token"),
                upload_dir,
                cors_allowed_origins: Arc::new(Vec::new()),
                max_upload_bytes: 1024 * 1024,
                max_upload_file_bytes: 512 * 1024,
                max_upload_files: 16,
            }),
        },
        None,
    )
}

fn default_http_server_config() -> HttpServerConfig {
    let configured_token = env_var_trimmed(HTTP_AUTH_TOKEN_ENV);
    let auth_token = configured_token
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    HttpServerConfig {
        auth_token: Arc::<str>::from(auth_token),
        upload_dir: PathBuf::from(DEFAULT_UPLOAD_DIR),
        cors_allowed_origins: Arc::new(parse_cors_allowed_origins()),
        max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
        max_upload_file_bytes: DEFAULT_MAX_UPLOAD_FILE_BYTES,
        max_upload_files: DEFAULT_MAX_UPLOAD_FILES,
    }
}

fn log_http_security_configuration(config: &HttpServerConfig) {
    let token_reference = format_token_reference(config.auth_token.as_ref());

    if std::env::var(HTTP_AUTH_TOKEN_ENV).is_ok() {
        capture_println(format!(
            "SeaLantern HTTP auth enabled with configured token {}",
            token_reference
        ));
    } else {
        capture_println(format!(
            "SeaLantern HTTP auth generated a process-local token {}",
            token_reference
        ));
        capture_println(format!(
            "Set '{}' explicitly for a stable token; otherwise use the in-process generated token with header '{}: Bearer <token>' when calling /api/*, /upload, or /api/logs/stream",
            HTTP_AUTH_TOKEN_ENV,
            AUTHORIZATION.as_str(),
        ));
    }

    if config.cors_allowed_origins.is_empty() {
        capture_println(
            "SeaLantern HTTP CORS disabled by default; set SEALANTERN_HTTP_CORS_ORIGINS to allow browser origins"
                .to_string(),
        );
    } else {
        capture_println(format!(
            "SeaLantern HTTP CORS allowlist enabled for {} origin(s)",
            config.cors_allowed_origins.len()
        ));
    }
}

fn build_cors_layer(allowed_origins: &[HeaderValue]) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    if allowed_origins.is_empty() {
        cors
    } else {
        cors.allow_origin(allowed_origins.to_vec())
    }
}

fn parse_cors_allowed_origins() -> Vec<HeaderValue> {
    env_var_trimmed(HTTP_CORS_ORIGINS_ENV)
        .map(|value| {
            value
                .split(',')
                .filter_map(|origin| {
                    let trimmed = origin.trim();
                    if trimmed.is_empty() {
                        return None;
                    }

                    HeaderValue::from_str(trimmed).ok()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn env_var_trimmed(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn token_fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest[..6]
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

fn token_prefix(token: &str) -> &str {
    let len = token.len().min(8);
    &token[..len]
}

fn format_token_reference(token: &str) -> String {
    format!("prefix={} fingerprint={}", token_prefix(token), token_fingerprint(token))
}

async fn require_bearer_auth(
    State(config): State<Arc<HttpServerConfig>>,
    request: Request,
    next: Next,
) -> Response {
    match extract_bearer_token(request.headers()) {
        Some(token) if token == config.auth_token.as_ref() => next.run(request).await,
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

/// 处理文件上传请求
async fn handle_file_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    match save_uploaded_files(&state, &mut multipart).await {
        Ok(uploaded_files) => {
            capture_println(format!(
                "[Upload] Successfully uploaded {} file(s)",
                uploaded_files.len()
            ));
            (
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "files": uploaded_files,
                    "count": uploaded_files.len()
                }))),
            )
                .into_response()
        }
        Err(error) => {
            capture_eprintln(format!("[Upload] {}", error.message));
            (error.status, Json(ApiResponse::error(error.message))).into_response()
        }
    }
}

async fn save_uploaded_files(
    state: &AppState,
    multipart: &mut Multipart,
) -> Result<Vec<Value>, UploadFailure> {
    let upload_root = ensure_upload_root(&state.config.upload_dir).await?;
    let mut uploaded_files = Vec::new();
    let mut committed_paths = Vec::new();
    let mut file_count = 0usize;

    loop {
        let next_field = multipart.next_field().await.map_err(|error| {
            UploadFailure::new(
                error.status(),
                format!("Failed to parse multipart payload: {}", error.body_text()),
            )
        })?;

        let Some(mut field) = next_field else {
            break;
        };

        file_count += 1;
        if file_count > state.config.max_upload_files {
            cleanup_saved_files(&committed_paths).await;
            return Err(UploadFailure::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "Too many files in a single upload request (max {})",
                    state.config.max_upload_files
                ),
            ));
        }

        let original_name = field
            .file_name()
            .ok_or_else(|| {
                UploadFailure::new(
                    StatusCode::BAD_REQUEST,
                    "Upload field is missing a filename".to_string(),
                )
            })?
            .to_string();
        let safe_name = sanitize_upload_basename(&original_name).map_err(|message| {
            UploadFailure::new(
                StatusCode::BAD_REQUEST,
                format!("Invalid upload filename '{}': {}", original_name, message),
            )
        })?;

        let (mut file, save_path) = open_unique_upload_file(&upload_root, &safe_name).await?;
        let mut size = 0usize;

        let write_result = async {
            while let Some(chunk) = field.chunk().await.map_err(|error| {
                UploadFailure::new(
                    error.status(),
                    format!(
                        "Failed to read uploaded file '{}': {}",
                        original_name,
                        error.body_text()
                    ),
                )
            })? {
                size = size.saturating_add(chunk.len());
                if size > state.config.max_upload_file_bytes {
                    return Err(UploadFailure::new(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "Uploaded file '{}' exceeds the single-file size limit of {} bytes",
                            original_name, state.config.max_upload_file_bytes
                        ),
                    ));
                }

                file.write_all(&chunk).await.map_err(|error| {
                    UploadFailure::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to save uploaded file '{}': {}", original_name, error),
                    )
                })?;
            }

            file.flush().await.map_err(|error| {
                UploadFailure::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to finalize uploaded file '{}': {}", original_name, error),
                )
            })?;

            Ok::<(), UploadFailure>(())
        }
        .await;

        if let Err(error) = write_result {
            cleanup_saved_files(&committed_paths).await;
            let _ = fs::remove_file(&save_path).await;
            return Err(error);
        }

        capture_println(format!(
            "[Upload] File '{}' saved to '{}'",
            original_name,
            save_path.display()
        ));

        committed_paths.push(save_path.clone());
        uploaded_files.push(serde_json::json!({
            "original_name": original_name,
            "saved_name": save_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
            "saved_path": save_path.to_string_lossy(),
            "size": size
        }));
    }

    if uploaded_files.is_empty() {
        return Err(UploadFailure::new(StatusCode::BAD_REQUEST, "No files uploaded".to_string()));
    }

    Ok(uploaded_files)
}

async fn ensure_upload_root(upload_dir: &FsPath) -> Result<PathBuf, UploadFailure> {
    fs::create_dir_all(upload_dir).await.map_err(|error| {
        UploadFailure::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to prepare upload directory '{}': {}", upload_dir.display(), error),
        )
    })?;

    fs::canonicalize(upload_dir).await.map_err(|error| {
        UploadFailure::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to resolve upload directory '{}': {}", upload_dir.display(), error),
        )
    })
}

async fn open_unique_upload_file(
    upload_root: &FsPath,
    safe_name: &str,
) -> Result<(tokio::fs::File, PathBuf), UploadFailure> {
    for _ in 0..8 {
        let save_name = build_unique_saved_name(safe_name);
        let save_path = build_upload_target_path(upload_root, &save_name)
            .map_err(|message| UploadFailure::new(StatusCode::BAD_REQUEST, message))?;

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&save_path)
            .await
        {
            Ok(file) => return Ok((file, save_path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(UploadFailure::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to open upload target '{}': {}", save_path.display(), error),
                ));
            }
        }
    }

    Err(UploadFailure::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to allocate a unique upload filename after several attempts".to_string(),
    ))
}

fn build_upload_target_path(upload_root: &FsPath, raw_name: &str) -> Result<PathBuf, String> {
    let safe_name = sanitize_upload_basename(raw_name)?;
    let candidate = upload_root.join(&safe_name);

    if candidate.starts_with(upload_root) {
        Ok(candidate)
    } else {
        Err(format!(
            "Resolved upload path '{}' escapes the upload directory '{}'",
            candidate.display(),
            upload_root.display()
        ))
    }
}

fn sanitize_upload_basename(raw_name: &str) -> Result<String, String> {
    let normalized = raw_name.replace('\\', "/");
    let basename = normalized
        .rsplit('/')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .trim_end_matches([' ', '.']);

    if basename.is_empty() || basename == "." || basename == ".." {
        return Err("filename must contain a non-empty basename".to_string());
    }

    if basename.chars().any(char::is_control) {
        return Err("filename contains control characters".to_string());
    }

    if basename.contains('/') || basename.contains('\\') {
        return Err("filename contains path separators after normalization".to_string());
    }

    let reserved = basename
        .split('.')
        .next()
        .unwrap_or(basename)
        .trim()
        .to_ascii_uppercase();
    if is_reserved_platform_name(&reserved) {
        return Err("filename uses a platform-reserved basename".to_string());
    }

    Ok(basename.to_string())
}

fn is_reserved_platform_name(name: &str) -> bool {
    matches!(
        name,
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn build_unique_saved_name(safe_name: &str) -> String {
    let path = FsPath::new(safe_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("upload");
    let extension = path.extension().and_then(|value| value.to_str());
    let suffix = Uuid::new_v4().simple().to_string();

    match extension {
        Some(extension) if !extension.is_empty() => format!("{}-{}.{}", stem, suffix, extension),
        _ => format!("{}-{}", stem, suffix),
    }
}

async fn cleanup_saved_files(paths: &[PathBuf]) {
    for path in paths {
        if let Err(error) = fs::remove_file(path).await {
            capture_eprintln(format!(
                "[Upload] Failed to clean up partial upload '{}': {}",
                path.display(),
                error
            ));
        }
    }
}

/// 返回当前可用的命令列表
async fn list_api_endpoints(State(state): State<AppState>) -> impl IntoResponse {
    let endpoints = state.command_registry.list_commands();

    let supported: Vec<&String> = endpoints
        .iter()
        .filter(|cmd: &&String| !cmd.starts_with("plugin/") || !cmd.contains("unsupported"))
        .collect();

    Json(ApiResponse::success(serde_json::json!({
        "endpoints": endpoints,
        "supported_count": supported.len(),
        "note": "Some plugin commands are supported in HTTP mode, but install and enable flows are still limited",
        "usage": "POST /api/{command} with JSON body {\"params\": {...}}"
    })))
}

/// 处理 `/api/{command}` 调用
async fn handle_api_command(
    Path(command): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ApiRequest>,
) -> impl IntoResponse {
    capture_eprintln(format!("[HTTP API] Received command: {}", command));

    let handler = match state.command_registry.get_handler(&command) {
        Some(h) => h,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(format!(
                    "Command '{}' not found. Use GET /api/list to see available commands.",
                    command
                ))),
            )
                .into_response();
        }
    };

    match handler(payload.params).await {
        Ok(data) => {
            capture_eprintln(format!("[HTTP API] Command '{}' succeeded", command));
            (StatusCode::OK, Json(ApiResponse::success(data))).into_response()
        }
        Err(e) => {
            capture_eprintln(format!("[HTTP API] Command '{}' failed: {}", command, e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(e))).into_response()
        }
    }
}

/// 处理日志 SSE 订阅
async fn handle_log_stream() -> impl IntoResponse {
    let receiver = LOG_BROADCAST.subscribe();
    let stream =
        tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(|result| match result {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                Some(Ok::<_, String>(Event::default().data(json)))
            }
            Err(e) => {
                capture_eprintln(format!("[SSE] Broadcast error: {}", e));
                None
            }
        });
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_http_app, build_unique_saved_name, build_upload_target_path,
        default_http_server_config, format_token_reference, sanitize_upload_basename, ApiResponse,
        AppState, CommandRegistry, HttpServerConfig, HTTP_AUTH_TOKEN_ENV, HTTP_CORS_ORIGINS_ENV,
    };
    use crate::utils::logger::GLOBAL_LOG_COLLECTOR;
    use axum::{
        body::Body,
        http::{self, header, Request, StatusCode},
    };
    use serde_json::Value;
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use tower::ServiceExt;

    static ENV_LOCK: once_cell::sync::Lazy<Mutex<()>> =
        once_cell::sync::Lazy::new(|| Mutex::new(()));

    struct EnvGuard {
        name: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let original = std::env::var(name).ok();
            std::env::set_var(name, value);
            Self { name, original }
        }

        fn remove(name: &'static str) -> Self {
            let original = std::env::var(name).ok();
            std::env::remove_var(name);
            Self { name, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(original) = &self.original {
                std::env::set_var(self.name, original);
            } else {
                std::env::remove_var(self.name);
            }
        }
    }

    fn test_state(upload_dir: PathBuf) -> AppState {
        AppState {
            command_registry: Arc::new(CommandRegistry::new()),
            config: Arc::new(HttpServerConfig {
                auth_token: Arc::<str>::from("test-token"),
                upload_dir,
                cors_allowed_origins: Arc::new(Vec::new()),
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
                config: Arc::new(HttpServerConfig {
                    auth_token: Arc::<str>::from("test-token"),
                    upload_dir,
                    cors_allowed_origins: Arc::new(Vec::new()),
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
                config: Arc::new(HttpServerConfig {
                    auth_token: Arc::<str>::from("test-token"),
                    upload_dir,
                    cors_allowed_origins: Arc::new(Vec::new()),
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
            .oneshot(
                Request::builder()
                    .uri("/api/logs/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("logs response");
        assert_eq!(logs.status(), StatusCode::UNAUTHORIZED);
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
        let reference = format_token_reference(token);

        assert!(reference.contains("prefix=12345678"));
        assert!(reference.contains("fingerprint="));
        assert!(!reference.contains(token));
    }

    #[test]
    fn generated_token_logging_does_not_leak_full_token() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _auth_guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _cors_guard = EnvGuard::remove(HTTP_CORS_ORIGINS_ENV);
        GLOBAL_LOG_COLLECTOR.clear();

        let config = default_http_server_config();
        super::log_http_security_configuration(&config);

        let logs = GLOBAL_LOG_COLLECTOR.get_logs(None);
        assert!(!logs.is_empty());
        assert!(logs
            .iter()
            .all(|entry| !entry.message.contains(config.auth_token.as_ref())));
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
        let app = test_app(upload_dir.path().to_path_buf());
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
    }

    #[tokio::test]
    async fn cors_is_disabled_by_default_and_whitelist_is_explicit() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _auth_guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _cors_guard = EnvGuard::remove(HTTP_CORS_ORIGINS_ENV);

        let upload_dir = tempfile::tempdir().expect("tempdir");
        let app = test_app(upload_dir.path().to_path_buf());
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

        let _cors_guard = EnvGuard::set(HTTP_CORS_ORIGINS_ENV, "https://example.com");
        let app = build_http_app(
            AppState {
                command_registry: Arc::new(CommandRegistry::new()),
                config: Arc::new(default_http_server_config()),
            },
            None,
        );
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
}
