use super::common::{convert_lua_string, LogContext};
use mlua::Function;

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
        println!("[{}] [{}] {}", level_display, plugin_id, message);
        let _ = emit_log_event(&plugin_id, &level, &message);
        Ok(())
    })
}
