//! 服务器核心下载链接管理
//!
//! 从远程配置加载服务器核心下载链接。

mod manager;

pub use crate::models::{BaseDownloadLinks, DownloadLink, TypeDownloadLinks};
pub use manager::{LinkError, LinkManager};
