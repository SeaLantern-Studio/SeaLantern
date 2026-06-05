use super::{PluginManager, PluginState};
use crate::plugins::api::emit_log_event;

pub(super) fn notify_page_changed(manager: &PluginManager, path: &str) {
    let runtimes = manager.runtimes.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });

    for (id, runtime) in runtimes.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }

        if let Err(e) = runtime.call_lifecycle_with_arg("onPageChanged", path) {
            eprintln!("[WARN] Failed to call onPageChanged for plugin '{}': {}", id, e);
        }
    }
}

pub(super) fn notify_locale_changed(manager: &PluginManager, locale: &str) {
    let runtimes = manager.runtimes.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });

    for (id, runtime) in runtimes.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }

        if let Err(e) = runtime.call_lifecycle_with_arg("onLocaleChanged", locale) {
            eprintln!("[WARN] Failed to call onLocaleChanged for plugin '{}': {}", id, e);
            let _ = emit_log_event(id, "error", &format!("Failed to call onLocaleChanged: {}", e));
        }
    }
}

fn plugin_is_enabled(manager: &PluginManager, plugin_id: &str) -> bool {
    manager
        .plugins
        .get(plugin_id)
        .is_some_and(|info| matches!(info.state, PluginState::Enabled))
}
