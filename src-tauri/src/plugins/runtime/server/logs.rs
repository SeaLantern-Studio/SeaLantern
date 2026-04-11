use super::common::{
    check_server_permission, create_logs_table, find_server, map_lua_err, running_log_pairs,
    set_logs_function, set_logs_table, ServerContext,
};
use mlua::{Lua, Table};

pub(super) fn register(lua: &Lua, server_table: &Table, ctx: &ServerContext) -> Result<(), String> {
    let logs_table = create_logs_table(lua)?;

    let get_logs_fn = get(lua, ctx)?;
    set_logs_function(&logs_table, "get", get_logs_fn, "server.set_logs_get_failed")?;

    let get_all_logs_fn = get_all(lua, ctx)?;
    set_logs_function(&logs_table, "getAll", get_all_logs_fn, "server.set_logs_getall_failed")?;

    set_logs_table(server_table, logs_table)
}

fn get(lua: &Lua, ctx: &ServerContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, (server_id, count): (String, Option<usize>)| {
        check_server_permission(&ctx.permissions)?;
        find_server(&server_id)?;

        let count = count.unwrap_or(100).min(1000);
        let logs = crate::services::server_log_pipeline::get_logs(&server_id, 0, Some(count));

        let result = lua.create_table()?;
        for (i, line) in logs.iter().enumerate() {
            result.set(i + 1, line.clone())?;
        }
        Ok(result)
    })
    .map_err(|e| map_lua_err("server.create_logs_get_failed", e))
}

fn get_all(lua: &Lua, ctx: &ServerContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, count: Option<usize>| {
        check_server_permission(&ctx.permissions)?;

        let count = count.unwrap_or(100).min(1000);
        let logs_pairs = running_log_pairs(count);

        let result = lua.create_table()?;
        let mut i = 1;
        for (server_id, logs) in logs_pairs {
            let entry = lua.create_table()?;
            entry.set("server_id", server_id)?;

            let lines_table = lua.create_table()?;
            for (j, line) in logs.iter().enumerate() {
                lines_table.set(j + 1, line.clone())?;
            }
            entry.set("logs", lines_table)?;

            result.set(i, entry)?;
            i += 1;
        }

        Ok(result)
    })
    .map_err(|e| map_lua_err("server.create_logs_getall_failed", e))
}
