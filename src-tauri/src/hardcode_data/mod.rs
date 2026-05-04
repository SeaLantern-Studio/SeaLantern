//! 这里专门放后端暂时不会频繁变化的固定数据。
//!
//! 这类内容主要是链接、仓库地址、允许域名、固定文件名之类的值。
//! 后面继续清理硬编码时，也优先往这里归类。

pub mod app_files;
pub mod dev_samples;
pub mod external_services;
pub mod plugin_manifest;
pub mod plugin_market;
pub mod plugin_permissions;
pub mod server_downloads;
pub mod update_sources;
