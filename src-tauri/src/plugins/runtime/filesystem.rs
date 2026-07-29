use super::PluginRuntime;
use mlua::Table;

mod common;
mod read;
mod transfer;
mod write;

pub(crate) use common::has_any_fs_permission;
use common::{create_fs_table, set_fs_function, set_fs_table, FsContext};

impl PluginRuntime {
    pub(super) fn setup_fs_namespace(&self, sl: &Table) -> Result<(), String> {
        let fs_table = create_fs_table(&self.lua)?;
        let ctx = FsContext::new(
            self.data_dir.clone(),
            self.server_dir.clone(),
            self.global_dir.clone(),
            self.plugin_id.clone(),
            self.permissions.clone(),
        );

        set_fs_function(&fs_table, "read", read::read(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "read_binary", read::read_binary(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "exists", read::exists(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "list", read::list(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "info", read::info(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "get_path", read::get_path(&self.lua, &ctx)?)?;

        set_fs_function(&fs_table, "write", write::write(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "mkdir", write::mkdir(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "remove", write::remove(&self.lua, &ctx)?)?;

        set_fs_function(&fs_table, "copy", transfer::copy(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "move", transfer::move_entry(&self.lua, &ctx)?)?;
        set_fs_function(&fs_table, "rename", transfer::rename_entry(&self.lua, &ctx)?)?;

        set_fs_table(sl, fs_table)
    }
}
