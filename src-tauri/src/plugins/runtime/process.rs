use super::PluginRuntime;
use mlua::Table;

mod commands;
mod common;
mod lifecycle;

use common::ProcessRegistry as InternalProcessRegistry;
#[cfg(test)]
pub(crate) use common::{collect_finished_processes, ProcessEntry};
pub use common::{
    kill_all_processes, kill_plugin_processes, new_process_registry, ProcessRegistry,
};

impl PluginRuntime {
    /// 安装 `sl.process` 命名空间
    ///
    /// # Parameters
    ///
    /// - `sl`: Lua 里的 `sl` 根表
    /// - `process_registry`: 当前插件系统共用的进程注册表
    ///
    /// # Returns
    ///
    /// 安装成功时返回 `Ok(())`
    pub(super) fn setup_process_namespace(
        &self,
        sl: &Table,
        process_registry: InternalProcessRegistry,
    ) -> Result<(), String> {
        let process_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create process table: {}", e))?;

        commands::register(
            &self.lua,
            &process_table,
            &self.plugin_dir,
            &self.plugin_id,
            &self.permissions,
            &process_registry,
        )?;
        lifecycle::register(&self.lua, &process_table, &self.plugin_id, &process_registry)?;

        sl.set("process", process_table)
            .map_err(|e| format!("Failed to set sl.process: {}", e))?;

        Ok(())
    }
}
