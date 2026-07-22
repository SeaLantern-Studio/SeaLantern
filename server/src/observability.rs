use std::fmt::Display;

/// 服务器应用服务的 tracing 目标。
pub const SERVER_TARGET: &str = "sealantern.server";

/// Event: 已处理一条服务器控制台命令。
pub const EVENT_CONSOLE_COMMAND_DISPATCHED: &str = "console_command_dispatched";

/// 记录控制台命令的处理结果，不记录命令正文。
pub fn console_command_dispatched(
    instance_id: &str,
    command_length: usize,
    result: Result<(), &dyn Display>,
) {
    match result {
        Ok(()) => tracing::info!(
            target: SERVER_TARGET,
            event_name = EVENT_CONSOLE_COMMAND_DISPATCHED,
            instance_id,
            command_length,
            outcome = "succeeded",
            "console command dispatched"
        ),
        Err(error) => tracing::warn!(
            target: SERVER_TARGET,
            event_name = EVENT_CONSOLE_COMMAND_DISPATCHED,
            instance_id,
            command_length,
            outcome = "failed",
            error = %error,
            "console command dispatch failed"
        ),
    }
}
