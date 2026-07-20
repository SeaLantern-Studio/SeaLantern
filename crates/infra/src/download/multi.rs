//! Multi-threaded file downloader.
//!
//! Provides the `Downloader` struct, wrapping a configured `NetClient`,
//! calling `download()` automatically probes the remote and downloads the file.

use std::sync::Arc;

use crate::download::status::{DownloadError, DownloadStatus};
use crate::download::tasks::{spawn_download_tasks, spawn_task_monitor};
use crate::net::client::NetClient;
use crate::observability;

/// Multi-threaded downloader.
///
/// Holds a configured HTTP client; call `download()` to download a file.
///
/// # Examples
///
/// ```ignore
/// let client = NetClient::from_config(&ClientConfig::default())?;
/// let downloader = Downloader::new(client);
/// let status = downloader.download(url, path, 8).await?;
/// ```
pub struct Downloader {
    client: NetClient,
}

impl Downloader {
    /// Creates a downloader.
    ///
    /// # Parameters
    ///
    /// - `client`: HTTP client with proxy configuration loaded
    pub fn new(client: NetClient) -> Self {
        Self { client }
    }

    /// Downloads a file and returns a status handle that can be queried for progress.
    ///
    /// Procedure:
    /// 1. Probe remote file info (whether Range is supported, file size)
    /// 2. Create and pre-allocate the local file
    /// 3. Choose segmented or single-threaded download based on Range support
    /// 4. Start background monitor task to aggregate segment results
    ///
    /// # Parameters
    ///
    /// - `url`: Download URL
    /// - `output_path`: Local save path
    /// - `thread_count`: Number of download threads
    ///
    /// # Returns
    ///
    /// Returns `Arc<DownloadStatus>`, which can be queried for real-time progress via `snapshot()`.
    pub async fn download(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<Arc<DownloadStatus>, DownloadError> {
        if thread_count == 0 {
            return Err(DownloadError::Message("Thread count must be positive".to_string()));
        }

        let remote = self.client.probe(url).await?;

        if remote.total_size == 0 {
            if let Some(parent) = std::path::Path::new(output_path).parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::File::create(output_path).await?;
            return Ok(Arc::new(DownloadStatus::new(0)));
        }

        let actual_thread_count = if remote.supports_range {
            thread_count
        } else {
            1
        };

        if let Some(parent) = std::path::Path::new(output_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file = tokio::fs::File::create(output_path).await?;

        file.set_len(remote.total_size).await?;

        drop(file);

        let status = Arc::new(DownloadStatus::new(remote.total_size));

        observability::download_started(url, remote.total_size, actual_thread_count);

        let tasks = spawn_download_tasks(
            self.client.clone(),
            url.to_string(),
            output_path.to_string(),
            actual_thread_count,
            remote.total_size,
            &status,
        );

        spawn_task_monitor(tasks, &status, url.to_string(), remote.total_size);

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn downloader_creation() {
        let client = NetClient::from_config(&Default::default()).unwrap();
        let downloader = Downloader::new(client);
        let result = downloader
            .download("https://example.com/test", "/tmp/test", 0)
            .await;
        assert!(result.is_err());
    }
}
