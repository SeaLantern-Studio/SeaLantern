//! 供 RPC 方法调用的宿主能力端口。

use std::fmt::Display;

mod runtime;

pub use runtime::ServerRuntime;

/// 由宿主实现的运行中实例控制台写入能力。
///
/// 此 trait 保持应用服务与 Tauri、HTTP 或插件运行时的具体实现解耦。
pub trait ConsoleCommandExecutor {
    type Error: Display;

    /// 将命令写入指定实例的控制台。
    fn send_console_command(&self, instance_id: &str, command: &str) -> Result<(), Self::Error>;
}
