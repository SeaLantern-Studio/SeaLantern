//! 文件下载模块。
//!
//! 提供多线程分段下载（通过 `DownloadManager`）和单线程流式下载。
//! 调用方通过 `DownloadManager::instance()` 获取全局下载管理器实例，
//! 使用 `create()` 或 `create_with_handle()` 启动下载任务。

pub(crate) mod chunk;
pub mod manager;
pub(crate) mod multi;
pub(crate) mod single;
pub mod status;
pub(crate) mod tasks;

pub use manager::DownloadManager;
pub use single::{fetch_to_bytes, fetch_to_string, stream_download};
pub use status::{DownloadError, DownloadSnapshot, DownloadStatus};
