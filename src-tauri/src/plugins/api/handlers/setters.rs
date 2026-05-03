use super::state::{
    recover_lock, ApiCallHandler, ComponentEventHandler, ContextMenuHandler, I18nEventHandler,
    LogEventHandler, PermissionLogHandler, ServerReadyHandler, SidebarEventHandler,
    UiEventHandler, API_CALL_HANDLER, COMPONENT_EVENT_HANDLER, CONTEXT_MENU_HANDLER,
    I18N_EVENT_HANDLER, LOG_EVENT_HANDLER, PERMISSION_LOG_HANDLER, SERVER_READY_HANDLER,
    SIDEBAR_EVENT_HANDLER, UI_EVENT_HANDLER,
};
use crate::plugins::api::context_menu::take_context_menu_snapshot;

pub fn set_api_call_handler(handler: ApiCallHandler) {
    let mut h = API_CALL_HANDLER.write().unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_ui_event_handler(handler: UiEventHandler) {
    let mut h = UI_EVENT_HANDLER.write().unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_log_event_handler(handler: LogEventHandler) {
    let mut h = LOG_EVENT_HANDLER.write().unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_context_menu_handler(handler: ContextMenuHandler) {
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

pub fn set_sidebar_event_handler(handler: SidebarEventHandler) {
    let mut h = SIDEBAR_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_permission_log_handler(handler: PermissionLogHandler) {
    let mut h = PERMISSION_LOG_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_component_event_handler(handler: ComponentEventHandler) {
    let mut h = COMPONENT_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_server_ready_handler(handler: ServerReadyHandler) {
    let mut h = SERVER_READY_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}

pub fn set_i18n_event_handler(handler: I18nEventHandler) {
    let mut h = I18N_EVENT_HANDLER
        .write()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    *h = Some(handler);
}
