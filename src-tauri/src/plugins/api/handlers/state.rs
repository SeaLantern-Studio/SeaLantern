use serde_json::Value as JsonValue;
use std::sync::{Arc, RwLock};

pub type ApiCallHandler =
    Arc<dyn Fn(&str, &str, &str, Vec<JsonValue>) -> Result<JsonValue, String> + Send + Sync>;

pub type UiEventHandler = Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

pub type LogEventHandler = Arc<dyn Fn(&str, &str, &str) -> Result<(), String> + Send + Sync>;

pub type ContextMenuHandler =
    Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

pub type SidebarEventHandler =
    Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

pub type PermissionLogHandler =
    Arc<dyn Fn(&str, &str, &str, &str, u64) -> Result<(), String> + Send + Sync>;

pub type ComponentEventHandler = Arc<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync>;

pub type ServerReadyHandler = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

pub type I18nEventHandler =
    Arc<dyn Fn(&str, &str, &str, &str) -> Result<(), String> + Send + Sync>;

pub(super) static API_CALL_HANDLER: RwLock<Option<ApiCallHandler>> = RwLock::new(None);
pub(super) static UI_EVENT_HANDLER: RwLock<Option<UiEventHandler>> = RwLock::new(None);
pub(super) static LOG_EVENT_HANDLER: RwLock<Option<LogEventHandler>> = RwLock::new(None);
pub(super) static CONTEXT_MENU_HANDLER: RwLock<Option<ContextMenuHandler>> = RwLock::new(None);
pub(super) static SIDEBAR_EVENT_HANDLER: RwLock<Option<SidebarEventHandler>> = RwLock::new(None);
pub(super) static PERMISSION_LOG_HANDLER: RwLock<Option<PermissionLogHandler>> =
    RwLock::new(None);
pub(super) static COMPONENT_EVENT_HANDLER: RwLock<Option<ComponentEventHandler>> =
    RwLock::new(None);
pub(super) static SERVER_READY_HANDLER: RwLock<Option<ServerReadyHandler>> = RwLock::new(None);
pub(super) static I18N_EVENT_HANDLER: RwLock<Option<I18nEventHandler>> = RwLock::new(None);

pub(super) fn recover_lock<T>(err: std::sync::PoisonError<T>, label: &str) -> T {
    eprintln!("[WARN] {} poisoned, recovering: {}", label, err);
    err.into_inner()
}
