//! 多线程文件下载器。
//!
//! 提供 `Downloader` 结构体，包装已配置的 `NetClient`，
//! 调用 `download()` 即可自动探测远端并下载文件。

use std::sync::Arc;

use crate::download::status::{DownloadError, DownloadStatus};
use crate::download::tasks::{spawn_download_tasks, spawn_task_monitor};
use crate::net::client::NetClient;
use crate::observability;

/// 多线程下载器。
///
/// 持有已配置的 HTTP 客户端，调用 `download()` 即可下载文件。
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
    /// 创建下载器。
    ///
    /// # Parameters
    ///
    /// - `client`: 已加载代理配置的 HTTP 客户端
    pub fn new(client: NetClient) -> Self {
        Self { client }
    }

    /// 下载文件并返回可查询进度的状态句柄。
    ///
    /// 流程：
    /// 1. 探测远端文件信息（是否支持 Range、文件大小）
    /// 2. 创建并预分配本地文件
    /// 3. 根据是否支持 Range 选择分片或单线程下载
    /// 4. 启动后台监控任务汇总分片结果
    ///
    /// # Parameters
    ///
    /// - `url`: 下载地址
    /// - `output_path`: 本地保存路径
    /// - `thread_count`: 下载线程数
    ///
    /// # Returns
    ///
    /// 返回 `Arc<DownloadStatus>`，可通过 `snapshot()` 查询实时进度。
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
