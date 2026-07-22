/// 服务器应用服务的 tracing 目标。
pub const SERVER_TARGET: &str = "sealantern.server";

/// Event: 已处理一条服务器控制台命令。
pub const EVENT_CONSOLE_COMMAND_DISPATCHED: &str = "console_command_dispatched";

/// 记录控制台命令的处理结果，不记录命令正文或执行错误。
pub fn console_command_dispatched(instance_id: &str, command_char_count: usize, succeeded: bool) {
    if succeeded {
        tracing::info!(
            target: SERVER_TARGET,
            event_name = EVENT_CONSOLE_COMMAND_DISPATCHED,
            instance_id,
            command_char_count,
            outcome = "succeeded",
            "console command dispatched"
        );
    } else {
        tracing::warn!(
            target: SERVER_TARGET,
            event_name = EVENT_CONSOLE_COMMAND_DISPATCHED,
            instance_id,
            command_char_count,
            outcome = "failed",
            "console command dispatch failed"
        );
    }
}
