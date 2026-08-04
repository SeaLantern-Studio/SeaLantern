//! 资源获取器（Fetcher）trait 定义。
//!
//! 定义 [`Fetcher`] trait，作为不同资源平台的统一数据获取接口。

use std::sync::Arc;

use async_trait::async_trait;

use crate::market::error::MarketError;
use crate::market::models::{MarketResource, ResourceInfo, SearchResult, Version};

/// 统一资源获取器 trait，定义了对接不同资源平台所需的核心操作。
///
/// 每个平台（如 Spigot、Modrinth）需要实现该 trait，以提供标准化的
/// 搜索、项目信息查询、版本列表获取以及资源下载能力。
#[async_trait]
pub trait Fetcher {
    /// 根据关键词搜索资源，支持分页。
    ///
    /// # Parameters
    /// - `query` — 搜索关键词
    /// - `page` — 页码，从 1 开始；传入 0 会返回错误
    /// - `page_size` — 每页返回的结果数量
    ///
    /// # Returns
    /// 返回 [`SearchResult`]，包含匹配的项目列表及分页信息。
    async fn search(
        &self,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, MarketError>;

    /// 根据资源 ID 获取项目的详细信息。
    ///
    /// # Parameters
    /// - `id` — 资源在对应平台上的唯一标识符
    ///
    /// # Returns
    /// 返回 [`ResourceInfo`]，包含项目的名称、描述、作者、图标等元数据。
    async fn get_resource(&self, id: &str) -> Result<ResourceInfo, MarketError>;

    /// 获取指定资源的所有可用版本。
    ///
    /// # Parameters
    /// - `id` — 资源在对应平台上的唯一标识符
    ///
    /// # Returns
    /// 返回版本列表 [`Vec<Version>`]，每个版本包含版本号、发布日期、下载链接等信息。
    async fn get_resource_versions(&self, id: &str) -> Result<Vec<Version>, MarketError>;

    /// 从指定 URL 下载资源到本地路径。
    ///
    /// # Parameters
    /// - `url` — 资源的下载链接
    /// - `destination` — 本地保存路径（包含文件名）
    ///
    /// # Returns
    /// 返回 [`DownloadStatus`] 的共享引用，可用于跟踪下载进度与结果。
    async fn download_resource(
        &self,
        url: &str,
        destination: &str,
    ) -> Result<Arc<sealantern_infra::download::DownloadStatus>, MarketError>;

    /// 获取随机资源列表，适用于"发现"或"推荐"场景。
    ///
    /// Modrinth 原生支持随机接口；Spiget 通过随机页码模拟。
    ///
    /// # Parameters
    /// - `count` — 期望返回的资源数量
    ///
    /// # Returns
    /// 返回资源列表，实际数量可能少于 `count`。
    async fn get_random_resources(&self, count: u32) -> Result<Vec<MarketResource>, MarketError>;
}
