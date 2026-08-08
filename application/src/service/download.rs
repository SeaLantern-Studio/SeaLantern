//! 下载任务管理服务实现。
//!
//! 实现 [`sealantern_interface::DownloadService`] 能力端口，显式持有
//! `infra` 的 [`DownloadManager`]（而非被其全局单例反向绑定），管理
//! 下载任务的创建、进度查询与取消。
//!
//! 错误分层：内部以应用层主错误 [`DownloadError`] 为源头，暴露
//! [`DownloadService`] 时统一转为接口契约错误 [`DownloadServiceError`]。

use async_trait::async_trait;
use sealantern_infra::download::DownloadManager;
use sealantern_infra::net::client::{ClientConfig, NetClient};
use sealantern_interface::download::{DownloadRequest, DownloadTaskInfo, DownloadTaskStatus};
use sealantern_interface::DownloadServiceError;

use crate::error::DownloadError;

/// 基于 `infra` 下载能力的下载任务管理服务实现。
pub struct CoreDownloadService {
    /// 下载任务管理器（显式持有，非全局单例）。
    manager: DownloadManager,
}

impl CoreDownloadService {
    /// 使用默认 HTTP 客户端配置构造下载服务。
    pub fn new() -> Result<Self, DownloadError> {
        let client = NetClient::from_config(&ClientConfig::default())
            .map_err(|e| DownloadError::OperationFailed { source: Box::new(e) })?;
        Ok(Self { manager: DownloadManager::new(client) })
    }

    /// 从既有管理器构造下载服务（便于测试注入）。
    pub fn with_manager(manager: DownloadManager) -> Self {
        Self { manager }
    }
}

impl Default for CoreDownloadService {
    fn default() -> Self {
        Self::new().expect("failed to construct default download service")
    }
}

#[async_trait]
impl sealantern_interface::DownloadService for CoreDownloadService {
    async fn create(&self, request: DownloadRequest) -> Result<String, DownloadServiceError> {
        if request.url.trim().is_empty() || request.save_path.trim().is_empty() {
            return Err(DownloadError::InvalidInput.into());
        }

        let (id, _) = self
            .manager
            .create_with_handle(&request.url, &request.save_path, request.thread_count)
            .await
            .map_err(DownloadError::from)?;
        Ok(id.to_string())
    }

    async fn poll(&self, id: &str) -> Result<Option<DownloadTaskInfo>, DownloadServiceError> {
        let task_id = parse_task_id(id)?;
        let Some(snapshot) = self.manager.get_progress(task_id).await else {
            return Ok(None);
        };

        Ok(Some(DownloadTaskInfo {
            id: id.to_string(),
            total_size: snapshot.total_size,
            downloaded: snapshot.downloaded,
            progress: snapshot.progress_percentage,
            status: to_frontend_status(&snapshot),
            is_finished: snapshot.is_finished,
        }))
    }

    async fn cancel(&self, id: &str) -> Result<(), DownloadServiceError> {
        let task_id = parse_task_id(id)?;
        self.manager.cancel(task_id).await;
        Ok(())
    }
}

/// 解析任务 ID 字符串为 UUID。
fn parse_task_id(id: &str) -> Result<uuid::Uuid, DownloadError> {
    uuid::Uuid::parse_str(id).map_err(|_| DownloadError::InvalidInput)
}

/// 将后端下载快照映射为前端任务状态。
fn to_frontend_status(
    snapshot: &sealantern_infra::download::DownloadSnapshot,
) -> DownloadTaskStatus {
    if let Some(error) = &snapshot.error {
        DownloadTaskStatus::Error(error.clone())
    } else if snapshot.is_finished {
        DownloadTaskStatus::Completed
    } else if snapshot.downloaded > 0 {
        DownloadTaskStatus::Downloading
    } else {
        DownloadTaskStatus::Pending
    }
}
