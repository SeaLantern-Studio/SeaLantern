use super::state::{
    recover_lock, API_CALL_HANDLER, COMPONENT_EVENT_HANDLER, CONTEXT_MENU_HANDLER,
    I18N_EVENT_HANDLER, LOG_EVENT_HANDLER, PERMISSION_LOG_HANDLER, SERVER_READY_HANDLER,
    SIDEBAR_EVENT_HANDLER, UI_EVENT_HANDLER,
};
use crate::plugins::api::component_events::buffer_component_event;
use crate::plugins::api::context_menu::buffer_context_menu_event;
use crate::plugins::api::permission_logs::buffer_permission_log;
use crate::plugins::api::sidebar::buffer_sidebar_event;
use crate::plugins::api::ui_snapshot::buffer_ui_event;
use crate::services::global::i18n_service;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

fn plugin_emit_t3(
    key: &str,
    a: impl Into<String>,
    b: impl Into<String>,
    c: impl Into<String>,
) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    m.insert("2".to_string(), c.into());
    i18n_service().t_with_options(key, &m)
}

pub fn call_api(
    source_plugin: &str,
    target_plugin: &str,
    api_name: &str,
    args: Vec<JsonValue>,
) -> Result<JsonValue, String> {
    let handler = API_CALL_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(source_plugin, target_plugin, api_name, args),
        None => Err(plugin_emit_t3(
            "plugin.bridge.api_handler_missing",
            api_name,
            target_plugin,
            source_plugin,
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

    let handler = UI_EVENT_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, element_id, html),
        None => {
            eprintln!(
                "{}",
                plugin_emit_t3("plugin.bridge.ui_handler_missing_buffered", plugin_id, action, "",)
            );
            Ok(())
        }
    }
}

pub fn emit_log_event(plugin_id: &str, level: &str, message: &str) -> Result<(), String> {
    let handler = LOG_EVENT_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, level, message),
        None => {
            eprintln!(
                "{}",
                plugin_emit_t3(
                    "plugin.bridge.log_handler_missing_ignored",
                    plugin_id,
                    level,
                    message,
                )
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

    let handler = CONTEXT_MENU_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, context, items_json),
        None => {
            eprintln!(
                "{}",
                plugin_emit_t3(
                    "plugin.bridge.context_menu_handler_missing_buffered",
                    plugin_id,
                    action,
                    "",
                )
            );
            Ok(())
        }
    }
}

pub fn emit_sidebar_event(
    plugin_id: &str,
    action: &str,
    label: &str,
    icon: &str,
) -> Result<(), String> {
    buffer_sidebar_event(plugin_id, action, label, icon);

    let handler = SIDEBAR_EVENT_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, label, icon),
        None => {
            eprintln!(
                "{}",
                plugin_emit_t3(
                    "plugin.bridge.sidebar_handler_missing_buffered",
                    plugin_id,
                    action,
                    "",
                )
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

    buffer_permission_log(plugin_id, log_type, action, detail, timestamp);

    let handler = PERMISSION_LOG_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, log_type, action, detail, timestamp),
        None => Ok(()),
    }
}

pub fn emit_component_event(plugin_id: &str, payload_json: &str) -> Result<(), String> {
    buffer_component_event(plugin_id, payload_json).map_err(|error| {
        plugin_emit_t3("plugin.bridge.component_payload_invalid", plugin_id, error.to_string(), "")
    })?;
    let handler = COMPONENT_EVENT_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, payload_json),
        None => Ok(()),
    }
}

pub fn emit_server_ready(server_id: &str) -> Result<(), String> {
    let handler = SERVER_READY_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(server_id),
        None => Ok(()),
    }
}

pub fn emit_i18n_event(
    plugin_id: &str,
    action: &str,
    locale: &str,
    payload: &str,
) -> Result<(), String> {
    let handler = I18N_EVENT_HANDLER
        .read()
        .unwrap_or_else(|e| recover_lock(e, "RwLock"));
    match handler.as_ref() {
        Some(h) => h(plugin_id, action, locale, payload),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::emit_component_event;
    use crate::plugins::api::{clear_plugin_component_snapshot, take_component_event_snapshot};

    #[test]
    fn emit_component_event_surfaces_invalid_payload_instead_of_emitting_null() {
        clear_plugin_component_snapshot("plugin-a");

        let error = emit_component_event("plugin-a", "{")
            .expect_err("invalid component event should fail explicitly");

        assert!(error.contains("plugin-a"));
        assert!(take_component_event_snapshot()
            .iter()
            .all(|event| event.plugin_id != "plugin-a"));
    }
}
