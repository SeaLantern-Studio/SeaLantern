//! 应用插件执行内核。
//!
//! 本模块只负责 API v2 插件的清单校验、Lua 脚本执行、生命周期与私有存储。
//! 它不依赖 Tauri、前端事件或全局宿主服务。
//!
//! 当前仅提供 \`log\` 与 \`storage\` 权限。文件系统、网络、进程、服务器、控制台、
//! 国际化、UI、元素和 API bridge 将在后续任务中通过显式宿主 trait 接入；在此之前，
//! 清单请求这些能力会被拒绝，而不是静默降级。

mod engine;
pub mod error;
pub mod loader;
mod manager;
pub mod manifest;

pub use error::AppPluginError;
pub use loader::PluginLoader;
pub use manager::{PluginInfo, PluginManager, PluginManagerConfig, PluginState};
pub use manifest::{PluginManifest, PluginPermission, PLUGIN_API_VERSION};
