use super::super::super::{PluginManager, PluginState};
use crate::plugins::api::{emit_log_event, emit_ui_event, ApiRegistryOps};
use crate::plugins::runtime::kill_all_processes;

pub(super) fn call_on_disable(manager: &PluginManager, plugin_id: &str) {
    let runtimes = manager.runtimes.read().unwrap_or_else(|e| {
        eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
        e.into_inner()
    });
    if let Some(runtime) = runtimes.get(plugin_id) {
        if let Err(e) = runtime.call_lifecycle("onDisable") {
            eprintln!("Error calling onDisable for '{}': {}", plugin_id, e);

            let error_msg = format!("Failed to call onDisable: {}", e);
            let _ = emit_log_event(plugin_id, "error", &error_msg);
        }
    }
}

pub(super) fn clear_plugin_side_effects(manager: &mut PluginManager, plugin_id: &str) {
    if let Err(e) = emit_ui_event(plugin_id, "remove_all", "", "") {
        eprintln!("[WARN] Failed to emit remove_all UI event for '{}': {}", plugin_id, e);
    }

    crate::plugins::api::clear_plugin_sidebar_snapshot(plugin_id);
    crate::plugins::api::clear_plugin_component_snapshot(plugin_id);
    crate::plugins::api::clear_plugin_context_menu_snapshot(plugin_id);

    manager.api_registry.clear_plugin_apis(plugin_id);
}

pub(super) fn cleanup_runtime_resources(manager: &mut PluginManager, plugin_id: &str) {
    {
        let runtimes = manager.runtimes.read().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        if let Some(runtime) = runtimes.get(plugin_id) {
            runtime.cleanup();
            kill_all_processes(&runtime.process_registry);
        }
    }

    {
        let mut runtimes = manager.runtimes.write().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        runtimes.remove(plugin_id);
    }
}

pub(super) fn mark_plugin_disabled(manager: &mut PluginManager, plugin_id: &str) {
    if let Some(info) = manager.plugins.get_mut(plugin_id) {
        info.state = PluginState::Disabled;
    }
}
