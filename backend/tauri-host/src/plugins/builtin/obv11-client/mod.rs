use crate::models::plugin::{PluginSettingField, PluginSettingOption};
use crate::models::server::ServerStatus;
use crate::plugins::manager::PluginManager;
use crate::services::events::{ServerEventEnvelope, ServerEventPayload};
use crate::services::global;
use crate::utils::logger::{log_error_ctx, log_info_ctx, log_warn_ctx};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures::{SinkExt, StreamExt};
use sea_lantern_server_config_core::discovery::discover_server_config_files;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, oneshot};

pub(crate) const PLUGIN_ID: &str = "sea-lantern-obv11-client";
const CONFIRM_TTL_SECS: u64 = 120;
const CONFIG_POLL_INTERVAL_SECS: u64 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Obv11Mode {
    QqHttpForward,
    ApiOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QqTarget {
    target_type: String,
    id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeSettings {
    enabled: bool,
    mode: Obv11Mode,
    listen_addr: String,
    access_token: String,
    self_id: i64,
    enable_http_api: bool,
    enable_ws_api: bool,
    enable_http_post: bool,
    http_post_urls: Vec<String>,
    http_post_secret: String,
    qq_api_base_url: String,
    qq_access_token: String,
    qq_targets: Vec<QqTarget>,
}

#[derive(Debug, Clone)]
struct RuntimeSnapshot {
    plugin_id: String,
    settings: RuntimeSettings,
    started_at: u64,
    event_sender: broadcast::Sender<String>,
    debug_sender: broadcast::Sender<String>,
}

struct RuntimeHandle {
    snapshot: Arc<RuntimeSnapshot>,
    shutdown_http: Option<oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[derive(Clone)]
struct HttpAppState {
    runtime: Arc<RuntimeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiEnvelope {
    action: String,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    echo: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfirmationTicket {
    token: String,
    action: String,
    server_id: String,
    payload: Value,
    expires_at: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ActionResponse {
    status: &'static str,
    retcode: i32,
    data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct ServerStatusEntry {
    server_id: String,
    server_name: String,
    status: String,
    detail: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct ConfigFingerprint {
    entries: HashMap<String, HashMap<String, u64>>,
}

fn runtime_registry() -> &'static Mutex<HashMap<String, RuntimeHandle>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, RuntimeHandle>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn confirmation_registry() -> &'static Mutex<HashMap<String, ConfirmationTicket>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, ConfirmationTicket>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_counter() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn log_plugin_info(function: &str, message: &str) {
    log_info_ctx("plugins.builtin.obv11_client", function, message);
}

fn log_plugin_warn(function: &str, message: &str) {
    log_warn_ctx("plugins.builtin.obv11_client", function, message);
}

fn log_plugin_error(function: &str, message: &str) {
    log_error_ctx("plugins.builtin.obv11_client", function, message);
}

fn log_enable_stage(function: &str, plugin_id: &str, stage: &str, message: &str) {
    log_plugin_info(
        function,
        &format!("plugin_id={} stage={} {}", plugin_id, stage, message),
    );
}

fn log_enable_failure(function: &str, plugin_id: &str, stage: &str, error: &str) {
    log_plugin_error(
        function,
        &format!("plugin_id={} stage={} error={}", plugin_id, stage, error),
    );
}

fn startup_error(plugin_id: &str, stage: &str, error: impl Into<String>) -> String {
    let error = error.into();
    log_enable_failure("enable", plugin_id, stage, &error);
    let final_message = format!(
        "plugin_id={} stage=startup_aborted result=plugin enable failed runtime attempt closed cause_stage={} error={}",
        plugin_id, stage, error
    );
    log_plugin_error("enable", &final_message);
    format!("plugin enable failed at stage '{}': {}", stage, error)
}

pub(crate) fn manifest_settings() -> Vec<PluginSettingField> {
    vec![
        PluginSettingField {
            key: "enabled".to_string(),
            label: "Enable Service".to_string(),
            field_type: "boolean".to_string(),
            display: None,
            default: Some(json!(true)),
            description: Some("启用 OneBot v11 服务端与事件分发。".to_string()),
            options: None,
            rows: None,
            maxlength: None,
        },
        PluginSettingField {
            key: "mode".to_string(),
            label: "Mode".to_string(),
            field_type: "select".to_string(),
            display: None,
            default: Some(json!("api_only")),
            description: Some(
                "api_only 仅开放 API；qq_http_forward 额外把事件转发到 QQ OneBot HTTP API。"
                    .to_string(),
            ),
            options: Some(vec![
                PluginSettingOption {
                    value: "api_only".to_string(),
                    label: "API Only".to_string(),
                },
                PluginSettingOption {
                    value: "qq_http_forward".to_string(),
                    label: "QQ HTTP Forward".to_string(),
                },
            ]),
            rows: None,
            maxlength: None,
        },
        PluginSettingField {
            key: "listen_addr".to_string(),
            label: "Listen Addr".to_string(),
            field_type: "string".to_string(),
            display: None,
            default: Some(json!("127.0.0.1:5710")),
            description: Some("HTTP 与 WebSocket 共用监听地址。".to_string()),
            options: None,
            rows: None,
            maxlength: Some(128),
        },
        PluginSettingField {
            key: "access_token".to_string(),
            label: "Access Token".to_string(),
            field_type: "string".to_string(),
            display: None,
            default: Some(json!("")),
            description: Some("HTTP / WebSocket API 鉴权 token；为空则不鉴权。".to_string()),
            options: None,
            rows: None,
            maxlength: Some(256),
        },
        PluginSettingField {
            key: "qq_targets".to_string(),
            label: "QQ Targets".to_string(),
            field_type: "textarea".to_string(),
            display: None,
            default: Some(json!("")),
            description: Some("每行一个目标，格式 `group:123456` 或 `private:10001`。".to_string()),
            options: None,
            rows: Some(4),
            maxlength: Some(2048),
        },
    ]
}

pub(crate) fn default_settings_json() -> Value {
    json!({
        "enabled": true,
        "mode": "api_only",
        "listen_addr": "127.0.0.1:5710",
        "access_token": "",
        "self_id": 0,
        "enable_http_api": true,
        "enable_ws_api": true,
        "enable_http_post": false,
        "http_post_urls": "",
        "http_post_secret": "",
        "qq_api_base_url": "",
        "qq_access_token": "",
        "qq_targets": ""
    })
}

pub(crate) fn enable(manager: &PluginManager, plugin_id: &str) -> Result<(), String> {
    log_enable_stage("enable", plugin_id, "enable_requested", "begin");
    disable(plugin_id);
    let settings = load_settings(manager, plugin_id)?;
    if !settings.enabled {
        log_enable_stage("enable", plugin_id, "enable_skipped", "service disabled by settings");
        return Ok(());
    }

    let runtime = build_runtime(plugin_id, settings);
    let handle = start_runtime_thread(Arc::clone(&runtime))?;
    runtime_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(plugin_id.to_string(), handle);
    send_meta_event(&runtime, "enable");
    log_enable_stage("enable", plugin_id, "startup_handshake_succeeded", "runtime registered");
    emit_debug(&runtime, "plugin_enabled", "OneBot v11 builtin plugin enabled");
    Ok(())
}

pub(crate) fn disable(plugin_id: &str) {
    let handle = runtime_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(plugin_id);
    if let Some(mut handle) = handle {
        send_meta_event(&handle.snapshot, "disable");
        if let Some(shutdown) = handle.shutdown_http.take() {
            let _ = shutdown.send(());
        }
        if let Some(thread) = handle.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(crate) fn reload(manager: &PluginManager, plugin_id: &str) -> Result<(), String> {
    if runtime_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .contains_key(plugin_id)
    {
        enable(manager, plugin_id)?;
    }
    Ok(())
}

pub(crate) fn notify_server_event(plugin_id: &str, event: &ServerEventEnvelope) {
    let runtime = {
        let registry = runtime_registry().lock().unwrap_or_else(|e| e.into_inner());
        registry
            .get(plugin_id)
            .map(|handle| Arc::clone(&handle.snapshot))
    };
    let Some(runtime) = runtime else {
        return;
    };
    let mapped = map_server_event_to_onebot(&runtime, event);
    dispatch_event(&runtime, mapped);
}

fn build_runtime(plugin_id: &str, settings: RuntimeSettings) -> Arc<RuntimeSnapshot> {
    let (event_sender, _) = broadcast::channel(256);
    let (debug_sender, _) = broadcast::channel(256);
    Arc::new(RuntimeSnapshot {
        plugin_id: plugin_id.to_string(),
        settings,
        started_at: now_secs(),
        event_sender,
        debug_sender,
    })
}

fn load_settings(manager: &PluginManager, plugin_id: &str) -> Result<RuntimeSettings, String> {
    let settings_path = settings_file_path(manager.data_dir_path(), plugin_id);
    let value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read builtin settings file: {}", e))?;
        serde_json::from_str::<Value>(&content)
            .map_err(|e| format!("Failed to parse builtin settings file: {}", e))?
    } else {
        default_settings_json()
    };
    parse_settings(value)
}

fn settings_file_path(data_dir: &Path, plugin_id: &str) -> PathBuf {
    crate::plugins::builtin::builtin_settings_dir(data_dir)
        .join(plugin_id)
        .join("settings.json")
}

fn parse_settings(value: Value) -> Result<RuntimeSettings, String> {
    let mode = match value
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("api_only")
    {
        "qq_http_forward" => Obv11Mode::QqHttpForward,
        _ => Obv11Mode::ApiOnly,
    };

    Ok(RuntimeSettings {
        enabled: value
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        mode,
        listen_addr: value
            .get("listen_addr")
            .and_then(Value::as_str)
            .unwrap_or("127.0.0.1:5710")
            .trim()
            .to_string(),
        access_token: value
            .get("access_token")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string(),
        self_id: value.get("self_id").and_then(Value::as_i64).unwrap_or(0),
        enable_http_api: value
            .get("enable_http_api")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        enable_ws_api: value
            .get("enable_ws_api")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        enable_http_post: value
            .get("enable_http_post")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        http_post_urls: parse_lines(
            value
                .get("http_post_urls")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ),
        http_post_secret: value
            .get("http_post_secret")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string(),
        qq_api_base_url: value
            .get("qq_api_base_url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string(),
        qq_access_token: value
            .get("qq_access_token")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string(),
        qq_targets: parse_targets(
            value
                .get("qq_targets")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ),
    })
}

fn parse_lines(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_targets(raw: &str) -> Vec<QqTarget> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (target_type, id) = trimmed.split_once(':')?;
            let id = id.trim().parse::<i64>().ok()?;
            Some(QqTarget {
                target_type: target_type.trim().to_string(),
                id,
            })
        })
        .collect()
}

fn start_runtime_thread(runtime: Arc<RuntimeSnapshot>) -> Result<RuntimeHandle, String> {
    log_enable_stage(
        "start_runtime_thread",
        &runtime.plugin_id,
        "startup_thread_prepare",
        &format!("listen_addr={}", runtime.settings.listen_addr),
    );
    let addr: SocketAddr =
        runtime.settings.listen_addr.parse().map_err(|e| {
            startup_error(
                &runtime.plugin_id,
                "listen_addr_parse_failed",
                format!("invalid listen_addr '{}': {}", runtime.settings.listen_addr, e),
            )
        })?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (startup_tx, startup_rx) = mpsc::sync_channel(1);
    let snapshot = Arc::clone(&runtime);
    let thread = std::thread::Builder::new()
        .name(format!("obv11-startup-{}", runtime.plugin_id))
        .spawn(move || {
        log_enable_stage(
            "start_runtime_thread",
            &snapshot.plugin_id,
            "startup_thread_spawned",
            &format!("listen_addr={addr}"),
        );
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(error) => {
                let message = startup_error(
                    &snapshot.plugin_id,
                    "tokio_runtime_build_failed",
                    format!("failed to build tokio runtime: {}", error),
                );
                let _ = startup_tx.send(Err(message));
                return;
            }
        };
        log_enable_stage(
            "start_runtime_thread",
            &snapshot.plugin_id,
            "tokio_runtime_built",
            "startup runtime ready",
        );
        rt.block_on(async move {
            let state = HttpAppState { runtime: Arc::clone(&snapshot) };
            let app = Router::new()
                .route("/health", get(|| async { "OK" }))
                .route("/api/{action}", get(handle_http_action).post(handle_http_action))
                .route("/api", get(handle_ws_api))
                .route("/event", get(handle_ws_event))
                .route("/", get(handle_ws_universal))
                .with_state(state);
            log_enable_stage(
                "start_runtime_thread",
                &snapshot.plugin_id,
                "listener_bind_requested",
                &format!("listen_addr={addr}"),
            );
            let listener = match TcpListener::bind(addr).await {
                Ok(listener) => listener,
                Err(error) => {
                    let message = startup_error(
                        &snapshot.plugin_id,
                        "listener_bind_failed",
                        format!("bind failed: {}", error),
                    );
                    let _ = startup_tx.send(Err(message));
                    return;
                }
            };

            log_enable_stage(
                "start_runtime_thread",
                &snapshot.plugin_id,
                "startup_handshake_succeeded",
                &format!("listen_addr={addr}"),
            );
            let _ = startup_tx.send(Ok(()));

            emit_debug(&snapshot, "http_server_bound", &format!("addr={addr}"));
            let poll_runtime = Arc::clone(&snapshot);
            let poll_task = tokio::spawn(async move { poll_config_changes(poll_runtime).await });

            let server = axum::serve(listener, app).with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            });
            if let Err(error) = server.await {
                log_enable_failure(
                    "start_runtime_thread",
                    &snapshot.plugin_id,
                    "runtime_closed",
                    &format!("server exited: {}", error),
                );
            } else {
                log_enable_stage(
                    "start_runtime_thread",
                    &snapshot.plugin_id,
                    "runtime_closed",
                    "runtime closed after shutdown",
                );
            }
            poll_task.abort();
        });
    })
        .map_err(|error| {
            startup_error(
                &runtime.plugin_id,
                "startup_thread_spawn_failed",
                format!("failed to spawn startup thread: {}", error),
            )
        })?;

    match startup_rx.recv() {
        Ok(Ok(())) => Ok(RuntimeHandle {
            snapshot: runtime,
            shutdown_http: Some(shutdown_tx),
            thread: Some(thread),
        }),
        Ok(Err(error)) => {
            let _ = thread.join();
            Err(error)
        }
        Err(_) => {
            let _ = thread.join();
            Err(startup_error(
                &runtime.plugin_id,
                "startup_handshake_failed",
                "runtime thread exited before startup completed",
            ))
        }
    }
}

async fn poll_config_changes(runtime: Arc<RuntimeSnapshot>) {
    let mut current = build_config_fingerprint();
    loop {
        tokio::time::sleep(Duration::from_secs(CONFIG_POLL_INTERVAL_SECS)).await;
        let next = build_config_fingerprint();
        for (server_id, changed_files) in diff_fingerprint(&current, &next) {
            dispatch_event(
                &runtime,
                json!({
                    "time": now_secs(),
                    "self_id": runtime.settings.self_id,
                    "post_type": "notice",
                    "notice_type": "sealantern_server_config_changed",
                    "server_id": server_id,
                    "changed_files": changed_files,
                }),
            );
        }
        current = next;
    }
}

fn build_config_fingerprint() -> ConfigFingerprint {
    let mut entries = HashMap::new();
    for server in global::server_manager().get_server_list() {
        let mut files = HashMap::new();
        let discovered = match discover_server_config_files(&server.path) {
            Ok(discovered) => discovered,
            Err(error) => {
                log_plugin_warn(
                    "build_config_fingerprint",
                    &format!(
                        "discover config files failed server_id={} error={}",
                        server.id, error
                    ),
                );
                Vec::new()
            }
        };
        for entry in discovered {
            let relative = entry.relative_path;
            let path = Path::new(&server.path).join(&relative);
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                        files.insert(relative, duration.as_secs());
                    }
                }
            }
        }
        entries.insert(server.id, files);
    }
    ConfigFingerprint { entries }
}

fn diff_fingerprint(
    current: &ConfigFingerprint,
    next: &ConfigFingerprint,
) -> Vec<(String, Vec<String>)> {
    let mut changes = Vec::new();
    for (server_id, next_files) in &next.entries {
        let current_files = current.entries.get(server_id);
        let mut changed = Vec::new();
        for (file, timestamp) in next_files {
            let prev = current_files.and_then(|files| files.get(file)).copied();
            if prev != Some(*timestamp) {
                changed.push(file.clone());
            }
        }
        if !changed.is_empty() {
            changes.push((server_id.clone(), changed));
        }
    }
    changes
}

async fn handle_http_action(
    State(state): State<HttpAppState>,
    AxumPath(action): AxumPath<String>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Json<Value>>,
) -> Response {
    if !authorized(&state.runtime.settings.access_token, &headers, &query) {
        return json_response(
            StatusCode::UNAUTHORIZED,
            action_response("failed", 1401, json!({ "error": "unauthorized" }), None),
        );
    }

    let params = body
        .map(|Json(value)| value)
        .unwrap_or_else(|| json!(query));
    handle_action_response(&state.runtime, ApiEnvelope { action, params, echo: None }).await
}

async fn handle_ws_api(
    ws: WebSocketUpgrade,
    State(state): State<HttpAppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !state.runtime.settings.enable_ws_api
        || !authorized(&state.runtime.settings.access_token, &headers, &query)
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let runtime = Arc::clone(&state.runtime);
    ws.on_upgrade(move |socket| handle_api_socket(runtime, socket))
        .into_response()
}

async fn handle_ws_event(
    ws: WebSocketUpgrade,
    State(state): State<HttpAppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !state.runtime.settings.enable_ws_api
        || !authorized(&state.runtime.settings.access_token, &headers, &query)
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let runtime = Arc::clone(&state.runtime);
    ws.on_upgrade(move |socket| handle_event_socket(runtime, socket, false))
        .into_response()
}

async fn handle_ws_universal(
    ws: WebSocketUpgrade,
    State(state): State<HttpAppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !state.runtime.settings.enable_ws_api
        || !authorized(&state.runtime.settings.access_token, &headers, &query)
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let runtime = Arc::clone(&state.runtime);
    ws.on_upgrade(move |socket| handle_event_socket(runtime, socket, true))
        .into_response()
}

fn authorized(access_token: &str, headers: &HeaderMap, query: &HashMap<String, String>) -> bool {
    if access_token.trim().is_empty() {
        return true;
    }
    if let Some(value) = headers
        .get("authorization")
        .and_then(|item| item.to_str().ok())
    {
        if value.trim() == format!("Bearer {}", access_token) {
            return true;
        }
    }
    query
        .get("access_token")
        .is_some_and(|value| value == access_token)
}

async fn handle_api_socket(runtime: Arc<RuntimeSnapshot>, mut socket: WebSocket) {
    while let Some(Ok(message)) = socket.recv().await {
        let Message::Text(text) = message else {
            continue;
        };
        let response = match serde_json::from_str::<ApiEnvelope>(&text) {
            Ok(envelope) => handle_action_value(&runtime, envelope).await,
            Err(error) => {
                action_response("failed", 1400, json!({ "error": error.to_string() }), None)
            }
        };
        let _ = socket
            .send(Message::Text(serde_json::to_string(&response).unwrap_or_default().into()))
            .await;
    }
}

async fn handle_event_socket(runtime: Arc<RuntimeSnapshot>, socket: WebSocket, universal: bool) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = runtime.event_sender.subscribe();
    let api_runtime = Arc::clone(&runtime);
    let read_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if !universal {
                continue;
            }
            let Message::Text(text) = message else {
                continue;
            };
            if let Ok(envelope) = serde_json::from_str::<ApiEnvelope>(&text) {
                let response = handle_action_value(&api_runtime, envelope).await;
                let _ = api_runtime
                    .event_sender
                    .send(serde_json::to_string(&response).unwrap_or_default());
            }
        }
    });

    send_meta_event(&runtime, "connect");
    while let Ok(message) = events.recv().await {
        if sender.send(Message::Text(message.into())).await.is_err() {
            break;
        }
    }
    read_task.abort();
}

async fn handle_action_response(runtime: &Arc<RuntimeSnapshot>, envelope: ApiEnvelope) -> Response {
    json_response(StatusCode::OK, handle_action_value(runtime, envelope).await)
}

async fn handle_action_value(
    runtime: &Arc<RuntimeSnapshot>,
    envelope: ApiEnvelope,
) -> ActionResponse {
    let result = match envelope.action.as_str() {
        "get_status" | "get_sealantern_status" => Ok(build_app_status(runtime)),
        "get_version_info" => Ok(json!({
            "app_name": "SeaLantern",
            "app_version": crate::utils::app_version::display_version(),
            "protocol_version": "obv11-compat",
        })),
        "get_running_server_count" => Ok(json!({
            "count": global::server_manager().get_running_server_ids_checked().unwrap_or_default().len(),
        })),
        "prepare_start_server" => prepare_confirmation("start_server", &envelope.params),
        "apply_start_server" => apply_confirmation("start_server", &envelope.params),
        "prepare_stop_server" => prepare_confirmation("stop_server", &envelope.params),
        "apply_stop_server" => apply_confirmation("stop_server", &envelope.params),
        "prepare_write_sl_config" => prepare_confirmation("write_sl_config", &envelope.params),
        "apply_write_sl_config" => apply_confirmation("write_sl_config", &envelope.params),
        other => Err(format!("unsupported action: {}", other)),
    };

    match result {
        Ok(data) => action_response("ok", 0, data, envelope.echo),
        Err(error) => action_response("failed", 1404, json!({ "error": error }), envelope.echo),
    }
}

fn build_app_status(runtime: &Arc<RuntimeSnapshot>) -> Value {
    let manager = global::server_manager();
    let servers = manager.get_server_list();
    let running = manager.get_running_server_ids_checked().unwrap_or_default();
    let statuses = servers
        .iter()
        .map(|server| {
            let status = manager.get_server_status(&server.id);
            ServerStatusEntry {
                server_id: server.id.clone(),
                server_name: server.name.clone(),
                status: status_string(&status.status).to_string(),
                detail: status.detail_message,
                error: status.error_message,
            }
        })
        .collect::<Vec<_>>();
    json!({
        "app_version": crate::utils::app_version::display_version(),
        "plugin_id": runtime.plugin_id,
        "plugin_mode": mode_name(&runtime.settings.mode),
        "plugin_started_at": runtime.started_at,
        "server_total": servers.len(),
        "running_server_count": running.len(),
        "running_server_ids": running,
        "server_statuses": statuses,
    })
}

fn prepare_confirmation(action: &str, params: &Value) -> Result<Value, String> {
    let server_id = params
        .get("server_id")
        .or_else(|| params.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "server_id is required".to_string())?;
    let token = format!("obv11-{}-{}", now_secs(), next_counter());
    let ticket = ConfirmationTicket {
        token: token.clone(),
        action: action.to_string(),
        server_id: server_id.to_string(),
        payload: params.clone(),
        expires_at: now_secs() + CONFIRM_TTL_SECS,
    };
    confirmation_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(token.clone(), ticket.clone());
    Ok(
        json!({ "token": token, "expires_at": ticket.expires_at, "action": action, "server_id": server_id }),
    )
}

fn apply_confirmation(expected_action: &str, params: &Value) -> Result<Value, String> {
    let token = params
        .get("token")
        .and_then(Value::as_str)
        .ok_or_else(|| "token is required".to_string())?;
    let ticket = confirmation_registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(token)
        .ok_or_else(|| "confirmation token not found".to_string())?;
    if ticket.expires_at < now_secs() {
        return Err("confirmation token expired".to_string());
    }
    if ticket.action != expected_action {
        return Err(format!(
            "confirmation token action mismatch: expected {} got {}",
            expected_action, ticket.action
        ));
    }
    match expected_action {
        "start_server" => {
            global::server_manager().start_server(&ticket.server_id)?;
            Ok(json!({ "ok": true }))
        }
        "stop_server" => {
            global::server_manager().stop_server(&ticket.server_id)?;
            Ok(json!({ "ok": true }))
        }
        "write_sl_config" => {
            let config = ticket
                .payload
                .get("config")
                .cloned()
                .ok_or_else(|| "config is required".to_string())?;
            let server = global::server_manager().find_server_clone(&ticket.server_id)?;
            let parsed = serde_json::from_value::<
                sea_lantern_server_config_core::types::SLStartupConfig,
            >(config)
            .map_err(|e| format!("invalid startup config: {}", e))?;
            crate::commands::server::config::write_sl_config(server.path.clone(), parsed)?;
            Ok(json!({ "ok": true }))
        }
        _ => Err(format!("unsupported confirmation action: {}", expected_action)),
    }
}

fn dispatch_event(runtime: &Arc<RuntimeSnapshot>, event: Value) {
    let Ok(text) = serde_json::to_string(&event) else {
        return;
    };
    let _ = runtime.event_sender.send(text.clone());
    maybe_forward_http_post(runtime, &text);
    maybe_forward_qq(runtime, &text);
}

fn emit_debug(runtime: &Arc<RuntimeSnapshot>, tag: &str, message: &str) {
    let payload = json!({
        "time": now_secs(),
        "self_id": runtime.settings.self_id,
        "post_type": "meta_event",
        "meta_event_type": "sealantern_debug",
        "sub_type": tag,
        "message": message,
    });
    if let Ok(text) = serde_json::to_string(&payload) {
        let _ = runtime.debug_sender.send(text.clone());
        let _ = runtime.event_sender.send(text.clone());
        maybe_forward_http_post(runtime, &text);
        maybe_forward_qq(runtime, &text);
    }
}

fn send_meta_event(runtime: &Arc<RuntimeSnapshot>, sub_type: &str) {
    dispatch_event(
        runtime,
        json!({
            "time": now_secs(),
            "self_id": runtime.settings.self_id,
            "post_type": "meta_event",
            "meta_event_type": "lifecycle",
            "sub_type": sub_type,
        }),
    );
}

fn maybe_forward_http_post(runtime: &Arc<RuntimeSnapshot>, body: &str) {
    if !runtime.settings.enable_http_post || runtime.settings.http_post_urls.is_empty() {
        return;
    }
    let urls = runtime.settings.http_post_urls.clone();
    let body = body.to_string();
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        for url in urls {
            if let Err(error) = client
                .post(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.clone())
                .send()
            {
                log_plugin_warn("maybe_forward_http_post", &format!("url={} error={}", url, error));
            }
        }
    });
}

fn maybe_forward_qq(runtime: &Arc<RuntimeSnapshot>, body: &str) {
    if !matches!(runtime.settings.mode, Obv11Mode::QqHttpForward)
        || runtime.settings.qq_api_base_url.is_empty()
        || runtime.settings.qq_targets.is_empty()
    {
        return;
    }
    let base = runtime
        .settings
        .qq_api_base_url
        .trim_end_matches('/')
        .to_string();
    let token = runtime.settings.qq_access_token.clone();
    let targets = runtime.settings.qq_targets.clone();
    let body = body.to_string();
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        for target in targets {
            let payload = if target.target_type == "group" {
                json!({ "message_type": "group", "group_id": target.id, "message": body, "auto_escape": false })
            } else {
                json!({ "message_type": "private", "user_id": target.id, "message": body, "auto_escape": false })
            };
            let mut request = client.post(format!("{}/send_msg", base)).json(&payload);
            if !token.is_empty() {
                request = request.bearer_auth(token.as_str());
            }
            if let Err(error) = request.send() {
                log_plugin_warn(
                    "maybe_forward_qq",
                    &format!("target={} error={}", target.id, error),
                );
            }
        }
    });
}

fn map_server_event_to_onebot(
    runtime: &Arc<RuntimeSnapshot>,
    event: &ServerEventEnvelope,
) -> Value {
    match &event.payload {
        ServerEventPayload::Lifecycle { detail, error, from_mode, to_mode } => json!({
            "time": now_secs(),
            "self_id": runtime.settings.self_id,
            "post_type": "notice",
            "notice_type": "sealantern_server_status",
            "server_id": event.server_id,
            "event_kind": format!("{:?}", event.kind).to_ascii_lowercase(),
            "detail": detail,
            "error": error,
            "from_mode": from_mode,
            "to_mode": to_mode,
        }),
        ServerEventPayload::StructuredLog { line, event_kind, player, message, .. } => json!({
            "time": now_secs(),
            "self_id": runtime.settings.self_id,
            "post_type": "notice",
            "notice_type": "sealantern_server_status",
            "server_id": event.server_id,
            "event_kind": event_kind,
            "player": player,
            "message": message,
            "line": line,
        }),
        ServerEventPayload::RawLine { line, stream } => json!({
            "time": now_secs(),
            "self_id": runtime.settings.self_id,
            "post_type": "meta_event",
            "meta_event_type": "sealantern_debug",
            "sub_type": "server_raw_line",
            "server_id": event.server_id,
            "stream": stream,
            "line": line,
        }),
        ServerEventPayload::Command { command, success, error, actor } => json!({
            "time": now_secs(),
            "self_id": runtime.settings.self_id,
            "post_type": "notice",
            "notice_type": "sealantern_command",
            "server_id": event.server_id,
            "command": command,
            "success": success,
            "error": error,
            "actor": actor,
        }),
    }
}

fn action_response(
    status: &'static str,
    retcode: i32,
    data: Value,
    echo: Option<Value>,
) -> ActionResponse {
    ActionResponse { status, retcode, data, echo }
}

fn json_response(status: StatusCode, value: impl Serialize) -> Response {
    (status, Json(value)).into_response()
}

fn mode_name(mode: &Obv11Mode) -> &'static str {
    match mode {
        Obv11Mode::QqHttpForward => "qq_http_forward",
        Obv11Mode::ApiOnly => "api_only",
    }
}

fn status_string(status: &ServerStatus) -> &'static str {
    match status {
        ServerStatus::Stopped => "stopped",
        ServerStatus::Starting => "starting",
        ServerStatus::Running => "running",
        ServerStatus::Stopping => "stopping",
        ServerStatus::Error => "error",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_runtime, default_settings_json, parse_settings, parse_targets, runtime_registry,
        start_runtime_thread, PLUGIN_ID,
    };

    #[test]
    fn default_settings_parse() {
        let settings =
            parse_settings(default_settings_json()).expect("default settings should parse");
        assert_eq!(settings.listen_addr, "127.0.0.1:5710");
        assert!(settings.enable_http_api);
        assert!(settings.enable_ws_api);
    }

    #[test]
    fn target_lines_parse() {
        let targets = parse_targets("group:123\nprivate:456\ninvalid");
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].target_type, "group");
        assert_eq!(targets[0].id, 123);
        assert_eq!(targets[1].target_type, "private");
        assert_eq!(targets[1].id, 456);
    }

    #[test]
    fn start_runtime_thread_surfaces_bind_failures() {
        let occupied = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("occupied listener should bind");
        let addr = occupied
            .local_addr()
            .expect("occupied listener should expose local addr");

        let mut settings =
            parse_settings(default_settings_json()).expect("default settings should parse");
        settings.listen_addr = addr.to_string();

        let runtime = build_runtime(PLUGIN_ID, settings);
        let error = match start_runtime_thread(runtime) {
            Ok(_) => panic!("occupied port should be reported as startup error"),
            Err(error) => error,
        };

        assert!(error.contains("bind failed"), "unexpected error: {}", error);
    }

    #[test]
    fn start_runtime_thread_bind_failure_does_not_register_runtime() {
        let occupied = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("occupied listener should bind");
        let addr = occupied
            .local_addr()
            .expect("occupied listener should expose local addr");

        runtime_registry()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(PLUGIN_ID);

        let mut settings =
            parse_settings(default_settings_json()).expect("default settings should parse");
        settings.listen_addr = addr.to_string();

        let runtime = build_runtime(PLUGIN_ID, settings);
        let result = start_runtime_thread(runtime);

        assert!(result.is_err(), "occupied port should fail startup");
        assert!(
            !runtime_registry()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains_key(PLUGIN_ID),
            "failed startup must not leave runtime registered"
        );
    }
}
