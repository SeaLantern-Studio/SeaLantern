use std::path::{Path, PathBuf};

use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::services::global::i18n_service;
use crate::utils::logger::log_warn_ctx;
use mlua::{Lua, Table};
use std::collections::HashMap;

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

pub(super) fn plugins_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(super) fn plugins_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn plugins_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn emit_plugins_log(plugin_id: &str, api_name: &str, resource: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", api_name, resource);
}

pub(super) fn create_plugins_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| plugins_t1("plugins.runtime.plugins_api.create_table_failed", e.to_string()))
}

pub(super) fn set_plugins_function(
    table: &Table,
    name: &str,
    function: mlua::Function,
    api_name: &str,
) -> Result<(), String> {
    table.set(name, function).map_err(|e| {
        plugins_t2("plugins.runtime.plugins_api.set_api_failed", api_name, e.to_string())
    })
}

pub(super) fn set_plugins_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("plugins", table)
        .map_err(|e| plugins_t1("plugins.runtime.plugins_api.set_namespace_failed", e.to_string()))
}

pub(super) fn plugin_dir(root: &Path, target_id: &str) -> Result<PathBuf, mlua::Error> {
    let _ = super::validate_path_static(root, target_id)?;
    let path = root.join(target_id);

    if !path.exists() {
        return Err(mlua::Error::runtime(plugins_t1(
            "plugins.runtime.plugins_api.plugin_dir_not_found",
            target_id,
        )));
    }

    if !path.is_dir() {
        return Err(mlua::Error::runtime(plugins_t1(
            "plugins.runtime.plugins_api.plugin_path_not_dir",
            target_id,
        )));
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
    let manifest_path = plugin_dir.join(PLUGIN_MANIFEST_FILE_NAME);
    if !manifest_path.exists() {
        return Ok(None);
    }

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            if let Some(warn_context) = warn_context {
                log_warn_ctx(
                    "plugins.runtime.plugins_api.common",
                    warn_context,
                    &format!("failed to read manifest '{}' : {}", manifest_path.display(), e),
                );
                return Ok(None);
            }

            return Err(mlua::Error::runtime(plugins_t1(
                "plugins.runtime.plugins_api.manifest_read_failed",
                e.to_string(),
            )));
        }
    };

    match serde_json::from_str(&content) {
        Ok(manifest) => Ok(Some(manifest)),
        Err(e) => {
            if let Some(warn_context) = warn_context {
                log_warn_ctx(
                    "plugins.runtime.plugins_api.common",
                    warn_context,
                    &format!("failed to parse manifest '{}' : {}", manifest_path.display(), e),
                );
                return Ok(None);
            }

            Err(mlua::Error::runtime(plugins_t1(
                "plugins.runtime.plugins_api.manifest_parse_failed",
                e.to_string(),
            )))
        }
    }
}
