//! Download task manager.
//!
//! Manages creation, progress querying, cancellation, and automatic cleanup of multiple download tasks.
//! Uses a `HashMap` internally to store all tasks; finished tasks are automatically removed on query.
//! Each task is identified by a UUID v4, so there is no risk of ID overflow or collision.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::download::multi::Downloader;
use crate::download::status::{DownloadError, DownloadSnapshot, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// Download task manager.
///
/// Wraps `Downloader` to provide multi-task lifecycle management.
/// Automatically cleans up finished tasks when querying progress.
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
    /// Downloader instance
    downloader: Downloader,
    /// Task map: ID → download status
    tasks: Arc<RwLock<HashMap<Uuid, Arc<DownloadStatus>>>>,
}

impl DownloadManager {
    /// Creates a new download manager.
    ///
    /// # Parameters
    ///
    /// - `client`: Configured HTTP client
    pub fn new(client: NetClient) -> Self {
        Self {
            downloader: Downloader::new(client),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a download task.
    ///
    /// Returns the task UUID immediately after starting the download; the download proceeds asynchronously in the background.
    ///
    /// # Parameters
    ///
    /// - `url`: Download URL
    /// - `output_path`: Local save path
    /// - `thread_count`: Number of download threads
    ///
    /// # Returns
    ///
    /// Returns the task UUID, which can be used later to query progress or cancel.
    pub async fn create(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<Uuid, DownloadError> {
        let status = self
            .downloader
            .download(url, output_path, thread_count)
            .await?;
        let id = Uuid::new_v4();

        let mut tasks = self.tasks.write().await;
        tasks.insert(id, status);

        observability::task_created(&id, url);

        Ok(id)
    }

    /// Queries the progress of a single task.
    ///
    /// Automatically removes the task from the manager when it finishes.
    ///
    /// # Parameters
    ///
    /// - `id`: Task UUID
    ///
    /// # Returns
    ///
    /// Returns `Some(DownloadSnapshot)` if the task exists, `None` otherwise.
    pub async fn get_progress(&self, id: Uuid) -> Option<DownloadSnapshot> {
        let status = {
            let tasks = self.tasks.read().await;
            tasks.get(&id).cloned()?
        };

        let snap = status.snapshot().await;

        // Automatically remove finished tasks to free management resources.
        // An acceptable race condition: two concurrent calls may both see the task as finished,
        // and both attempt to remove it. The second remove is idempotent and does not affect correctness.
        if snap.is_finished {
            let mut tasks = self.tasks.write().await;
            tasks.remove(&id);
        }

        Some(snap)
    }

    /// Queries progress of all tasks.
    ///
    /// Returns progress of all ongoing tasks; finished tasks are automatically cleaned up.
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

    /// Cancels a download task.
    ///
    /// The task will be removed from the manager after cancellation.
    ///
    /// # Parameters
    ///
    /// - `id`: Task UUID
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

    /// Returns the current number of managed tasks.
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
