//! 前端与后端共享的数据模型

mod app;
mod java;
mod server;
mod task;

pub use app::AppSettings;
pub use java::JavaInfo;
pub use server::ServerInstance;
pub use task::{TaskProgressResponse, TaskStatus};

// 从 download_link 重新导出
pub use crate::download_link::{BaseDownloadLinks, DownloadLink, LinkManager, TypeDownloadLinks};
