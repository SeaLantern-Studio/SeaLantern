//! 文件下载模块。
//!
//! 提供多线程分段下载（`Downloader`）和单线程流式下载（`single` 模块）。
//! 通过 `Downloader::download()` 自动选择分段策略，
//! `single::stream_download()` 用于小文件或流式场景。

pub(crate) mod chunk;
pub mod manager;
pub mod multi;
pub(crate) mod single;
pub mod status;
pub(crate) mod tasks;

pub use manager::DownloadManager;
pub use multi::Downloader;
pub use single::{fetch_to_bytes, fetch_to_string, stream_download};
pub use status::{DownloadError, DownloadSnapshot, DownloadStatus};
