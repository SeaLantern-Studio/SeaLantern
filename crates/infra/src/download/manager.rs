//! 下载任务管理器。
//!
//! 提供全局单例 `DownloadManager`，统一管理所有多线程下载任务。
//! 调用方通过 `DownloadManager::instance()` 获取管理器，
//! 使用 `create()` 或 `create_with_handle()` 启动下载，
//! 通过 `get_progress()` / `cancel()` 查询和取消任务。
//! 已结束的任务在查询时自动清理。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::download::multi::Downloader;
use crate::download::status::{DownloadError, DownloadSnapshot, DownloadStatus};
use crate::net::client::NetClient;
use crate::net::{global_client_provider, ClientProvider};
use crate::observability;

/// 下载任务管理器。
///
/// 封装多线程下载能力，提供全局单例。
/// 所有通过此管理器创建的下载任务均可通过 UUID 查询进度或取消。
///
/// # Examples
///
/// ```ignore
/// let manager = DownloadManager::instance();
/// let id = manager.create("https://...", "./file.zip", 8).await;
/// let snap = manager.get_progress(id).await;
/// manager.cancel(id).await;
/// ```
pub struct DownloadManager {
    /// 下载器实例
    downloader: Downloader,
    /// 任务映射：ID → 下载状态
    tasks: Arc<RwLock<HashMap<Uuid, Arc<DownloadStatus>>>>,
}

impl DownloadManager {
    /// 创建下载任务管理器（供上层显式持有与注入）。
    ///
    /// 也可通过 [`Self::instance`] 获取进程级全局单例；显式构造便于
    /// application 层按服务装配方式持有，而非被全局单例反向绑定。
    ///
    /// 从既有客户端构造：内部包装为固定 provider，便于测试注入。
    /// 生产代码应优先使用 [`Self::with_provider`] 获取全局客户端。
    pub fn new(client: NetClient) -> Self {
        Self::with_provider(Box::new(move || Ok(client.clone())))
    }

    /// 从客户端获取器构造下载任务管理器。
    ///
    /// 每次创建下载任务时都会调用 provider 获取当前全局客户端，
    /// 避免缓存固定客户端导致代理更新不生效。
    pub fn with_provider(client_provider: ClientProvider) -> Self {
        Self {
            downloader: Downloader::new(client_provider),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建一个下载任务。
    ///
    /// 启动下载后立即返回任务 UUID，下载在后台异步进行。
    ///
    /// # Parameters
    ///
    /// - `url`: 下载 URL
    /// - `output_path`: 本地保存路径
    /// - `thread_count`: 下载线程数
    ///
    /// # Returns
    ///
    /// 返回任务 UUID，后续可用于查询进度或取消。
    pub async fn create(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<Uuid, DownloadError> {
        let (id, _) = self
            .create_with_handle(url, output_path, thread_count)
            .await?;
        Ok(id)
    }

    /// 创建下载任务并返回任务 UUID 和状态句柄。
    ///
    /// 与 `create()` 的区别在于同时返回 `Arc<DownloadStatus>`，
    /// 调用方可直接轮询进度或监听取消信号，无需通过管理器查询。
    pub async fn create_with_handle(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<(Uuid, Arc<DownloadStatus>), DownloadError> {
        let status = self
            .downloader
            .download(url, output_path, thread_count)
            .await?;
        let id = Uuid::new_v4();

        let mut tasks = self.tasks.write().await;
        tasks.insert(id, status.clone());

        observability::task_created(&id, url);

        Ok((id, status))
    }

    /// 查询单个任务的进度。
    ///
    /// 任务完成时自动从管理器中移除。
    ///
    /// # Parameters
    ///
    /// - `id`: 任务 UUID
    ///
    /// # Returns
    ///
    /// 如果任务存在，返回 `Some(DownloadSnapshot)`，否则返回 `None`。
    pub async fn get_progress(&self, id: Uuid) -> Option<DownloadSnapshot> {
        let status = {
            let tasks = self.tasks.read().await;
            tasks.get(&id).cloned()?
        };

        let snap = status.snapshot().await;

        // 自动移除已完成的任务以释放管理资源。
        // 可接受的竞态条件：两个并发调用可能同时看到任务已完成，
        // 并且都尝试移除它。第二次移除是幂等的，不影响正确性。
        if snap.is_finished {
            let mut tasks = self.tasks.write().await;
            tasks.remove(&id);
        }

        Some(snap)
    }

    /// 查询所有任务的进度。
    ///
    /// 返回所有进行中任务的进度；已完成的任务会被自动清理。
    pub async fn get_all_progress(&self) -> Vec<(Uuid, DownloadSnapshot)> {
        let snapshot: Vec<(Uuid, Arc<DownloadStatus>)> = {
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

    /// 取消一个下载任务。
    ///
    /// 取消后任务将从管理器中移除。
    ///
    /// # Parameters
    ///
    /// - `id`: 任务 UUID
    pub async fn cancel(&self, id: Uuid) {
        let status = {
            let tasks = self.tasks.read().await;
            tasks.get(&id).cloned()
        };

        if let Some(status) = status {
            status.cancel();
            let mut tasks = self.tasks.write().await;
            tasks.remove(&id);

            observability::task_cancelled(&id);
        }
    }

    /// 返回当前管理的任务数量。
    pub async fn task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
}

static GLOBAL_DOWNLOAD_MANAGER: OnceLock<DownloadManager> = OnceLock::new();

impl DownloadManager {
    /// 获取全局下载管理器实例（懒加载）。
    ///
    /// 管理器内部持有全局客户端 provider：每次创建下载任务时都会
    /// 重新获取当前全局客户端，与代理设置同步。
    pub fn instance() -> &'static Self {
        GLOBAL_DOWNLOAD_MANAGER
            .get_or_init(|| DownloadManager::with_provider(global_client_provider()))
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
        manager.cancel(Uuid::nil()).await;
        assert_eq!(manager.task_count().await, 0);
    }

    #[tokio::test]
    async fn get_progress_nonexistent_returns_none() {
        let client = NetClient::from_config(&Default::default()).unwrap();
        let manager = DownloadManager::new(client);
        let snap = manager.get_progress(Uuid::nil()).await;
        assert!(snap.is_none());
    }
}
