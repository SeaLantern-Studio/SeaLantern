use super::PluginManager;
use std::collections::HashSet;

pub(super) fn save_enabled_plugins(manager: &PluginManager) {
    let enabled: Vec<&str> = manager
        .plugins
        .iter()
        .filter(|(_, info)| matches!(info.state, crate::models::plugin::PluginState::Enabled))
        .map(|(id, _)| id.as_str())
        .collect();
    let path = manager.data_dir.join("enabled_plugins.json");
    match serde_json::to_string(&enabled) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("[WARN] Failed to save enabled plugins: {}", e);
            }
        }
        Err(e) => eprintln!("[WARN] Failed to serialize enabled plugins: {}", e),
    }
}

pub(super) fn load_enabled_plugin_ids(manager: &PluginManager) -> Vec<String> {
    let path = manager.data_dir.join("enabled_plugins.json");
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str::<Vec<String>>(&json).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(super) fn auto_enable_plugins(manager: &mut PluginManager) {
    let ids = load_enabled_plugin_ids(manager);
    if ids.is_empty() {
        return;
    }

    let mut enabled_set: HashSet<String> = HashSet::new();
    let mut remaining: Vec<String> = ids;
    let mut max_passes = remaining.len() + 1;
    while !remaining.is_empty() && max_passes > 0 {
        max_passes -= 1;
        let mut next = Vec::new();
        for id in remaining {
            let deps_ok = if let Some(info) = manager.plugins.get(&id) {
                info.manifest
                    .dependencies
                    .iter()
                    .all(|d| enabled_set.contains(d.id()))
            } else {
                false
            };
            if deps_ok {
                if let Err(e) = super::enable_plugin(manager, &id) {
                    eprintln!("[WARN] Auto-enable plugin '{}' failed: {}", id, e);
                } else {
                    enabled_set.insert(id);
                }
            } else {
                next.push(id);
            }
        }
        remaining = next;
    }

    for id in remaining {
        eprintln!("[WARN] Auto-enable skipped '{}': dependencies not met", id);
    }
}

pub(super) fn disable_all_plugins_for_shutdown(manager: &mut PluginManager) {
    let enabled_ids: Vec<String> = manager
        .plugins
        .iter()
        .filter(|(_, info)| matches!(info.state, crate::models::plugin::PluginState::Enabled))
        .map(|(id, _)| id.clone())
        .collect();
    for id in enabled_ids {
        let mut visited = HashSet::new();
        if let Err(e) = super::disable_plugin_internal(manager, &id, &mut visited) {
            eprintln!("[WARN] Failed to disable plugin '{}' during shutdown: {}", id, e);
        }
    }
}
