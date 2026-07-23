/// 服务器应用服务的 tracing 目标。
pub const SERVER_TARGET: &str = "sealantern.server";

/// Event: 已处理一条服务器控制台命令。
pub const EVENT_CONSOLE_COMMAND_DISPATCHED: &str = "console_command_dispatched";
/// Event: RPC 方法调用失败。
pub const EVENT_RPC_METHOD_FAILED: &str = "rpc_method_failed";

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

/// 记录 RPC 方法失败，不记录参数、响应正文或错误消息。
pub fn rpc_method_failed(
    method: &str,
    context: &crate::rpc::RpcContext,
    error: &crate::rpc::RpcError,
) {
    tracing::warn!(
        target: SERVER_TARGET,
        event_name = EVENT_RPC_METHOD_FAILED,
        method,
        request_id = context.request_id().as_str(),
        transport = context.transport().as_str(),
        error_code = error.code().as_str(),
        retryable = error.is_retryable(),
        "rpc method failed"
    );
}
