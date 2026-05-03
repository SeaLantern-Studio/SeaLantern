use super::shared::{
    call_on_disable, cleanup_runtime_resources, clear_plugin_side_effects, mark_plugin_disabled,
};
use super::super::super::{PluginManager, PluginState};
use std::collections::HashSet;

pub(in crate::plugins::manager::lifecycle) fn disable_plugin(
    manager: &mut PluginManager,
    plugin_id: &str,
) -> Result<Vec<String>, String> {
    let mut visited = HashSet::new();
    let disabled_plugins = disable_plugin_internal(manager, plugin_id, &mut visited)?;
    super::super::save_enabled_plugins(manager);
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

    let dependent_ids = super::super::get_dependent_plugin_ids(manager, plugin_id);

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
