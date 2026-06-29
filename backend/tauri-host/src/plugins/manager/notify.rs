use super::{PluginManager, PluginState};
use crate::plugins::api::emit_log_event;
use crate::services::events::ServerEventEnvelope;
use crate::utils::logger::{log_error_ctx, log_warn_ctx};

fn recover_runtimes_read_lock<'a>(
    manager: &'a PluginManager,
    function: &str,
) -> std::sync::RwLockReadGuard<
    'a,
    std::collections::HashMap<String, crate::plugins::runtime::PluginRuntime>,
> {
    manager.runtimes.read().unwrap_or_else(|e| {
        log_warn_ctx(
            "plugins.manager.notify",
            function,
            &format!("RwLock poisoned; recovering: {}", e),
        );
        e.into_inner()
    })
}

pub(super) fn notify_page_changed(manager: &PluginManager, path: &str) {
    let runtimes = recover_runtimes_read_lock(manager, "notify_page_changed");

    for (id, runtime) in runtimes.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }

        if let Err(e) = runtime.call_lifecycle_with_arg("onPageChanged", path) {
            log_warn_ctx(
                "plugins.manager.notify",
                "notify_page_changed",
                &format!("failed to call onPageChanged for plugin '{}': {}", id, e),
            );
        }
    }
}

pub(super) fn notify_locale_changed(manager: &PluginManager, locale: &str) {
    let runtimes = recover_runtimes_read_lock(manager, "notify_locale_changed");

    for (id, runtime) in runtimes.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }

        if let Err(e) = runtime.call_lifecycle_with_arg("onLocaleChanged", locale) {
            log_warn_ctx(
                "plugins.manager.notify",
                "notify_locale_changed",
                &format!("failed to call onLocaleChanged for plugin '{}': {}", id, e),
            );
            if let Err(error) =
                emit_log_event(id, "error", &format!("Failed to call onLocaleChanged: {}", e))
            {
                log_error_ctx(
                    "plugins.manager.notify",
                    "notify_locale_changed",
                    &format!(
                        "plugin locale change error log emit failed: plugin_id={} error={}",
                        id, error
                    ),
                );
            }
        }
    }
}

pub(super) fn notify_server_event(manager: &PluginManager, event: &ServerEventEnvelope) {
    let runtimes = recover_runtimes_read_lock(manager, "notify_server_event");

    for (id, runtime) in runtimes.iter() {
        if !plugin_is_enabled(manager, id) {
            continue;
        }

        if let Err(error) = runtime.notify_server_event(event) {
            log_warn_ctx(
                "plugins.manager.notify",
                "notify_server_event",
                &format!(
                    "failed to call onServerEvent for plugin '{}': {}",
                    id, error
                ),
            );
            if let Err(log_error) =
                emit_log_event(id, "error", &format!("Failed to call onServerEvent: {}", error))
            {
                log_error_ctx(
                    "plugins.manager.notify",
                    "notify_server_event",
                    &format!(
                        "plugin server event error log emit failed: plugin_id={} error={}",
                        id, log_error
                    ),
                );
            }
        }
    }
}

fn plugin_is_enabled(manager: &PluginManager, plugin_id: &str) -> bool {
    manager
        .plugins
        .get(plugin_id)
        .is_some_and(|info| matches!(info.state, PluginState::Enabled))
}
