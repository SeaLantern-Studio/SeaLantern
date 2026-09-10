//! 服务器核心下载目录宿主能力端口。

use async_trait::async_trait;
use sealantern_contract::{DownloadLink, ServerCatalogServiceError};

/// 服务器核心下载目录宿主能力端口。
///
/// 方法均为异步：下载目录数据可能来自远程配置。实现方组合 `infra` 或 `feature` 的
/// 下载链接能力，不依赖任何具体宿主。
#[async_trait]
pub trait ServerCatalogService: Send + Sync {
    /// 返回全部可用的服务器核心类型。
    async fn server_types(&self) -> Result<Vec<String>, ServerCatalogServiceError>;
    /// 返回指定服务器核心类型的可用版本列表。
    async fn versions(&self, server_type: String)
    -> Result<Vec<String>, ServerCatalogServiceError>;
    /// 返回指定服务器核心类型、指定版本的下载链接。
    async fn details(
        &self,
        server_type: String,
        server_version: String,
    ) -> Result<DownloadLink, ServerCatalogServiceError>;
}
