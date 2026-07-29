use super::PluginRuntime;
use mlua::Table;

mod common;
mod info;
mod metrics;

use common::{create_system_table, set_system_function, set_system_table, SystemContext};

impl PluginRuntime {
    pub(super) fn setup_system_namespace(&self, sl: &Table) -> Result<(), String> {
        let system_table = create_system_table(&self.lua)?;
        let ctx = SystemContext::new(self.plugin_id.clone());

        set_system_function(
            &system_table,
            "get_os",
            info::get_os(&self.lua, &ctx)?,
            "system.set_get_os_failed",
        )?;
        set_system_function(
            &system_table,
            "get_arch",
            info::get_arch(&self.lua, &ctx)?,
            "system.set_get_arch_failed",
        )?;
        set_system_function(
            &system_table,
            "get_app_version",
            info::get_app_version(&self.lua, &ctx)?,
            "system.set_get_app_version_failed",
        )?;
        set_system_function(
            &system_table,
            "get_memory",
            metrics::get_memory(&self.lua, &ctx)?,
            "system.set_get_memory_failed",
        )?;
        set_system_function(
            &system_table,
            "get_cpu",
            metrics::get_cpu(&self.lua, &ctx)?,
            "system.set_get_cpu_failed",
        )?;

        set_system_table(sl, system_table)
    }
}
