use super::common::{convert_lua_string, LogContext};
use mlua::Function;

use crate::utils::logger::{log_debug_ctx, log_error_ctx};

pub(super) fn create_log_function(
    ctx: &LogContext,
    level: &str,
    enabled: bool,
) -> Result<Function, mlua::Error> {
    use crate::plugins::api::emit_log_event;

    let plugin_id = ctx.plugin_id.clone();
    let level = level.to_string();
    let level_display = level.to_uppercase();

    ctx.lua.create_function(move |_, msg: mlua::String| {
        if !enabled {
            return Ok(());
        }

        let message = convert_lua_string(&msg);
        log_debug_ctx(
            "plugins.runtime.log.emit",
            "create_log_function",
            &format!("plugin_id={} level={} message={}", plugin_id, level_display, message),
        );
        if let Err(error) = emit_log_event(&plugin_id, &level, &message) {
            log_error_ctx(
                "plugins.runtime.log.emit",
                "create_log_function",
                &format!(
                    "plugin log emit failed: plugin_id={} level={} error={}",
                    plugin_id, level, error
                ),
            );
        }
        Ok(())
    })
}
