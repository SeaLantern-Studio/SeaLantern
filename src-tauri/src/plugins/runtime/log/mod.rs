use super::PluginRuntime;
use mlua::Table;

mod common;
mod emit;

use common::{create_log_table, map_log_err, set_log_function, set_log_table, LogContext};

impl PluginRuntime {
    pub(super) fn setup_log_namespace(
        &self,
        sl: &Table,
        has_log_permission: bool,
    ) -> Result<(), String> {
        let log =
            create_log_table(&self.lua).map_err(|e| map_log_err("log.create_table_failed", e))?;
        let ctx = LogContext::new(self.plugin_id.clone(), self.lua.clone());

        for (name, enabled, create_key, set_key) in [
            ("debug", has_log_permission, "log.create_debug_failed", "log.set_debug_failed"),
            ("info", true, "log.create_info_failed", "log.set_info_failed"),
            ("warn", true, "log.create_warn_failed", "log.set_warn_failed"),
            ("error", true, "log.create_error_failed", "log.set_error_failed"),
        ] {
            let function = emit::create_log_function(&ctx, name, enabled)
                .map_err(|e| map_log_err(create_key, e))?;
            set_log_function(&log, name, function).map_err(|e| map_log_err(set_key, e))?;
        }

        set_log_table(sl, log).map_err(|e| map_log_err("log.set_log_failed", e))
    }
}
