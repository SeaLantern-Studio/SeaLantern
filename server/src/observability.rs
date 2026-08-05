//! 服务器应用服务的可观测性。
//!
//! 提供 tracing 目标与事件常量，以及日志初始化入口。

use std::sync::Once;

use tracing_subscriber::EnvFilter;

/// 服务器应用服务的 tracing 目标。
pub const SERVER_TARGET: &str = "sealantern.server";

/// Event: 已处理一条服务器控制台命令。
pub const EVENT_CONSOLE_COMMAND_DISPATCHED: &str = "console_command_dispatched";
/// Event: RPC 方法调用失败。
pub const EVENT_RPC_METHOD_FAILED: &str = "rpc_method_failed";
/// Event: HTTP 适配器在调度前拒绝请求。
pub const EVENT_RPC_HTTP_REQUEST_REJECTED: &str = "rpc_http_request_rejected";

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

/// 记录 HTTP 适配器拒绝的请求，不记录请求头、请求正文或认证材料。
pub fn rpc_http_request_rejected(request_id: &str, reason: &'static str, error_code: &str) {
    tracing::warn!(
        target: SERVER_TARGET,
        event_name = EVENT_RPC_HTTP_REQUEST_REJECTED,
        request_id,
        reason,
        error_code,
        "rpc HTTP request rejected"
    );
}

static INIT_TRACING: Once = Once::new();

/// 初始化 tracing 日志。
///
/// 通过 `RUST_LOG` 环境变量控制日志等级与目标过滤
/// （未设置时默认 `sealantern.server=info`，即仅输出本 crate 的日志），
/// 以人类可读格式输出到标准输出。由 `Once` 保证幂等，多次调用安全。
pub fn init() {
    INIT_TRACING.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("sealantern.server=info"));
        tracing_subscriber::fmt().with_env_filter(filter).init();
    });
}
