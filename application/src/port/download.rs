//! 下载任务管理服务端口。

use async_trait::async_trait;
use sealantern_contract::DownloadServiceError;
use sealantern_contract::download::{DownloadRequest, DownloadTaskInfo};

/// 下载任务管理宿主能力端口。
///
/// 管理下载任务的创建、进度查询与取消。实现方组合 `infra` 的下载执行能力，
/// 不依赖任何具体宿主。
#[async_trait]
pub trait DownloadService: Send + Sync {
    /// 创建下载任务，返回任务 ID。
    async fn create(&self, request: DownloadRequest) -> Result<String, DownloadServiceError>;

    /// 查询任务进度；任务不存在时返回 `None`。
    async fn poll(&self, id: &str) -> Result<Option<DownloadTaskInfo>, DownloadServiceError>;

    /// 取消下载任务。
    async fn cancel(&self, id: &str) -> Result<(), DownloadServiceError>;
}
