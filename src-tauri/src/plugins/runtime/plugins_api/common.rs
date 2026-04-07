use std::path::{Path, PathBuf};

use crate::utils::logger::log_warn;
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

pub(super) fn plugin_dir(root: &Path, target_id: &str) -> Result<PathBuf, mlua::Error> {
    let _ = super::validate_path_static(root, target_id)?;
    let path = root.join(target_id);

    if !path.exists() {
        return Err(mlua::Error::runtime(format!("Plugin directory not found: {}", target_id)));
    }

    if !path.is_dir() {
        return Err(mlua::Error::runtime(format!("Plugin path is not a directory: {}", target_id)));
    }

    Ok(path)
}

pub(super) fn resolve_plugin_path(
    root: &Path,
    target_id: &str,
    relative_path: &str,
) -> Result<PathBuf, mlua::Error> {
    let target_dir = plugin_dir(root, target_id)?;
    super::validate_path_static(&target_dir, relative_path)
}

pub(super) fn read_manifest_json(
    plugin_dir: &Path,
    warn_context: Option<&str>,
) -> Result<Option<serde_json::Value>, mlua::Error> {
    let manifest_path = plugin_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Ok(None);
    }

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            if let Some(warn_context) = warn_context {
                log_warn(&format!(
                    "[{}] Failed to read manifest '{}': {}",
                    warn_context,
                    manifest_path.display(),
                    e
                ));
                return Ok(None);
            }

            return Err(mlua::Error::runtime(format!("Failed to read manifest: {}", e)));
        }
    };

    match serde_json::from_str(&content) {
        Ok(manifest) => Ok(Some(manifest)),
        Err(e) => {
            if let Some(warn_context) = warn_context {
                log_warn(&format!(
                    "[{}] Failed to parse manifest '{}': {}",
                    warn_context,
                    manifest_path.display(),
                    e
                ));
                return Ok(None);
            }

            Err(mlua::Error::runtime(format!("Failed to parse manifest: {}", e)))
        }
    }
}
