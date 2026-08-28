//! 服务器目录服务实现。
//!
//! 实现 [`crate::port::ServerCatalogService`] 能力端口，组合
//! `feature` 的下载链接管理能力（[`LinkManager`]），向宿主提供可用的
//! 服务器类型、版本与下载详情查询。
//!
//! 错误分层：配置拉取 / 解析失败统一收敛为
//! [`ServerCatalogServiceError::OperationFailed`]；指定的类型不存在
//! 收敛为 [`ServerCatalogServiceError::NotFound`]。

use async_trait::async_trait;
use sealantern_contract::{DownloadLink, ServerCatalogServiceError};
use sealantern_feature::download_link::{LinkError, LinkManager};

use crate::port::ServerCatalogService;

/// 基于 `feature` 下载链接管理的服务器目录服务实现。
#[derive(Debug, Default)]
pub struct CoreServerCatalogService;

#[async_trait]
impl ServerCatalogService for CoreServerCatalogService {
    /// 查询可用的服务器类型列表。
    ///
    /// 类型列表来自远程下载链接配置，配置拉取 / 解析失败时收敛为
    /// [`ServerCatalogServiceError::OperationFailed`]。
    async fn server_types(&self) -> Result<Vec<String>, ServerCatalogServiceError> {
        LinkManager::get_server_types()
            .await
            .map_err(|_| ServerCatalogServiceError::OperationFailed)
    }
    /// 查询指定服务器类型支持的版本列表。
    ///
    /// 类型不存在时收敛为 [`ServerCatalogServiceError::NotFound`]；
    /// 底层配置拉取 / 解析失败收敛为
    /// [`ServerCatalogServiceError::OperationFailed`]。
    async fn versions(
        &self,
        server_type: String,
    ) -> Result<Vec<String>, ServerCatalogServiceError> {
        LinkManager::get_versions_by_type(&server_type)
            .await
            .map_err(map_link_error)
    }
    /// 查询指定服务器类型、指定版本的下载链接。
    ///
    /// 类型或版本不存在时收敛为 [`ServerCatalogServiceError::NotFound`]；
    /// 底层配置拉取 / 解析失败收敛为
    /// [`ServerCatalogServiceError::OperationFailed`]。
    async fn details(
        &self,
        server_type: String,
        server_version: String,
    ) -> Result<DownloadLink, ServerCatalogServiceError> {
        LinkManager::get_link_by_type_and_version(&server_type, &server_version)
            .await
            .map_err(map_link_error)
    }
}

/// 将下载链接查询错误按语义收敛为目录契约错误。
///
/// 配置 / 基础设施失败归为操作失败，指定条目缺失归为不存在；
/// 底层诊断消息在收敛前记录到日志，便于排查目录数据问题。
fn map_link_error(error: LinkError) -> ServerCatalogServiceError {
    match &error {
        LinkError::Config(message) => {
            tracing::warn!(
                target: "sealantern.application.catalog",
                error = %message,
                "catalog configuration is unavailable"
            );
            ServerCatalogServiceError::OperationFailed
        }
        LinkError::NotFound(message) => {
            tracing::debug!(
                target: "sealantern.application.catalog",
                error = %message,
                "catalog entry not found"
            );
            ServerCatalogServiceError::NotFound
        }
    }
}
