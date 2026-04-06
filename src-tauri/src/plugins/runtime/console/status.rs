use crate::services::global::server_manager;
use mlua::Lua;

use super::common::{map_console_err, with_valid_server, ConsoleContext};

pub(super) fn get_status(lua: &Lua, _ctx: &ConsoleContext) -> Result<mlua::Function, String> {
    lua.create_function(move |_, server_id: String| {
        with_valid_server(&server_id, || {
            let status = server_manager().get_server_status(&server_id);
            Ok(status.status.as_str().to_string())
        })
    })
    .map_err(|e| map_console_err("console.create_get_status_failed", e))
}
