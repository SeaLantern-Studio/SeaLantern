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
use sealantern_infra::net::{ClientProvider, global_client_provider};
use sealantern_interface::DownloadServiceError;
use sealantern_interface::download::{DownloadRequest, DownloadTaskInfo, DownloadTaskStatus};

use crate::error::DownloadError;

/// 下载线程数上限（防止资源滥用）。
const MAX_DOWNLOAD_THREAD_COUNT: usize = 64;

/// 基于 `infra` 下载能力的下载任务管理服务实现。
pub struct CoreDownloadService {
    /// 下载任务管理器（显式持有，非全局单例）。
    manager: DownloadManager,
}

impl CoreDownloadService {
    /// 以全局网络客户端构造下载服务。
    ///
    /// 下载管理器内部持有全局客户端 provider，每次创建下载任务时
    /// 获取当前全局客户端，与代理设置保持一致。
    pub fn new() -> Self {
        Self {
            manager: DownloadManager::with_provider(global_client_provider()),
        }
    }

    /// 从客户端获取器构造下载服务（便于测试注入假 provider）。
    pub fn with_provider(client_provider: ClientProvider) -> Self {
        Self {
            manager: DownloadManager::with_provider(client_provider),
        }
    }

    /// 从既有管理器构造下载服务（便于测试注入）。
    pub fn with_manager(manager: DownloadManager) -> Self {
        Self { manager }
    }
}

impl Default for CoreDownloadService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl sealantern_interface::DownloadService for CoreDownloadService {
    async fn create(&self, request: DownloadRequest) -> Result<String, DownloadServiceError> {
        if request.url.trim().is_empty() || request.save_path.trim().is_empty() {
            return Err(DownloadError::InvalidInput.into());
        }
        // 校验线程数范围，避免 0 或超大值传播到 infra 层（资源滥用/意外错误）。
        if request.thread_count == 0 || request.thread_count > MAX_DOWNLOAD_THREAD_COUNT {
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

#[cfg(test)]
mod tests {
    use sealantern_interface::DownloadService;

    use super::*;

    #[tokio::test]
    async fn provider_failure_propagates_without_network() {
        // 假 provider：每次创建任务前被调用并返回失败，验证不会发出网络请求。
        let service = CoreDownloadService::with_provider(Box::new(|| {
            Err(sealantern_infra::net::NetError::Config("模拟获取客户端失败".into()))
        }));
        let request = DownloadRequest {
            url: "https://example.com/file.zip".to_owned(),
            save_path: "C:\\temp\\file.zip".to_owned(),
            thread_count: 8,
        };

        let error = service
            .create(request)
            .await
            .expect_err("provider failure must fail");
        assert_eq!(error, DownloadServiceError::OperationFailed);
    }
}
