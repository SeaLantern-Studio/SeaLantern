use super::process::common::process_msg1;
use super::PluginRuntime;
use mlua::Table;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod commands;
mod common;
mod lifecycle;

pub(crate) use common::ProcessEntry;
#[cfg(test)]
pub(crate) use common::{
    collect_finished_processes, new_process_output, update_output_timestamp, ProcessOutputBuffers,
};
pub use common::{kill_all_processes, kill_plugin_processes, new_process_registry};

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
        process_registry: Arc<Mutex<HashMap<u32, common::ProcessEntry>>>,
    ) -> Result<(), String> {
        let process_table = self.lua.create_table().map_err(|e| {
            process_msg1("plugins.runtime.process.create_table_failed", e.to_string())
        })?;

        commands::register(
            &self.lua,
            &process_table,
            &self.plugin_dir,
            &self.plugin_id,
            &self.permissions,
            &self.allowed_programs,
            &process_registry,
        )?;
        lifecycle::register(
            &self.lua,
            &process_table,
            &self.plugin_id,
            &self.permissions,
            &process_registry,
        )?;

        sl.set("process", process_table).map_err(|e| {
            process_msg1("plugins.runtime.process.set_namespace_failed", e.to_string())
        })?;

        Ok(())
    }
}
