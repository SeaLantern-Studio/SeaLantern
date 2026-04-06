use std::path::PathBuf;

use mlua::{Lua, Table};

#[derive(Clone)]
pub(super) struct PluginsContext {
    pub(super) plugins_root: PathBuf,
    pub(super) plugin_id: String,
}

impl PluginsContext {
    pub(super) fn new(plugins_root: PathBuf, plugin_id: String) -> Self {
        Self { plugins_root, plugin_id }
    }
}

pub(super) fn emit_plugins_log(plugin_id: &str, api_name: &str, resource: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", api_name, resource);
}

pub(super) fn create_plugins_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| format!("Failed to create plugins table: {}", e))
}

pub(super) fn set_plugins_function(
    table: &Table,
    name: &str,
    function: mlua::Function,
    api_name: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| format!("Failed to set {}: {}", api_name, e))
}

pub(super) fn set_plugins_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("plugins", table)
        .map_err(|e| format!("Failed to set sl.plugins: {}", e))
}

pub(super) fn plugin_dir(root: &std::path::Path, target_id: &str) -> Result<PathBuf, mlua::Error> {
    let _ = super::validate_path_static(root, target_id)?;
    Ok(root.join(target_id))
}
