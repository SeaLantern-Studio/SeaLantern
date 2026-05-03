mod chunk;
mod probe;
mod tasks;

use super::DownloadStatus;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// 多线程下载器
pub struct MultiThreadDownloader {
    client: Client,
}

impl MultiThreadDownloader {
    /// 创建多线程下载器
    ///
    /// # Parameters
    ///
    /// - `user_agent`: 请求使用的浏览器标识
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(Duration::from_secs(15))
                .read_timeout(Duration::from_secs(30))
                .user_agent(user_agent)
                .build()
                .unwrap(),
        }
    }

    /// 下载文件并返回可查询的状态句柄
    ///
    /// # Parameters
    ///
    /// - `url`: 下载地址
    /// - `output_path`: 保存路径
    /// - `thread_count`: 下载线程数
    pub async fn download(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<Arc<DownloadStatus>, String> {
        if thread_count == 0 {
            return Err("Thread count must be positive".to_string());
        }

        let remote = probe::probe_remote_file(&self.client, url).await?;
        let actual_thread_count = if remote.supports_range {
            thread_count
        } else {
            1
        };

        let file = tokio::fs::File::create(output_path)
            .await
            .map_err(|e| e.to_string())?;
        file.set_len(remote.total_size)
            .await
            .map_err(|e| e.to_string())?;

        let status = Arc::new(DownloadStatus::new(remote.total_size));
        let client = Arc::new(self.client.clone());
        let tasks = tasks::spawn_download_tasks(
            client,
            url,
            output_path,
            actual_thread_count,
            remote.total_size,
            &status,
        );

        tasks::spawn_task_monitor(tasks, &status);

        Ok(status)
    }
}
