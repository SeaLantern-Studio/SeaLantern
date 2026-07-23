//! 服务器实例管理方法。

mod console;
mod console_command;

pub use console::dispatch_console_command;
pub use console_command::{ConsoleCommandRequest, SendConsoleCommand};
