//! 面向上层请求的服务器操作方法。

pub(crate) mod permissions;
pub mod plugin;
pub mod server;

pub use permissions::{PERMISSION_PLUGIN_V2_INVOKE, PERMISSION_SERVER_CONSOLE_SEND};
