use super::common::{convert_lua_string, LogContext};
use mlua::Function;

pub(super) fn create_log_function(ctx: &LogContext, level: &str) -> Result<Function, mlua::Error> {
    use crate::plugins::api::emit_log_event;

    let plugin_id = ctx.plugin_id.clone();
    let level = level.to_string();

    ctx.lua.create_function(move |_, msg: mlua::String| {
        let message = convert_lua_string(&msg);
        println!("[{}] [{}] {}", level.to_uppercase(), plugin_id, message);
        let _ = emit_log_event(&plugin_id, &level, &message);
        Ok(())
    })
}

pub(super) fn create_noop_log_function(ctx: &LogContext) -> Result<Function, mlua::Error> {
    ctx.lua.create_function(move |_, _msg: mlua::String| Ok(()))
}
