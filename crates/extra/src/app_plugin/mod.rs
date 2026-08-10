//! 应用插件执行内核。
//!
//! 本模块只负责 API v2 插件的清单校验、Lua 脚本执行、生命周期与私有存储。
//! 它不依赖 Tauri、前端事件或全局宿主服务。
//!
//! 除测试专用的序列化存储构造器外，生产能力统一经宿主 dispatcher 调用。私有存储在
//! SQLite dispatcher adapter 完成前不会暴露直连文件 API，避免绕过会话授权和审计。

mod engine;
pub mod error;
pub mod loader;
mod manager;
pub mod manifest;

pub use error::AppPluginError;
pub use loader::PluginLoader;
pub use manager::{
    AsyncPluginManager, PluginInfo, PluginManager, PluginManagerConfig, PluginState,
};
pub use manifest::{PluginCapability, PluginManifest, PLUGIN_API_VERSION};
