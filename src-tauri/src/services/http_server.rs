use super::http_command_handlers::CommandRegistry;
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub command_registry: Arc<CommandRegistry>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiRequest {
    #[serde(default)]
    pub params: Value,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ApiResponse {
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

pub async fn run_http_server(addr: &str, static_dir: Option<String>) {
    // 创建命令注册表
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
        // 上传路由添加请求体大小限制（500MB）
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024))
        .layer(cors)
        .with_state(state);

    // 添加静态文件服务
    if let Some(dir) = static_dir {
        let serve_dir = ServeDir::new(&dir).append_index_html_on_directories(true);
        app = app.fallback_service(serve_dir);
        println!("Serving static files from: {}", dir);
    }

    // 创建上传目录
    let upload_dir = "/app/uploads";
    if let Err(e) = fs::create_dir_all(upload_dir).await {
        eprintln!("Failed to create upload directory: {}", e);
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("SeaLantern HTTP server listening on {}", addr);
    println!("API endpoints available at http://{}/api/<command>", addr);
    println!("Health check at http://{}/health", addr);
    println!("File upload available at http://{}/upload", addr);
    axum::serve(listener, app).await.unwrap();
}

/// 处理文件上传请求
async fn handle_file_upload(mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = "/app/uploads";
    let mut uploaded_files = Vec::new();
    let mut error_message = None;

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
                error_message = Some(msg);
                break;
            }
        };

        // 生成唯一文件名
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let unique_filename = format!("{}-{}", timestamp, file_name);
        let file_path = format!("{}/{}", upload_dir, unique_filename);

        // 写入文件
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

async fn list_api_endpoints(State(state): State<AppState>) -> impl IntoResponse {
    let endpoints = state.command_registry.list_commands();

    let supported: Vec<&String> = endpoints
        .iter()
        .filter(|cmd: &&String| !cmd.starts_with("plugin/") || !cmd.contains("unsupported"))
        .collect();

    let _unsupported: Vec<&String> = endpoints
        .iter()
        .filter(|cmd: &&String| cmd.starts_with("plugin/"))
        .collect();

    Json(ApiResponse::success(serde_json::json!({
        "endpoints": endpoints,
        "supported_count": supported.len(),
        "note": "Plugin commands are not yet supported in HTTP mode",
        "usage": "POST /api/{command} with JSON body {\"params\": {...}}"
    })))
}

async fn handle_api_command(
    Path(command): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ApiRequest>,
) -> impl IntoResponse {
    eprintln!("[HTTP API] Received command: {}", command);

    // 获取命令处理器
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

    // 调用处理器（HTTP 模式下不需要 AppHandle）
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
