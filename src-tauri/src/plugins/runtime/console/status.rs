use mlua::Lua;

use super::common::{get_server_status_checked, map_console_err, ConsoleContext};

pub(super) fn get_status(lua: &Lua, _ctx: &ConsoleContext) -> Result<mlua::Function, String> {
    lua.create_function(move |_, server_id: String| {
        let status = get_server_status_checked(&server_id)?;
        Ok(status.status.as_str().to_string())
    })
    .map_err(|e| map_console_err("console.create_get_status_failed", e))
}
