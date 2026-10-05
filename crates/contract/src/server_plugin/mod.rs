//! 服务器插件（`plugins` 目录）管理契约模型。
//!
//! 定义跨宿主共享的插件摘要与配置文件 DTO；具体目录操作由实现层提供。

mod models;

pub use models::{PluginConfigFile, PluginSummary};
