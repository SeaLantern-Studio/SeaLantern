mod catalog;
mod dependencies;
mod persistence;
mod runtime;
mod scan;

use super::{PluginInfo, PluginManager, PluginState};
use crate::models::plugin::PluginDependency;

pub(super) fn scan_plugins(manager: &mut PluginManager) -> Result<Vec<PluginInfo>, String> {
    scan::scan_plugins(manager)
}

pub(super) fn update_all_missing_dependencies(manager: &mut PluginManager) {
    dependencies::update_all_missing_dependencies(manager)
}

pub(super) fn enable_plugin(manager: &mut PluginManager, plugin_id: &str) -> Result<(), String> {
    runtime::enable_plugin(manager, plugin_id)
}

pub(super) fn disable_plugin(
    manager: &mut PluginManager,
    plugin_id: &str,
) -> Result<Vec<String>, String> {
    runtime::disable_plugin(manager, plugin_id)
}

pub(super) fn disable_plugin_internal(
    manager: &mut PluginManager,
    plugin_id: &str,
    visited: &mut std::collections::HashSet<String>,
) -> Result<Vec<String>, String> {
    runtime::disable_plugin_internal(manager, plugin_id, visited)
}

pub(super) fn check_dependencies(
    manager: &PluginManager,
    dependencies: &[PluginDependency],
) -> Vec<String> {
    dependencies::check_dependencies(manager, dependencies)
}

pub(super) fn get_dependent_plugin_ids(manager: &PluginManager, plugin_id: &str) -> Vec<String> {
    dependencies::get_dependent_plugin_ids(manager, plugin_id)
}

pub(super) fn save_enabled_plugins(manager: &PluginManager) {
    persistence::save_enabled_plugins(manager)
}

pub(super) fn auto_enable_plugins(manager: &mut PluginManager) {
    persistence::auto_enable_plugins(manager)
}

pub(super) fn disable_all_plugins_for_shutdown(manager: &mut PluginManager) {
    persistence::disable_all_plugins_for_shutdown(manager)
}

pub(super) fn get_plugin_list(manager: &PluginManager) -> Vec<PluginInfo> {
    dependencies::get_plugin_list(manager)
}

pub(super) fn get_nav_items(manager: &PluginManager) -> Vec<serde_json::Value> {
    catalog::get_nav_items(manager)
}
