//! sl.server 命名空间装配入口
//!
//! 职责：
//! - 创建 Lua 表 sl.server，并按职责挂载服务器管理能力
//! - 保持入口轻量，不在此处堆积具体文件/日志闭包实现

use super::PluginRuntime;
use mlua::Table;

mod common;
mod files;
mod logs;

use common::{
    create_server_table, map_lua_err, set_server_function, set_server_table, ServerContext,
};

impl PluginRuntime {
    pub(super) fn setup_server_namespace(&self, sl: &Table) -> Result<(), String> {
        let server_table = create_server_table(&self.lua)?;
        let ctx = ServerContext::new(self.permissions.clone());

        set_server_function(
            &server_table,
            "list",
            files::list(&self.lua, &ctx)?,
            "server.set_list_failed",
        )?;
        set_server_function(
            &server_table,
            "get_path",
            files::get_path(&self.lua, &ctx)?,
            "server.set_get_path_failed",
        )?;
        set_server_function(
            &server_table,
            "read_file",
            files::read_file(&self.lua, &ctx)?,
            "server.set_read_file_failed",
        )?;
        set_server_function(
            &server_table,
            "write_file",
            files::write_file(&self.lua, &ctx)?,
            "server.set_write_file_failed",
        )?;
        set_server_function(
            &server_table,
            "list_dir",
            files::list_dir(&self.lua, &ctx)?,
            "server.set_list_dir_failed",
        )?;
        set_server_function(
            &server_table,
            "exists",
            files::exists(&self.lua, &ctx)?,
            "server.set_exists_failed",
        )?;

        logs::register(&self.lua, &server_table, &ctx)?;
        set_server_table(sl, server_table)
            .map_err(|e| map_lua_err("server.set_server_failed", mlua::Error::runtime(e)))
    }
}
