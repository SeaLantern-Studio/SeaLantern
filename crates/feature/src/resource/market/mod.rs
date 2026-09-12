//! 资源市场（Market）模块。
//!
//! 本模块提供了与 Minecraft 插件/模组资源市场交互的能力，
//! 支持从 Spiget 和 Modrinth 等源获取资源信息、搜索资源、
//! 查询版本详情及下载资源文件等功能。
//!
//! 模块结构：
//! - [`error`]：市场操作相关的错误类型定义；
//! - [`models`]：资源、版本、搜索结果等数据模型；
//! - [`traits`]：统一资源获取器（[`Fetcher`]）trait；
//! - `modrinth` / `spiget`：两平台的具体获取器实现。

use std::sync::Arc;

use sealantern_infra::download::{DownloadManager, DownloadStatus};
use sealantern_infra::net::NetClient;

pub mod error;
pub mod models;
pub mod traits;

mod modrinth;
mod spiget;

pub use error::MarketError;
pub use models::{
    MarketResource, MarketSource, ResourceInfo, ResourceType, SearchResult, Version, VersionFile,
};
pub use modrinth::ModrinthFetcher;
pub use spiget::SpigetFetcher;
pub use traits::Fetcher;

/// 市场 API 请求使用的 User-Agent。
pub(crate) const USER_AGENT: &str = "SeaLantern/feature/0.1.0";

/// 使用给定的网络客户端发送一次带 User-Agent 的 GET 请求。
///
/// 封装各平台获取器重复的"取客户端 → 构造请求 → 发送 → 映射错误"样板，
/// 统一错误语义（HTTP 失败 → [`MarketError::Http`]）。
pub(crate) async fn send_get(
    client: NetClient,
    url: &str,
    operation: &'static str,
    backend: &'static str,
) -> Result<reqwest::Response, MarketError> {
    client
        .get(url)
        .map_err(|e| MarketError::config(e.to_string()))?
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| MarketError::http(operation, backend, e.to_string()))
}

/// 通过**显式注入**的下载管理器执行资源下载。
///
/// 不使用全局单例，便于测试注入，也与项目"显式持有服务"的装配风格一致。
pub(crate) async fn download_file(
    manager: &DownloadManager,
    url: &str,
    destination: &str,
) -> Result<Arc<DownloadStatus>, MarketError> {
    let (_id, status) = manager
        .create_with_handle(url, destination, 8)
        .await
        .map_err(|e| MarketError::download(e.to_string()))?;
    Ok(status)
}
