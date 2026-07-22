//! 资源获取器（Fetcher）模块。
//!
//! 提供 [`Fetcher`] trait 定义以及各平台的实现（Spigot、Modrinth）。
//! 还包含通用的 `download_file` 辅助函数，供各平台实现复用。

pub mod error;
pub mod models;
mod modrinth;
mod spiget;
pub mod traits;

use std::sync::Arc;

pub use error::FetcherError;
pub use models::VersionFile;
pub use traits::Fetcher;
pub use modrinth::ModrinthFetcher;
pub use spiget::SpigetFetcher;

/// 市场 API 请求使用的 User-Agent。
const USER_AGENT: &str = "SeaLantern/extra/0.1.0";

/// 通用的文件下载函数，使用全局 `DownloadManager` 执行多线程下载。
///
/// # Parameters
/// - `url` — 文件的下载链接
/// - `destination` — 本地保存路径（包含文件名）
///
/// # Returns
/// 返回 [`DownloadStatus`] 的共享引用，可用于跟踪下载进度与结果。
///
/// # Errors
/// 如果下载初始化失败，返回 [`FetcherError::Download`]。
pub(crate) async fn download_file(
    url: &str,
    destination: &str,
) -> Result<Arc<sealantern_infra::download::DownloadStatus>, FetcherError> {
    let (_id, status) = sealantern_infra::download::DownloadManager::instance()
        .create_with_handle(url, destination, 8)
        .await
        .map_err(|e| FetcherError::Download(e.to_string()))?;
    Ok(status)
}
