use crate::{models::server::ServerStatus, services::global::server_manager};
use mlua::Lua;

use super::common::{
    emit_console_log, get_server_status_checked, is_command_allowed, map_console_err,
    runtime_console_err, runtime_console_msg, ConsoleContext,
};

fn send_audit_detail(server_id: &str, command: &str) -> String {
    format!("[{}] {}", server_id, command)
}

pub(super) fn send(lua: &Lua, ctx: &ConsoleContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (server_id, command): (String, String)| {
        let audit_detail = send_audit_detail(&server_id, &command);
        let status = get_server_status_checked(&server_id)?;
        if status.status != ServerStatus::Running {
            emit_console_log(&ctx.plugin_id, "command_denied", "sl.console.send", &audit_detail);
            return Err(runtime_console_msg("console.server_not_running"));
        }

        let sanitized_cmd = match is_command_allowed(&command) {
            Ok(sanitized) => sanitized,
            Err(err) => {
                emit_console_log(
                    &ctx.plugin_id,
                    "command_denied",
                    "sl.console.send",
                    &audit_detail,
                );
                return Err(mlua::Error::runtime(err));
            }
        };

        let sanitized_audit_detail = send_audit_detail(&server_id, &sanitized_cmd);
        match server_manager().send_command(&server_id, &sanitized_cmd) {
            Ok(_) => {
                emit_console_log(
                    &ctx.plugin_id,
                    "command",
                    "sl.console.send",
                    &sanitized_audit_detail,
                );
                Ok(true)
            }
            Err(err) => {
                emit_console_log(
                    &ctx.plugin_id,
                    "command_failed",
                    "sl.console.send",
                    &sanitized_audit_detail,
                );
                Err(runtime_console_err("console.send_command_failed", err))
            }
        }
    })
    .map_err(|e| map_console_err("console.create_send_failed", e))
}
