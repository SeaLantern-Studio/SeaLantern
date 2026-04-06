use mlua::Lua;

use super::common::{with_valid_server, ConsoleContext, DEFAULT_LOG_COUNT, MAX_LOG_COUNT};

pub(super) fn get_logs(lua: &Lua, _ctx: &ConsoleContext) -> Result<mlua::Function, String> {
    lua.create_function(move |lua, (server_id, count): (String, Option<usize>)| {
        with_valid_server(&server_id, || {
            let count = count.unwrap_or(DEFAULT_LOG_COUNT).min(MAX_LOG_COUNT);
            let logs = crate::services::server_log_pipeline::get_logs(&server_id, 0, Some(count));

            let result = lua.create_table()?;
            for (i, log) in logs.iter().enumerate() {
                let entry = lua.create_table()?;
                entry.set("content", log.clone())?;
                result.set(i + 1, entry)?;
            }

            Ok(result)
        })
    })
    .map_err(|e| super::common::map_console_err("console.create_get_logs_failed", e))
}
