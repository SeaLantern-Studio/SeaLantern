use super::state::{
    recover_lock, ApiCallHandler, API_CALL_HANDLER, ComponentEventHandler,
    COMPONENT_EVENT_HANDLER, ContextMenuHandler, CONTEXT_MENU_HANDLER, I18nEventHandler,
    I18N_EVENT_HANDLER, LogEventHandler, LOG_EVENT_HANDLER, PermissionLogHandler,
    PERMISSION_LOG_HANDLER, ServerReadyHandler, SERVER_READY_HANDLER, SidebarEventHandler,
    SIDEBAR_EVENT_HANDLER, UiEventHandler, UI_EVENT_HANDLER,
};
use crate::plugins::api::context_menu::take_context_menu_snapshot;
use std::sync::Arc;

pub fn set_api_call_handler(handler: Arc<ApiCallHandler>) {
    let mut h = API_CALL_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_ui_event_handler(handler: Arc<UiEventHandler>) {
    let mut h = UI_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_log_event_handler(handler: Arc<LogEventHandler>) {
    let mut h = LOG_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_context_menu_handler(handler: Arc<ContextMenuHandler>) {
    {
        let mut h = CONTEXT_MENU_HANDLER
            .write()
            .unwrap_or_else(|e| recover_lock(e, "RwLock"));
        *h = Some(handler);
    }

    let snapshot = take_context_menu_snapshot();
    if !snapshot.is_empty() {
        let handler = CONTEXT_MENU_HANDLER
            .read()
            .unwrap_or_else(|e| recover_lock(e, "RwLock"));
        if let Some(handler) = handler.as_ref() {
            for e in snapshot {
                let _ = handler(&e.plugin_id, &e.action, &e.context, &e.items);
            }
        }
    }
}

pub fn set_sidebar_event_handler(handler: Arc<SidebarEventHandler>) {
    let mut h = SIDEBAR_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_permission_log_handler(handler: Arc<PermissionLogHandler>) {
    let mut h = PERMISSION_LOG_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_component_event_handler(handler: Arc<ComponentEventHandler>) {
    let mut h = COMPONENT_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_server_ready_handler(handler: Arc<ServerReadyHandler>) {
    let mut h = SERVER_READY_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_i18n_event_handler(handler: Arc<I18nEventHandler>) {
    let mut h = I18N_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}
