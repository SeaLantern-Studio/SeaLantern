//! 这里专门放后端暂时不会频繁变化的固定数据。
//!
//! 这类内容主要是链接、仓库地址、允许域名、固定文件名之类的值。
//! 后面继续清理硬编码时，也优先往这里归类。

pub(crate) mod app_files;
pub(crate) mod dev_samples;
pub(crate) mod external_services;
pub(crate) mod plugin_manifest;
pub(crate) mod plugin_market;
pub(crate) mod plugin_permissions;
pub(crate) mod server_downloads;
pub(crate) mod update_sources;
