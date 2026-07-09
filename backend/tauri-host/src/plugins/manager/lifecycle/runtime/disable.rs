use super::super::super::{PluginManager, PluginState};
use super::shared::{
    call_on_disable, cleanup_runtime_resources, clear_plugin_side_effects, mark_plugin_disabled,
};
use crate::plugins::manager::lifecycle::dependencies::get_dependent_plugin_ids;
use crate::plugins::manager::lifecycle::persistence::save_enabled_plugins_checked;
use std::collections::HashSet;

pub(in crate::plugins::manager) fn disable_plugin(
    manager: &mut PluginManager,
    plugin_id: &str,
) -> Result<Vec<String>, String> {
    let mut visited = HashSet::new();
    let disabled_plugins = disable_plugin_internal(manager, plugin_id, &mut visited)?;
    save_enabled_plugins_checked(manager)?;
    Ok(disabled_plugins)
}

pub(in crate::plugins::manager::lifecycle) fn disable_plugin_internal(
    manager: &mut PluginManager,
    plugin_id: &str,
    visited: &mut HashSet<String>,
) -> Result<Vec<String>, String> {
    if visited.contains(plugin_id) {
        return Ok(Vec::new());
    }
    visited.insert(plugin_id.to_string());

    let mut disabled_plugins = Vec::new();

    let plugin_info = manager
        .plugins
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

    if !matches!(plugin_info.state, PluginState::Enabled) {
        return Ok(disabled_plugins);
    }

    let dependent_ids = get_dependent_plugin_ids(manager, plugin_id);

    for dep_id in dependent_ids {
        disabled_plugins.push(dep_id.clone());

        if let Ok(mut cascaded) = disable_plugin_internal(manager, &dep_id, visited) {
            disabled_plugins.append(&mut cascaded);
        }
    }

    call_on_disable(manager, plugin_id);
    clear_plugin_side_effects(manager, plugin_id);
    cleanup_runtime_resources(manager, plugin_id);
    mark_plugin_disabled(manager, plugin_id);

    Ok(disabled_plugins)
}

#[cfg(test)]
mod tests {
    use super::disable_plugin;
    use crate::plugins::manager::PluginManager;

    #[test]
    fn disable_plugin_surfaces_enabled_plugin_persistence_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = temp_dir.path().join("plugins");
        let data_dir = temp_dir.path().join("plugin-data");
        std::fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");
        std::fs::create_dir_all(&data_dir).expect("plugin data dir should exist");

        let plugin_root = plugins_dir.join("example-plugin");
        std::fs::create_dir_all(&plugin_root).expect("plugin root should exist");
        std::fs::create_dir_all(data_dir.join("enabled_plugins.json"))
            .expect("directory-backed enabled plugins path should exist");

        let mut manager = PluginManager::new(plugins_dir, data_dir);
        manager.plugins.insert(
            "example-plugin".to_string(),
            super::super::test_support::example_local_plugin_info(
                &plugin_root,
                crate::plugins::manager::PluginState::Enabled,
            ),
        );

        let error = disable_plugin(&mut manager, "example-plugin")
            .expect_err("enabled plugin persistence failure should not be silently downgraded");

        assert!(error.contains("Failed to save enabled plugins"), "unexpected error: {}", error);
        assert!(error.contains("enabled_plugins.json"), "unexpected error: {}", error);
    }
}
