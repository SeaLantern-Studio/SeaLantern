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

        let debug_fn = if has_log_permission {
            emit::create_log_function(&ctx, "debug")
                .map_err(|e| map_log_err("log.create_debug_failed", e))?
        } else {
            emit::create_noop_log_function(&ctx)
                .map_err(|e| map_log_err("log.create_debug_noop_failed", e))?
        };
        set_log_function(&log, "debug", debug_fn)
            .map_err(|e| map_log_err("log.set_debug_failed", e))?;

        let info_fn = emit::create_log_function(&ctx, "info")
            .map_err(|e| map_log_err("log.create_info_failed", e))?;
        set_log_function(&log, "info", info_fn)
            .map_err(|e| map_log_err("log.set_info_failed", e))?;

        let warn_fn = emit::create_log_function(&ctx, "warn")
            .map_err(|e| map_log_err("log.create_warn_failed", e))?;
        set_log_function(&log, "warn", warn_fn)
            .map_err(|e| map_log_err("log.set_warn_failed", e))?;

        let error_fn = emit::create_log_function(&ctx, "error")
            .map_err(|e| map_log_err("log.create_error_failed", e))?;
        set_log_function(&log, "error", error_fn)
            .map_err(|e| map_log_err("log.set_error_failed", e))?;

        set_log_table(sl, log).map_err(|e| map_log_err("log.set_log_failed", e))
    }
}
