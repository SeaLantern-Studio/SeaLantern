use mlua::Lua;

use super::common::{
    get_server_status_checked, map_console_err, ConsoleContext, DEFAULT_LOG_COUNT, MAX_LOG_COUNT,
};

pub(super) fn get_logs(lua: &Lua, _ctx: &ConsoleContext) -> Result<mlua::Function, String> {
    lua.create_function(
        move |lua, (server_id, offset, count): (String, Option<usize>, Option<usize>)| {
            let _ = get_server_status_checked(&server_id)?;
            let offset = offset.unwrap_or(0);
            let count = count.unwrap_or(DEFAULT_LOG_COUNT).min(MAX_LOG_COUNT);
            let logs =
                crate::services::server_log_pipeline::get_logs(&server_id, offset, Some(count));

            let result = lua.create_table()?;
            result.set("server_id", server_id.clone())?;
            result.set("offset", offset)?;
            result.set("count", count)?;
            result.set("next_offset", offset + logs.len())?;

            let entries = lua.create_table()?;
            for (i, log) in logs.iter().enumerate() {
                let entry = lua.create_table()?;
                entry.set("index", offset + i)?;
                entry.set("content", log.clone())?;
                entries.set(i + 1, entry)?;
            }
            result.set("logs", entries)?;

            Ok(result)
        },
    )
    .map_err(|e| map_console_err("console.create_get_logs_failed", e))
}
