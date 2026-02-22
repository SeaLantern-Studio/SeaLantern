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
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub command_handler: Arc<Mutex<Option<CommandHandler>>>,
}

type CommandHandler = Box<dyn Fn(&str, Value) -> futures::future::BoxFuture<'static, Result<Value, String>> + Send + Sync>;

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
    let state = AppState {
        command_handler: Arc::new(Mutex::new(None)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new()
        .route("/api/:command", post(handle_api_command))
        .route("/health", get(|| async { "OK" }))
        .route("/api/list", get(list_api_endpoints))
        .layer(cors)
        .with_state(state);

    // 添加静态文件服务
    if let Some(dir) = static_dir {
        let serve_dir = ServeDir::new(&dir).append_index_html_on_directories(true);
        app = app.nest_service("/", serve_dir);
        println!("Serving static files from: {}", dir);
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("SeaLantern HTTP server listening on {}", addr);
    println!("API endpoints available at http://{}/api/<command>", addr);
    println!("Health check at http://{}/health", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn list_api_endpoints() -> impl IntoResponse {
    let endpoints = vec![
        // Server commands
        "server/create_server",
        "server/import_server",
        "server/import_modpack",
        "server/start_server",
        "server/stop_server",
        "server/send_command",
        "server/get_server_list",
        "server/get_server_status",
        "server/delete_server",
        "server/get_server_logs",
        "server/update_server_name",
        // Java commands
        "java/detect_java",
        "java/validate_java_path",
        "java/install_java",
        "java/cancel_java_install",
        // Config commands
        "config/read_config",
        "config/write_config",
        "config/read_server_properties",
        "config/write_server_properties",
        // System commands
        "system/get_system_info",
        "system/pick_jar_file",
        "system/pick_startup_file",
        "system/pick_java_file",
        "system/pick_folder",
        "system/pick_image_file",
        // Player commands
        "player/get_whitelist",
        "player/get_banned_players",
        "player/get_ops",
        "player/add_to_whitelist",
        "player/remove_from_whitelist",
        "player/ban_player",
        "player/unban_player",
        "player/add_op",
        "player/remove_op",
        "player/kick_player",
        "player/export_logs",
        // Settings commands
        "settings/get_settings",
        "settings/save_settings",
        "settings/reset_settings",
        "settings/export_settings",
        "settings/import_settings",
        "settings/check_acrylic_support",
        "settings/apply_acrylic",
        "settings/get_system_fonts",
        // Update commands
        "update/check_update",
        "update/open_download_url",
        // Plugin commands
        "plugin/list_plugins",
        "plugin/scan_plugins",
        "plugin/enable_plugin",
        "plugin/disable_plugin",
        "plugin/get_plugin_nav_items",
        "plugin/install_plugin",
        "plugin/get_plugin_icon",
        "plugin/get_plugin_settings",
        "plugin/set_plugin_settings",
        "plugin/get_plugin_css",
        "plugin/get_all_plugin_css",
        "plugin/delete_plugin",
        "plugin/delete_plugins",
        "plugin/check_plugin_update",
        "plugin/check_all_plugin_updates",
        "plugin/fetch_market_plugins",
        "plugin/fetch_market_categories",
        "plugin/fetch_market_plugin_detail",
        "plugin/install_from_market",
        "plugin/install_plugins_batch",
        "plugin/context_menu_callback",
        "plugin/context_menu_show_notify",
        "plugin/context_menu_hide_notify",
        "plugin/on_locale_changed",
        "plugin/component_mirror_register",
        "plugin/component_mirror_unregister",
        "plugin/component_mirror_clear",
        "plugin/on_page_changed",
        "plugin/get_plugin_component_snapshot",
        "plugin/get_plugin_ui_snapshot",
        "plugin/get_plugin_sidebar_snapshot",
        "plugin/get_plugin_context_menu_snapshot",
    ];

    Json(ApiResponse::success(serde_json::json!({
        "endpoints": endpoints,
        "usage": "POST /api/{command} with JSON body {\"params\": {...}}"
    })))
}

async fn handle_api_command(
    Path(command): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<ApiRequest>,
) -> impl IntoResponse {
    // 记录请求
    eprintln!("[HTTP API] Received command: {}", command);

    // TODO: 实现命令处理逻辑
    // 目前返回一个占位符响应，后续可以通过注册命令处理器来实现真正的命令调用
    let response = ApiResponse {
        success: false,
        data: None,
        error: Some(format!(
            "Command '{}' is not yet implemented in HTTP mode. \
             This requires integrating with Tauri's command system. \
             Available commands: GET /api/list",
            command
        )),
    };

    (StatusCode::NOT_IMPLEMENTED, Json(response)).into_response()
}
