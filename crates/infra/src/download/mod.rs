//! File download module.
//!
//! Provides multi-threaded segmented download (`Downloader`) and single-threaded streaming download (`single` module).
//! Automatically selects segmentation strategy via `Downloader::download()`,
//! `single::stream_download()` is used for small files or streaming scenarios.

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
