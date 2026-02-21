use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

pub type ApiCallHandler =
    Arc<dyn Fn(&str, &str, &str, Vec<JsonValue>) -> Result<JsonValue, String> + Send + Sync>;

pub type UiEventHandler = Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

pub type LogEventHandler = Arc<dyn Fn(&str, &str, &str) -> Result<(), String> + Send + Sync>;

static API_CALL_HANDLER: RwLock<Option<ApiCallHandler>> = RwLock::new(None);

static UI_EVENT_HANDLER: RwLock<Option<UiEventHandler>> = RwLock::new(None);

static LOG_EVENT_HANDLER: RwLock<Option<LogEventHandler>> = RwLock::new(None);

pub type ContextMenuHandler =
    Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

static CONTEXT_MENU_HANDLER: RwLock<Option<ContextMenuHandler>> = RwLock::new(None);

pub type SidebarEventHandler =
    Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

static SIDEBAR_EVENT_HANDLER: RwLock<Option<SidebarEventHandler>> = RwLock::new(None);

pub type PermissionLogHandler =
    Arc<dyn Fn(&str, &str, &str, &str, u64) -> Result<(), String> + Send + Sync>;

static PERMISSION_LOG_HANDLER: RwLock<Option<PermissionLogHandler>> = RwLock::new(None);

pub type ComponentEventHandler = Arc<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync>;

static COMPONENT_EVENT_HANDLER: RwLock<Option<ComponentEventHandler>> = RwLock::new(None);

pub type ServerReadyHandler = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

static SERVER_READY_HANDLER: RwLock<Option<ServerReadyHandler>> = RwLock::new(None);

pub type I18nEventHandler = Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

static I18N_EVENT_HANDLER: RwLock<Option<I18nEventHandler>> = RwLock::new(None);

#[derive(Clone, serde::Serialize)]
pub struct BufferedUiEvent {
    pub plugin_id: String,
    pub action: String,
    pub element_id: String,
    pub html: String,
}

static UI_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedUiEvent>>> = OnceLock::new();

fn get_ui_snapshot_store() -> &'static Mutex<Vec<BufferedUiEvent>> {
    UI_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn buffer_ui_event(plugin_id: &str, action: &str, element_id: &str, html: &str) {
    let mut store = get_ui_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    match action {
        "inject" | "insert" => {
            store.retain(|e| {
                !(e.plugin_id == plugin_id && e.element_id == element_id && e.action == action)
            });
            store.push(BufferedUiEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                element_id: element_id.to_string(),
                html: html.to_string(),
            });
        }
        "update" => {
            if let Some(existing) = store
                .iter_mut()
                .find(|e| e.plugin_id == plugin_id && e.element_id == element_id)
            {
                existing.html = html.to_string();
                existing.action = "inject".to_string();
            } else {
                store.push(BufferedUiEvent {
                    plugin_id: plugin_id.to_string(),
                    action: "inject".to_string(),
                    element_id: element_id.to_string(),
                    html: html.to_string(),
                });
            }
        }
        "inject_css" => {
            store.retain(|e| {
                !(e.plugin_id == plugin_id
                    && e.element_id == element_id
                    && e.action == "inject_css")
            });
            store.push(BufferedUiEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                element_id: element_id.to_string(),
                html: html.to_string(),
            });
        }
        "remove_css" => {
            store.retain(|e| {
                !(e.plugin_id == plugin_id
                    && e.element_id == element_id
                    && e.action == "inject_css")
            });
        }
        "remove" => {
            store.retain(|e| !(e.plugin_id == plugin_id && e.element_id == element_id));
        }
        "remove_all" => {
            store.retain(|e| e.plugin_id != plugin_id);
        }
        _ => {}
    }
}

pub fn take_ui_event_snapshot() -> Vec<BufferedUiEvent> {
    let store = get_ui_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

#[allow(dead_code)]
pub fn clear_plugin_ui_snapshot(plugin_id: &str) {
    let mut store = get_ui_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}

pub fn set_api_call_handler(handler: ApiCallHandler) {
    let mut h = API_CALL_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn set_ui_event_handler(handler: UiEventHandler) {
    let mut h = UI_EVENT_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn set_log_event_handler(handler: LogEventHandler) {
    let mut h = LOG_EVENT_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn set_context_menu_handler(handler: ContextMenuHandler) {
    {
        let mut h = CONTEXT_MENU_HANDLER.write().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        *h = Some(handler);
    }

    let snapshot = take_context_menu_snapshot();
    if !snapshot.is_empty() {
        let handler = CONTEXT_MENU_HANDLER.read().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        if let Some(handler) = handler.as_ref() {
            for e in snapshot {
                let _ = handler(&e.plugin_id, &e.action, &e.context, &e.items);
            }
        }
    }
}

pub fn set_sidebar_event_handler(handler: SidebarEventHandler) {
    let mut h = SIDEBAR_EVENT_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn set_permission_log_handler(handler: PermissionLogHandler) {
    let mut h = PERMISSION_LOG_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn call_api(
    source_plugin: &str,
    target_plugin: &str,
    api_name: &str,
    args: Vec<JsonValue>,
) -> Result<JsonValue, String> {
    let handler = API_CALL_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(source_plugin, target_plugin, api_name, args),
        None => Err(format!(
            "API '{}' 调用失败: API 调用处理器未初始化 (目标插件: {}, 源插件: {})",
            api_name, target_plugin, source_plugin
        )),
    }
}

pub fn emit_ui_event(
    plugin_id: &str,
    action: &str,
    element_id: &str,
    html: &str,
) -> Result<(), String> {
    buffer_ui_event(plugin_id, action, element_id, html);

    let handler = UI_EVENT_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, element_id, html),
        None => {
            eprintln!(
                "[WARN] UI 事件处理器未初始化，事件已缓冲 (plugin: {}, action: {})",
                plugin_id, action
            );
            Ok(())
        }
    }
}

pub fn emit_log_event(plugin_id: &str, level: &str, message: &str) -> Result<(), String> {
    let handler = LOG_EVENT_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, level, message),
        None => {
            eprintln!(
                "[WARN] 日志事件处理器未初始化，插件 '{}' 的日志 (level: {}) 将被忽略: {}",
                plugin_id, level, message
            );
            Ok(())
        }
    }
}

pub fn emit_context_menu_event(
    plugin_id: &str,
    action: &str,
    context: &str,
    items_json: &str,
) -> Result<(), String> {
    buffer_context_menu_event(plugin_id, action, context, items_json);

    let handler = CONTEXT_MENU_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, context, items_json),
        None => {
            eprintln!(
                "[WARN] 右键菜单事件处理器未初始化，事件已缓冲 (plugin: {}, action: {})",
                plugin_id, action
            );
            Ok(())
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct BufferedContextMenuEvent {
    pub plugin_id: String,
    pub action: String,
    pub context: String,
    pub items: String,
}

static CONTEXT_MENU_SNAPSHOT: OnceLock<Mutex<Vec<BufferedContextMenuEvent>>> = OnceLock::new();

fn get_context_menu_snapshot_store() -> &'static Mutex<Vec<BufferedContextMenuEvent>> {
    CONTEXT_MENU_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

fn buffer_context_menu_event(plugin_id: &str, action: &str, context: &str, items_json: &str) {
    let mut store = get_context_menu_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    match action {
        "register" => {
            store.retain(|e| !(e.plugin_id == plugin_id && e.context == context));
            store.push(BufferedContextMenuEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                context: context.to_string(),
                items: items_json.to_string(),
            });
        }
        "unregister" => {
            store.retain(|e| !(e.plugin_id == plugin_id && e.context == context));
        }
        _ => {}
    }
}

pub fn take_context_menu_snapshot() -> Vec<BufferedContextMenuEvent> {
    let store = get_context_menu_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn clear_plugin_context_menu_snapshot(plugin_id: &str) {
    let mut store = get_context_menu_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}

#[derive(Clone, serde::Serialize)]
pub struct BufferedSidebarEvent {
    pub plugin_id: String,
    pub action: String,
    pub label: String,
    pub icon: String,
}

static SIDEBAR_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedSidebarEvent>>> = OnceLock::new();

fn get_sidebar_snapshot_store() -> &'static Mutex<Vec<BufferedSidebarEvent>> {
    SIDEBAR_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

fn buffer_sidebar_event(plugin_id: &str, action: &str, label: &str, icon: &str) {
    let mut store = get_sidebar_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    match action {
        "register" => {
            store.retain(|e| e.plugin_id != plugin_id);
            store.push(BufferedSidebarEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                label: label.to_string(),
                icon: icon.to_string(),
            });
        }
        "unregister" => {
            store.retain(|e| e.plugin_id != plugin_id);
        }
        _ => {}
    }
}

pub fn take_sidebar_event_snapshot() -> Vec<BufferedSidebarEvent> {
    let store = get_sidebar_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn clear_plugin_sidebar_snapshot(plugin_id: &str) {
    let mut store = get_sidebar_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}

pub fn emit_sidebar_event(
    plugin_id: &str,
    action: &str,
    label: &str,
    icon: &str,
) -> Result<(), String> {
    buffer_sidebar_event(plugin_id, action, label, icon);

    let handler = SIDEBAR_EVENT_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, label, icon),
        None => {
            eprintln!(
                "[WARN] 侧边栏事件处理器未初始化，事件已缓冲 (plugin: {}, action: {})",
                plugin_id, action
            );
            Ok(())
        }
    }
}

pub fn emit_permission_log(
    plugin_id: &str,
    log_type: &str,
    action: &str,
    detail: &str,
) -> Result<(), String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let handler = PERMISSION_LOG_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, log_type, action, detail, timestamp),
        None => Ok(()),
    }
}

pub fn set_component_event_handler(handler: ComponentEventHandler) {
    let mut h = COMPONENT_EVENT_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

#[derive(Clone, serde::Serialize)]
pub struct BufferedComponentEvent {
    pub plugin_id: String,
    pub payload_json: String,
}

static COMPONENT_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedComponentEvent>>> = OnceLock::new();

fn get_component_snapshot_store() -> &'static Mutex<Vec<BufferedComponentEvent>> {
    COMPONENT_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

fn buffer_component_event(plugin_id: &str, payload_json: &str) {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload_json) {
        let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let component_id = parsed
            .get("component_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let prop = parsed.get("prop").and_then(|v| v.as_str()).unwrap_or("");
        let mut store = get_component_snapshot_store()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match action {
            "create" => {
                store.retain(|e| e.plugin_id != plugin_id);
                store.push(BufferedComponentEvent {
                    plugin_id: plugin_id.to_string(),
                    payload_json: payload_json.to_string(),
                });
            }
            "set" => {
                store.retain(|e| {
                    if e.plugin_id != plugin_id {
                        return true;
                    }
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json) {
                        let a = p.get("action").and_then(|v| v.as_str()).unwrap_or("");
                        let c = p.get("component_id").and_then(|v| v.as_str()).unwrap_or("");
                        let pr = p.get("prop").and_then(|v| v.as_str()).unwrap_or("");
                        !(a == "set" && c == component_id && pr == prop)
                    } else {
                        true
                    }
                });
                store.push(BufferedComponentEvent {
                    plugin_id: plugin_id.to_string(),
                    payload_json: payload_json.to_string(),
                });
            }
            _ => {}
        }
    }
}

pub fn take_component_event_snapshot() -> Vec<BufferedComponentEvent> {
    let store = get_component_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn clear_plugin_component_snapshot(plugin_id: &str) {
    let mut store = get_component_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}

pub fn emit_component_event(plugin_id: &str, payload_json: &str) -> Result<(), String> {
    buffer_component_event(plugin_id, payload_json);
    let handler = COMPONENT_EVENT_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, payload_json),
        None => Ok(()),
    }
}

pub fn set_server_ready_handler(handler: ServerReadyHandler) {
    let mut h = SERVER_READY_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn emit_server_ready(server_id: &str) -> Result<(), String> {
    let handler = SERVER_READY_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(server_id),
        None => Ok(()),
    }
}

pub fn set_i18n_event_handler(handler: I18nEventHandler) {
    let mut h = I18N_EVENT_HANDLER.write().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    *h = Some(handler);
}

pub fn emit_i18n_event(
    plugin_id: &str,
    action: &str,
    locale: &str,
    payload: &str,
) -> Result<(), String> {
    let handler = I18N_EVENT_HANDLER.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, locale, payload),
        None => Ok(()),
    }
}

pub type ApiRegistry = Arc<Mutex<HashMap<String, HashMap<String, String>>>>;

pub fn new_api_registry() -> ApiRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

pub trait ApiRegistryOps {
    fn register_api(&self, plugin_id: &str, api_name: &str, lua_fn_name: &str);

    fn has_api(&self, plugin_id: &str, api_name: &str) -> bool;

    fn list_apis(&self, plugin_id: &str) -> Vec<String>;

    fn get_api_fn_name(&self, plugin_id: &str, api_name: &str) -> Option<String>;

    fn clear_plugin_apis(&self, plugin_id: &str);
}

impl ApiRegistryOps for ApiRegistry {
    fn register_api(&self, plugin_id: &str, api_name: &str, lua_fn_name: &str) {
        let mut registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        let plugin_apis = registry.entry(plugin_id.to_string()).or_default();
        plugin_apis.insert(api_name.to_string(), lua_fn_name.to_string());
    }

    fn has_api(&self, plugin_id: &str, api_name: &str) -> bool {
        let registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry
            .get(plugin_id)
            .map(|apis| apis.contains_key(api_name))
            .unwrap_or(false)
    }

    fn list_apis(&self, plugin_id: &str) -> Vec<String> {
        let registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry
            .get(plugin_id)
            .map(|apis| apis.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn get_api_fn_name(&self, plugin_id: &str, api_name: &str) -> Option<String> {
        let registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry
            .get(plugin_id)
            .and_then(|apis| apis.get(api_name).cloned())
    }

    fn clear_plugin_apis(&self, plugin_id: &str) {
        let mut registry = self.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Mutex poisoned, recovering: {}", e);
            e.into_inner()
        });
        registry.remove(plugin_id);
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentEntry {
    pub id: String,
    pub component_type: String,
}

static COMPONENT_MIRROR: RwLock<Vec<ComponentEntry>> = RwLock::new(Vec::new());

pub fn component_mirror_register(id: &str, component_type: &str) {
    let mut mirror = COMPONENT_MIRROR.write().unwrap_or_else(|e| e.into_inner());

    mirror.retain(|e| e.id != id);
    mirror.push(ComponentEntry {
        id: id.to_string(),
        component_type: component_type.to_string(),
    });
}

pub fn component_mirror_unregister(id: &str) {
    let mut mirror = COMPONENT_MIRROR.write().unwrap_or_else(|e| e.into_inner());
    mirror.retain(|e| e.id != id);
}

pub fn component_mirror_list(page_filter: Option<&str>) -> Vec<ComponentEntry> {
    let mirror = COMPONENT_MIRROR.read().unwrap_or_else(|e| e.into_inner());
    match page_filter {
        Some(f) => mirror
            .iter()
            .filter(|e| e.id.starts_with(&format!("{}/", f)))
            .cloned()
            .collect(),
        None => mirror.clone(),
    }
}

pub fn component_mirror_clear() {
    let mut mirror = COMPONENT_MIRROR.write().unwrap_or_else(|e| e.into_inner());
    mirror.clear();
}

static ELEMENT_REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

static ELEMENT_RESPONSE_STORE: OnceLock<Mutex<HashMap<u64, std::sync::mpsc::Sender<String>>>> =
    OnceLock::new();

fn get_element_response_store() -> &'static Mutex<HashMap<u64, std::sync::mpsc::Sender<String>>> {
    ELEMENT_RESPONSE_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn element_response_create() -> (u64, std::sync::mpsc::Receiver<String>) {
    let id = ELEMENT_REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut store = get_element_response_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.insert(id, tx);
    (id, rx)
}

pub fn element_response_resolve(request_id: u64, data: String) {
    let mut store = get_element_response_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(tx) = store.remove(&request_id) {
        let _ = tx.send(data);
    }
}
