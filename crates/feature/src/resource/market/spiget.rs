//! Spiget 市场数据获取器。
//!
//! 对应 [Spiget API v2](https://api.spiget.org/v2)，用于从 SpigotMC 官方市场
//! 搜索和获取插件资源信息。响应格式为 JSON，部分端点返回的数据可能包裹在 `value` 字段中。
//! 注意：Spiget API 有速率限制，生产环境中建议配合缓存使用。

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use sealantern_infra::download::{DownloadManager, DownloadStatus};
use sealantern_infra::net::{ClientProvider, NetClient};

use crate::observability;

use super::error::MarketError;
use super::models::*;
use super::traits::Fetcher;
use super::{download_file, send_get};

/// Spiget API 的基础 URL。
const SPIGET_BASE: &str = "https://api.spiget.org/v2";

// ─── Spiget API 响应结构体（自动反序列化） ──────────────────────────────

/// Spiget 资源详情响应。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpigetResource {
    id: i64,
    name: String,
    tag: String,
    external: bool,
    downloads: i64,
    file: SpigetFile,
    tested_versions: Vec<String>,
}

/// Spiget 资源文件信息。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpigetFile {
    external_url: Option<String>,
}

/// Spiget 搜索命中项（资源列表项）。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpigetSearchHit {
    id: i64,
    name: String,
    tag: String,
    downloads: i64,
}

/// Spiget 版本响应（可能包裹在 `value` 中）。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpigetVersion {
    id: i64,
    name: String,
    downloads: i64,
}

/// 带 `value` 包裹的版本列表。
#[derive(Deserialize)]
struct SpigetVersionList {
    value: Vec<SpigetVersion>,
}

// ─── SpigetFetcher ───────────────────────────────────────────────────────

/// 基于 Spiget API 的资源获取器。
///
/// 持有客户端获取器（provider）与显式注入的下载管理器，避免全局单例：
/// 每次请求前获取当前全局客户端，保证代理更新即时生效。
pub struct SpigetFetcher {
    client_provider: ClientProvider,
    download: DownloadManager,
}

impl SpigetFetcher {
    /// 使用全局客户端获取器与全局下载器构造（生产装配推荐）。
    pub fn global() -> Self {
        Self {
            client_provider: sealantern_infra::net::global_client_provider(),
            download: DownloadManager::with_provider(
                sealantern_infra::net::global_client_provider(),
            ),
        }
    }

    /// 使用自定义客户端获取器构造（测试注入）；下载器使用全局配置。
    pub fn with_provider(client_provider: ClientProvider) -> Self {
        Self {
            client_provider,
            download: DownloadManager::with_provider(
                sealantern_infra::net::global_client_provider(),
            ),
        }
    }

    /// 使用具体客户端构造（兼容旧调用与测试注入）。
    pub fn new(client: NetClient) -> Self {
        Self::with_provider(Box::new(move || Ok(client.clone())))
    }
}

#[async_trait]
impl Fetcher for SpigetFetcher {
    /// 在 Spiget 市场中搜索资源。
    ///
    /// 调用 `GET /search/resources/{query}?size={page_size}&page={page}`。
    /// Spiget 的 `page` 从 1 开始；`offset` 语义为"当前页之前已跳过的条目数"，
    /// 与 Modrinth 保持一致（`(page - 1) * page_size`）。
    async fn search(
        &self,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, MarketError> {
        if page == 0 {
            return Err(MarketError::config("page must be 1 or greater"));
        }
        observability::market_search_started(query, page, page_size, "spiget");

        let url = format!(
            "{}/search/resources/{}?size={}&page={}",
            SPIGET_BASE,
            urlencoding::encode(query),
            page_size,
            page
        );

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "search resources", "spiget").await?;
        let hits: Vec<SpigetSearchHit> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse search results", "spiget", e.to_string()))?;

        let items: Vec<MarketResource> = hits
            .into_iter()
            .map(|hit| MarketResource {
                id: hit.id.to_string(),
                name: hit.name,
                description: hit.tag,
                download_count: hit.downloads as u64,
                source: MarketSource::Spiget,
            })
            .collect();

        observability::market_search_completed(query, items.len() as u64, "spiget");

        Ok(SearchResult {
            // Spiget 不返回匹配总数，以当前页数量近似。
            total: items.len() as u64,
            offset: ((page - 1) * page_size) as u64,
            limit: page_size as u64,
            resources: items,
        })
    }

    /// 获取 Spiget 上指定资源的详细信息。
    ///
    /// 调用 `GET /resources/{id}`。Spiget 会对插件给出 `external` 标记与
    /// 下载 URL（外部托管或 CDN），因此 [`ResourceInfo::download_url`] 会被填充。
    async fn get_resource(&self, id: &str) -> Result<ResourceInfo, MarketError> {
        let url = format!("{}/resources/{}", SPIGET_BASE, id);

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "get resource details", "spiget").await?;
        let resource: SpigetResource = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse resource details", "spiget", e.to_string()))?;

        let download_url = build_spiget_download_url(&resource, id);

        observability::market_resource_fetched(id, &resource.name, "spiget");

        Ok(ResourceInfo {
            id: resource.id.to_string(),
            name: resource.name,
            description: resource.tag,
            download_count: resource.downloads as u64,
            source: MarketSource::Spiget,
            icon_url: None,
            game_versions: resource.tested_versions,
            loaders: vec!["spigot".to_string()],
            resource_type: ResourceType::Plugin,
            external: resource.external,
            download_url,
        })
    }

    /// 获取指定资源的所有版本列表。
    ///
    /// 调用 `GET /resources/{id}/versions?size=100`。Spiget 响应有两种格式：
    /// 直接返回数组 `[...]` 或包裹在 `{"value": [...]}` 中，两者都兼容。
    async fn get_resource_versions(&self, id: &str) -> Result<Vec<Version>, MarketError> {
        let url = format!("{}/resources/{}/versions?size=100", SPIGET_BASE, id);

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "get resource versions", "spiget").await?;
        let outer: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse version list", "spiget", e.to_string()))?;

        let versions_raw = if outer.get("value").and_then(|v| v.as_array()).is_some() {
            serde_json::from_value::<SpigetVersionList>(outer)
                .map_err(|e| MarketError::json("parse version list", "spiget", e.to_string()))?
                .value
        } else {
            serde_json::from_value::<Vec<SpigetVersion>>(outer)
                .map_err(|e| MarketError::json("parse version list", "spiget", e.to_string()))?
        };

        // version_number 复用 name（Spiget 不单独提供语义化版本号）。
        let versions: Vec<Version> = versions_raw
            .into_iter()
            .map(|v| Version {
                id: v.id.to_string(),
                name: v.name.clone(),
                version_number: v.name,
                game_versions: vec![],
                loaders: vec!["spigot".to_string()],
                downloads: v.downloads as u64,
                files: vec![],
            })
            .collect();

        observability::market_versions_fetched(id, versions.len(), "spiget");

        Ok(versions)
    }

    /// 下载资源文件。
    ///
    /// 委托给显式注入的下载管理器执行，不使用全局单例。
    async fn download_resource(
        &self,
        url: &str,
        destination: &str,
    ) -> Result<Arc<DownloadStatus>, MarketError> {
        observability::market_download_started(url, "spiget");
        download_file(&self.download, url, destination).await
    }

    /// 获取随机资源列表。
    ///
    /// Spiget 无原生随机接口；这里以时间为种子选取一个"下载量倒序"的页面。
    async fn get_random_resources(&self, count: u32) -> Result<Vec<MarketResource>, MarketError> {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let page = (seed % 100) as u32 + 1;
        let limit = count.min(8);
        let url = format!("{}/resources?size={}&page={}&sort=-downloads", SPIGET_BASE, limit, page);

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "get random resources", "spiget").await?;
        let list: Vec<SpigetSearchHit> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse random resources", "spiget", e.to_string()))?;

        Ok(list
            .into_iter()
            .map(|h| MarketResource {
                id: h.id.to_string(),
                name: h.name,
                description: h.tag,
                download_count: h.downloads as u64,
                source: MarketSource::Spiget,
            })
            .collect())
    }
}

/// 根据资源信息构建 Spiget 下载 URL。
///
/// 若资源为外部托管（`external == true`），则返回 `file.external_url`；
/// 否则返回 Spiget CDN 的标准下载路径。
fn build_spiget_download_url(resource: &SpigetResource, id: &str) -> String {
    if resource.external {
        resource.file.external_url.clone().unwrap_or_default()
    } else {
        format!("{}/resources/{}/download", SPIGET_BASE, id)
    }
}

#[cfg(test)]
mod tests {
    use crate::resource::market::models::MarketSource;

    use super::*;

    fn test_fetcher() -> SpigetFetcher {
        let client = sealantern_infra::net::NetClient::from_config(&Default::default()).unwrap();
        SpigetFetcher::new(client)
    }

    /// 需要真实网络与第三方 API 可用性。CI / 离线环境请用 `--ignored` 显式运行。
    #[tokio::test]
    #[ignore = "依赖真实 Spiget API 与网络"]
    async fn test_search_returns_results() {
        let fetcher = test_fetcher();
        let result = fetcher.search("luckperms", 1, 5).await.unwrap();
        assert!(!result.resources.is_empty());
        assert!(result.total > 0);
        for r in &result.resources {
            assert_eq!(r.source, MarketSource::Spiget);
        }
    }

    /// 依赖 Spiget 平台具体资源 ID（外部托管示例），资源变动会导致测试失败。
    #[tokio::test]
    #[ignore = "依赖真实 Spiget API、网络与具体资源 ID"]
    async fn test_external_resource_66647() {
        let fetcher = test_fetcher();
        let info = fetcher.get_resource("66647").await.unwrap();
        assert_eq!(info.name, "Waypoints");
        assert!(info.external);
        assert!(!info.download_url.is_empty());
        assert!(info.download_url.contains("modrinth") || info.download_url.contains("http"));
    }

    /// 依赖 Spiget 平台具体资源 ID（CDN 托管示例）。
    #[tokio::test]
    #[ignore = "依赖真实 Spiget API、网络与具体资源 ID"]
    async fn test_cdn_resource_28140() {
        let fetcher = test_fetcher();
        let info = fetcher.get_resource("28140").await.unwrap();
        assert_eq!(info.name, "LuckPerms");
        assert!(!info.external);
        assert!(info.download_url.contains("spiget.org"));
    }

    #[tokio::test]
    #[ignore = "依赖真实 Spiget API 与网络"]
    async fn test_resource_versions_returns_list() {
        let fetcher = test_fetcher();
        let versions = fetcher.get_resource_versions("28140").await.unwrap();
        assert!(!versions.is_empty());
        for v in &versions {
            assert!(!v.id.is_empty());
            assert!(!v.name.is_empty());
        }
    }

    #[tokio::test]
    #[ignore = "依赖真实 Spiget API 与网络"]
    async fn test_get_random_resources() {
        let fetcher = test_fetcher();
        let resources = fetcher.get_random_resources(3).await.unwrap();
        assert!(!resources.is_empty());
        assert!(resources.len() <= 3);
        for r in &resources {
            assert_eq!(r.source, MarketSource::Spiget);
            assert!(!r.name.is_empty());
        }
    }

    #[tokio::test]
    async fn provider_failure_propagates_without_network() {
        // 假 provider：每次请求前被调用并返回失败，验证请求不会发出网络调用。
        let fetcher = SpigetFetcher::with_provider(Box::new(|| {
            Err(sealantern_infra::net::NetError::Config("模拟获取客户端失败".into()))
        }));
        let error = fetcher.search("luckperms", 1, 5).await.unwrap_err();
        assert!(matches!(error, MarketError::Config(_)));
    }
}
