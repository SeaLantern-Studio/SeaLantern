use super::{PluginManager, PluginState};
use crate::services::events::ServerEventEnvelope;

pub(super) fn notify_page_changed(manager: &PluginManager, path: &str) {
    for (id, info) in manager.plugins.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }
        if !manager
            .runtime_driver_for(info)
            .runtime_capabilities()
            .supports_page_events
        {
            continue;
        }
        manager
            .runtime_driver_for(info)
            .notify_page_changed(manager, id, path);
    }
}

pub(super) fn notify_locale_changed(manager: &PluginManager, locale: &str) {
    for (id, info) in manager.plugins.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }
        if !manager
            .runtime_driver_for(info)
            .runtime_capabilities()
            .supports_locale_events
        {
            continue;
        }
        manager
            .runtime_driver_for(info)
            .notify_locale_changed(manager, id, locale);
    }
}

pub(super) fn notify_server_event(manager: &PluginManager, event: &ServerEventEnvelope) {
    for (id, info) in manager.plugins.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }
        if !manager
            .runtime_driver_for(info)
            .runtime_capabilities()
            .supports_server_events
        {
            continue;
        }
        manager
            .runtime_driver_for(info)
            .notify_server_event(manager, id, event);
    }
}

pub(super) fn notify_context_menu_show(
    manager: &PluginManager,
    context: &str,
    target_data: &serde_json::Value,
    x: f64,
    y: f64,
) {
    for (id, info) in manager.plugins.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }
        if !manager
            .runtime_driver_for(info)
            .runtime_capabilities()
            .supports_context_menu
        {
            continue;
        }
        manager
            .runtime_driver_for(info)
            .notify_context_menu_show(manager, id, context, target_data, x, y);
    }
}

pub(super) fn notify_context_menu_hide(manager: &PluginManager) {
    for (id, info) in manager.plugins.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }
        if !manager
            .runtime_driver_for(info)
            .runtime_capabilities()
            .supports_context_menu
        {
            continue;
        }
        manager
            .runtime_driver_for(info)
            .notify_context_menu_hide(manager, id);
    }
}

fn plugin_is_enabled(manager: &PluginManager, plugin_id: &str) -> bool {
    manager
        .plugins
        .get(plugin_id)
        .is_some_and(|info| matches!(info.state, PluginState::Enabled))
}
