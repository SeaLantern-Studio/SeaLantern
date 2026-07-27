//! 服务器核心下载链接管理
//!
//! 从远程配置加载服务器核心下载链接。

mod manager;
mod models;

pub use manager::LinkManager;
pub use models::{BaseDownloadLinks, DownloadLink, TypeDownloadLinks};
