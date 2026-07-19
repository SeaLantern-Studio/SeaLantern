//! 下载任务管理器。
//!
//! 管理多个下载任务的创建、进度查询、取消和自动清理。
//! 内部使用 `HashMap` 存储所有任务，已结束的任务在查询时自动移除。

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::download::multi::Downloader;
use crate::download::status::{DownloadSnapshot, DownloadStatus};
use crate::net::client::NetClient;

/// 全局自增任务 ID 计数器。从 1 起始，0 作为无效哨兵值。
static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);

/// 下载任务管理器。
///
/// 包装 `Downloader`，提供多任务生命周期管理。
/// 查询进度时会自动清理已结束的任务。
///
/// # Examples
///
/// ```ignore
/// let client = NetClient::from_config(&ClientConfig::default())?;
/// let manager = DownloadManager::new(client);
/// let id = manager.create("https://...", "./file.zip", 8).await;
/// let snap = manager.get_progress(id).await;
/// manager.cancel(id).await;
/// ```
pub struct DownloadManager {
    /// 下载器实例
    downloader: Downloader,
    /// 任务集合：ID → 下载状态
    tasks: Arc<RwLock<HashMap<usize, Arc<DownloadStatus>>>>,
}

impl DownloadManager {
    /// 创建下载管理器。
    ///
    /// # Parameters
    ///
    /// - `client`: 已配置的 HTTP 客户端
    pub fn new(client: NetClient) -> Self {
        Self {
            downloader: Downloader::new(client),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建下载任务。
    ///
    /// 启动下载后立即返回任务 ID，下载在后台异步进行。
    ///
    /// # Parameters
    ///
    /// - `url`: 下载地址
    /// - `output_path`: 本地保存路径
    /// - `thread_count`: 下载线程数
    ///
    /// # Returns
    ///
    /// 返回任务 ID（`usize`），后续通过此 ID 查询进度或取消。
    pub async fn create(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<usize, String> {
        let status = self
            .downloader
            .download(url, output_path, thread_count)
            .await?;
        let id = NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut tasks = self.tasks.write().await;
        tasks.insert(id, status);

        tracing::info!(
            target: crate::observability::DOWNLOAD_TARGET,
            event_name = "task_created",
            task_id = id,
            url,
            "download task created"
        );

        Ok(id)
    }

    /// 查询单个任务进度。
    ///
    /// 任务已结束时自动从管理器中移除。
    ///
    /// # Parameters
    ///
    /// - `id`: 任务 ID
    ///
    /// # Returns
    ///
    /// 任务存在时返回 `Some(DownloadSnapshot)`，不存在返回 `None`。
    pub async fn get_progress(&self, id: usize) -> Option<DownloadSnapshot> {
        let status = {
            let tasks = self.tasks.read().await;
            tasks.get(&id).cloned()?
        };

        let snap = status.snapshot().await;

        // 任务结束后自动移除，释放管理资源。
        // 此处存在可接受的竞态：两个并发调用可能同时看到任务 finished，
        // 同时尝试 remove。第二次 remove 是幂等操作，不影响正确性。
        if snap.is_finished {
            let mut tasks = self.tasks.write().await;
            tasks.remove(&id);
        }

        Some(snap)
    }

    /// 查询全部任务进度。
    ///
    /// 返回所有正在进行的任务进度，已结束的任务会被自动清理。
    pub async fn get_all_progress(&self) -> Vec<(usize, DownloadSnapshot)> {
        let snapshot: Vec<(usize, Arc<DownloadStatus>)> = {
            let tasks = self.tasks.read().await;
            tasks.iter().map(|(id, s)| (*id, s.clone())).collect()
        };

        let mut results = Vec::new();
        let mut to_remove = Vec::new();

        for (id, status) in snapshot {
            let snap = status.snapshot().await;
            if snap.is_finished {
                to_remove.push(id);
            }
            results.push((id, snap));
        }

        if !to_remove.is_empty() {
            let mut tasks = self.tasks.write().await;
            for id in to_remove {
                tasks.remove(&id);
            }
        }

        results
    }

    /// 取消下载任务。
    ///
    /// 取消后任务会从管理器中移除。
    ///
    /// # Parameters
    ///
    /// - `id`: 任务 ID
    pub async fn cancel(&self, id: usize) {
        let status = {
            let tasks = self.tasks.read().await;
            tasks.get(&id).cloned()
        };

        if let Some(status) = status {
            status.cancel();
            let mut tasks = self.tasks.write().await;
            tasks.remove(&id);

            tracing::warn!(
                target: crate::observability::DOWNLOAD_TARGET,
                event_name = "task_cancelled",
                task_id = id,
                "download task cancelled"
            );
        }
    }

    /// 获取当前管理的任务数量。
    pub async fn task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_manager_is_empty() {
        let client = NetClient::from_config(&Default::default()).unwrap();
        let manager = DownloadManager::new(client);
        assert_eq!(manager.task_count().await, 0);
    }

    #[tokio::test]
    async fn cancel_nonexistent_task_does_nothing() {
        let client = NetClient::from_config(&Default::default()).unwrap();
        let manager = DownloadManager::new(client);
        manager.cancel(999).await;
        assert_eq!(manager.task_count().await, 0);
    }

    #[tokio::test]
    async fn get_progress_nonexistent_returns_none() {
        let client = NetClient::from_config(&Default::default()).unwrap();
        let manager = DownloadManager::new(client);
        let snap = manager.get_progress(999).await;
        assert!(snap.is_none());
    }
}
