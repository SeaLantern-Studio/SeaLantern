//! HTTP 服务入口
//!
//! 这里负责启动 Axum 服务、处理命令请求、文件上传和日志 SSE 推送

use super::command_registry::CommandRegistry;
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

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

/// HTTP 路由共用状态
#[derive(Clone)]
pub struct AppState {
    /// 命令名到处理函数的注册表
    pub command_registry: Arc<CommandRegistry>,
}

/// `/api/{command}` 的请求体
#[derive(Serialize, Deserialize)]
pub struct ApiRequest {
    /// 透传给命令处理器的参数
    #[serde(default)]
    pub params: Value,
}

/// 统一的 HTTP 响应包裹结构
#[derive(Serialize)]
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
    ///
    /// # Parameters
    ///
    /// - `data`: 命令返回数据
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 构建失败响应
    ///
    /// # Parameters
    ///
    /// - `message`: 返回给调用方的错误信息
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// 启动 HTTP 服务
///
/// # Parameters
///
/// - `addr`: 监听地址
/// - `static_dir`: 可选静态文件目录
pub async fn run_http_server(addr: &str, static_dir: Option<String>) {
    let command_registry = Arc::new(CommandRegistry::new());

    let state = AppState { command_registry };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new()
        .route("/api/{command}", post(handle_api_command))
        .route("/health", get(|| async { "OK" }))
        .route("/api/list", get(list_api_endpoints))
        .route("/upload", post(handle_file_upload))
        .route("/api/logs/stream", get(handle_log_stream))
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024))
        .layer(cors)
        .with_state(state);

    if let Some(dir) = static_dir {
        let index_path = format!("{}/index.html", dir);
        let serve_dir = ServeDir::new(&dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(&index_path));
        app = app.fallback_service(serve_dir);
        println!("Serving static files from: {} (SPA fallback enabled)", dir);
    }

    let upload_dir = "/app/uploads";
    if let Err(e) = fs::create_dir_all(upload_dir).await {
        eprintln!("Failed to create upload directory: {}", e);
    }

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("SeaLantern HTTP server failed to bind at {}: {}", addr, e);
            return;
        }
    };

    println!("SeaLantern HTTP server listening on {}", addr);
    println!("API endpoints available at http://{}/api/<command>", addr);
    println!("Health check at http://{}/health", addr);
    println!("File upload available at http://{}/upload", addr);

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("SeaLantern HTTP server error on {}: {}", addr, e);
    }
}

/// 处理文件上传请求
async fn handle_file_upload(mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = "/app/uploads";
    let mut uploaded_files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => {
                eprintln!("[Upload] Field without filename, skipping");
                continue;
            }
        };

        eprintln!("[Upload] Processing file: {} (mime: {:?})", file_name, field.content_type());

        let file_data: Vec<u8> = match field.bytes().await {
            Ok(data) => data.to_vec(),
            Err(e) => {
                let msg = format!("Failed to read file '{}': {}", file_name, e);
                eprintln!("[Upload] {}", msg);
                break;
            }
        };

        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => {
                eprintln!("[Upload] Failed to get system time: {}", e);
                0
            }
        };
        let unique_filename = format!("{}-{}", timestamp, file_name);
        let file_path = format!("{}/{}", upload_dir, unique_filename);

        if let Err(e) = fs::write(&file_path, &file_data).await {
            eprintln!("[Upload] Failed to write file '{}': {}", file_path, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to save file: {}", e))),
            )
                .into_response();
        }

        println!("[Upload] File '{}' saved to '{}'", file_name, file_path);
        uploaded_files.push(serde_json::json!({
            "original_name": file_name,
            "saved_path": file_path,
            "size": file_data.len()
        }));
    }

    if uploaded_files.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("No files uploaded".to_string())),
        )
            .into_response();
    }

    println!("[Upload] Successfully uploaded {} file(s)", uploaded_files.len());
    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "files": uploaded_files,
            "count": uploaded_files.len()
        }))),
    )
        .into_response()
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
        "note": "Plugin commands are not yet supported in HTTP mode",
        "usage": "POST /api/{command} with JSON body {\"params\": {...}}"
    })))
}

/// 处理 `/api/{command}` 调用
async fn handle_api_command(
    Path(command): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ApiRequest>,
) -> impl IntoResponse {
    eprintln!("[HTTP API] Received command: {}", command);

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
            eprintln!("[HTTP API] Command '{}' succeeded", command);
            (StatusCode::OK, Json(ApiResponse::success(data))).into_response()
        }
        Err(e) => {
            eprintln!("[HTTP API] Command '{}' failed: {}", command, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(e))).into_response()
        }
    }
}

/// 处理日志 SSE 订阅
async fn handle_log_stream() -> impl IntoResponse {
    let receiver = LOG_BROADCAST.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(|result| {
        match result {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                Some(Ok::<_, String>(Event::default().data(json)))
            }
            Err(e) => {
                eprintln!("[SSE] Broadcast error: {}", e);
                None
            }
        }
    });
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}
