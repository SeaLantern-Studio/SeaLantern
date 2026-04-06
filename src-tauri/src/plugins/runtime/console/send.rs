use crate::services::global::{i18n_service, server_manager};
use mlua::Lua;

use super::common::{
    emit_console_log, is_command_allowed, map_console_err, with_valid_server, ConsoleContext,
};

pub(super) fn send(lua: &Lua, ctx: &ConsoleContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (server_id, command): (String, String)| {
        with_valid_server(&server_id, || {
            let status = server_manager().get_server_status(&server_id);
            if status.status != crate::models::server::ServerStatus::Running {
                return Err(mlua::Error::runtime(i18n_service().t("console.server_not_running")));
            }

            let sanitized_cmd = is_command_allowed(&command).map_err(mlua::Error::runtime)?;

            match server_manager().send_command(&server_id, &sanitized_cmd) {
                Ok(_) => {
                    emit_console_log(
                        &ctx.plugin_id,
                        "command",
                        "sl.console.send",
                        &format!("[{}] {}", server_id, sanitized_cmd),
                    );
                    Ok(true)
                }
                Err(e) => Err(mlua::Error::runtime(i18n_service().t_with_options(
                    "console.send_command_failed",
                    &super::common::i18n_arg("0", &e.to_string()),
                ))),
            }
        })
    })
    .map_err(|e| map_console_err("console.create_send_failed", e))
}
