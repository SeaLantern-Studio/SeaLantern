//! RPC 宿主能力实现层。
//!
//! 传输无关契约（trait）定义于 `super::traits`，本目录只承载实现：
//! [`ServerRuntime`]、[`RpcServices`] 容器与调度辅助函数。

mod console;
mod runtime;
pub(crate) mod services;

pub use console::dispatch_console_command;
pub use runtime::ServerRuntime;
pub use services::RpcServices;
