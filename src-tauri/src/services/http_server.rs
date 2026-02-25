use super::http_command_handlers::CommandRegistry;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
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
        .layer(cors)
        .with_state(state);

    // 添加静态文件服务
    if let Some(dir) = static_dir {
        let serve_dir = ServeDir::new(&dir).append_index_html_on_directories(true);
        app = app.fallback_service(serve_dir);
        println!("Serving static files from: {}", dir);
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("SeaLantern HTTP server listening on {}", addr);
    println!("API endpoints available at http://{}/api/<command>", addr);
    println!("Health check at http://{}/health", addr);
    axum::serve(listener, app).await.unwrap();
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
