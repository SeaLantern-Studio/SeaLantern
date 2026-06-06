use crate::plugins::manager::lifecycle::runtime::{disable_plugin_internal, enable_plugin};
use crate::plugins::manager::PluginManager;
use std::collections::HashSet;

pub(in crate::plugins::manager::lifecycle) fn save_enabled_plugins_checked(
    manager: &PluginManager,
) -> Result<(), String> {
    let enabled: Vec<&str> = manager
        .plugins
        .iter()
        .filter(|(_, info)| matches!(info.state, crate::models::plugin::PluginState::Enabled))
        .map(|(id, _)| id.as_str())
        .collect();
    let path = manager.data_dir.join("enabled_plugins.json");
    let json = serde_json::to_string(&enabled)
        .map_err(|e| format!("Failed to serialize enabled plugins '{}': {}", path.display(), e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to save enabled plugins '{}': {}", path.display(), e))
}

#[allow(dead_code)]
pub(in crate::plugins::manager::lifecycle) fn save_enabled_plugins(manager: &PluginManager) {
    if let Err(error) = save_enabled_plugins_checked(manager) {
        eprintln!("[WARN] {}", error);
    }
}

pub(super) fn load_enabled_plugin_ids_checked(
    manager: &PluginManager,
) -> Result<Vec<String>, String> {
    let path = manager.data_dir.join("enabled_plugins.json");
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!("Failed to read enabled plugins '{}': {}", path.display(), error));
        }
    };

    serde_json::from_str::<Vec<String>>(&json)
        .map_err(|error| format!("Failed to parse enabled plugins '{}': {}", path.display(), error))
}

#[allow(dead_code)]
pub(super) fn load_enabled_plugin_ids(manager: &PluginManager) -> Vec<String> {
    load_enabled_plugin_ids_checked(manager).unwrap_or_default()
}

pub(in crate::plugins::manager) fn auto_enable_plugins_checked(
    manager: &mut PluginManager,
) -> Result<(), String> {
    let ids = load_enabled_plugin_ids_checked(manager)?;
    if ids.is_empty() {
        return Ok(());
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
                if let Err(e) = enable_plugin(manager, &id) {
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

    Ok(())
}

#[allow(dead_code)]
pub(in crate::plugins::manager) fn auto_enable_plugins(manager: &mut PluginManager) {
    if let Err(error) = auto_enable_plugins_checked(manager) {
        eprintln!("[WARN] Failed to auto-enable plugins: {}", error);
    }
}

pub(in crate::plugins::manager) fn disable_all_plugins_for_shutdown(manager: &mut PluginManager) {
    let enabled_ids: Vec<String> = manager
        .plugins
        .iter()
        .filter(|(_, info)| matches!(info.state, crate::models::plugin::PluginState::Enabled))
        .map(|(id, _)| id.clone())
        .collect();
    for id in enabled_ids {
        let mut visited = HashSet::new();
        if let Err(e) = disable_plugin_internal(manager, &id, &mut visited) {
            eprintln!("[WARN] Failed to disable plugin '{}' during shutdown: {}", id, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{auto_enable_plugins_checked, load_enabled_plugin_ids_checked};
    use crate::plugins::manager::PluginManager;

    #[test]
    fn load_enabled_plugin_ids_checked_surfaces_invalid_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        let manager = PluginManager::new(plugins_dir, data_dir.clone());

        std::fs::write(data_dir.join("enabled_plugins.json"), "{")
            .expect("broken enabled_plugins.json should exist");

        let error = load_enabled_plugin_ids_checked(&manager)
            .expect_err("invalid enabled plugin state should not be treated as an empty list");

        assert!(error.contains("Failed to parse enabled plugins"));
        assert!(error.contains("enabled_plugins.json"));
    }

    #[test]
    fn auto_enable_plugins_checked_surfaces_invalid_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        let mut manager = PluginManager::new(plugins_dir, data_dir.clone());

        std::fs::write(data_dir.join("enabled_plugins.json"), "{")
            .expect("broken enabled_plugins.json should exist");

        let error = auto_enable_plugins_checked(&mut manager)
            .expect_err("invalid enabled plugin state should abort auto-enable");

        assert!(error.contains("Failed to parse enabled plugins"));
        assert!(error.contains("enabled_plugins.json"));
    }
}
