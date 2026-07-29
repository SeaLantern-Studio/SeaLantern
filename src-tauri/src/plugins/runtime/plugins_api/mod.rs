use super::shared::validate_path_static;
use super::PluginRuntime;
use mlua::Table;
use std::fs;

mod common;
mod query;
mod write;

use common::{create_plugins_table, set_plugins_function, set_plugins_table, PluginsContext};

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

impl PluginRuntime {
    pub(super) fn setup_plugins_namespace(&self, sl: &Table) -> Result<(), String> {
        let plugins_table = create_plugins_table(&self.lua)?;
        let plugins_root = self
            .plugin_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.plugin_dir.clone());
        let ctx = PluginsContext::new(plugins_root, self.plugin_id.clone());

        set_plugins_function(
            &plugins_table,
            "list",
            query::list(&self.lua, &ctx)?,
            "plugins.list",
        )?;
        set_plugins_function(
            &plugins_table,
            "get_manifest",
            query::get_manifest(&self.lua, &ctx)?,
            "plugins.get_manifest",
        )?;
        set_plugins_function(
            &plugins_table,
            "read_file",
            query::read_file(&self.lua, &ctx)?,
            "plugins.read_file",
        )?;
        set_plugins_function(
            &plugins_table,
            "file_exists",
            query::file_exists(&self.lua, &ctx)?,
            "plugins.file_exists",
        )?;
        set_plugins_function(
            &plugins_table,
            "list_files",
            query::list_files(&self.lua, &ctx)?,
            "plugins.list_files",
        )?;

        set_plugins_function(
            &plugins_table,
            "write_file",
            write::write_file(&self.lua, &ctx)?,
            "plugins.write_file",
        )?;

        set_plugins_table(sl, plugins_table)
    }
}
