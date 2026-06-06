use serde_json::Value as JsonValue;
use std::sync::{Arc, RwLock};

pub(super) type ApiCallHandler = dyn Fn(&str, &str, &str, Vec<JsonValue>) -> Result<JsonValue, String> + Send + Sync;
pub(super) type UiEventHandler = dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync;
pub(super) type LogEventHandler = dyn Fn(&str, &str, &str) -> Result<(), String> + Send + Sync;
pub(super) type ContextMenuHandler = dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync;
pub(super) type SidebarEventHandler = dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync;
pub(super) type PermissionLogHandler = dyn Fn(&str, &str, &str, &str, u64) -> Result<(), String> + Send + Sync;
pub(super) type ComponentEventHandler = dyn Fn(&str, &str) -> Result<(), String> + Send + Sync;
pub(super) type ServerReadyHandler = dyn Fn(&str) -> Result<(), String> + Send + Sync;
pub(super) type I18nEventHandler = dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync;

pub(super) static API_CALL_HANDLER: RwLock<Option<Arc<ApiCallHandler>>> = RwLock::new(None);
pub(super) static UI_EVENT_HANDLER: RwLock<Option<Arc<UiEventHandler>>> = RwLock::new(None);
pub(super) static LOG_EVENT_HANDLER: RwLock<Option<Arc<LogEventHandler>>> = RwLock::new(None);
pub(super) static CONTEXT_MENU_HANDLER: RwLock<Option<Arc<ContextMenuHandler>>> = RwLock::new(None);
pub(super) static SIDEBAR_EVENT_HANDLER: RwLock<Option<Arc<SidebarEventHandler>>> = RwLock::new(None);
pub(super) static PERMISSION_LOG_HANDLER: RwLock<Option<Arc<PermissionLogHandler>>> = RwLock::new(None);
pub(super) static COMPONENT_EVENT_HANDLER: RwLock<Option<Arc<ComponentEventHandler>>> = RwLock::new(None);
pub(super) static SERVER_READY_HANDLER: RwLock<Option<Arc<ServerReadyHandler>>> = RwLock::new(None);
pub(super) static I18N_EVENT_HANDLER: RwLock<Option<Arc<I18nEventHandler>>> = RwLock::new(None);

pub(super) fn recover_lock<T>(err: std::sync::PoisonError<T>, label: &str) -> T {
    eprintln!("[WARN] {} poisoned, recovering: {}", label, err);
    err.into_inner()
}
