//! Spiget 市场数据获取器。
//!
//! 对应 [Spiget API v2](https://api.spiget.org/v2)，用于从 SpigotMC 官方市场
//! 搜索和获取插件资源信息。响应格式为 JSON，部分端点返回的数据可能包裹在 `value` 字段中。
//! 注意：Spiget API 有速率限制，生产环境中建议配合缓存使用。

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use sealantern_infra::net::NetClient;

use crate::market::error::MarketError;
use crate::market::fetcher;
use crate::market::fetcher::Fetcher;
use crate::market::models::*;
use crate::observability;

// ---------------------------------------------------------------------------
// Spiget API 响应结构体（自动反序列化）
// ---------------------------------------------------------------------------

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

/// Spiget 搜索结果条目。
#[derive(Deserialize)]
struct SpigetSearchHit {
    id: i64,
    name: String,
    tag: String,
    downloads: i64,
}

/// Spiget 版本列表（可能包裹在 `value` 字段中）。
#[derive(Deserialize)]
struct SpigetVersionList {
    value: Vec<SpigetVersion>,
}

/// Spiget 版本信息。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpigetVersion {
    id: i64,
    name: String,
    downloads: i64,
}

// ---------------------------------------------------------------------------
// 获取器实现
// ---------------------------------------------------------------------------

/// Spiget API 的基础 URL。
const SPIGET_BASE: &str = "https://api.spiget.org/v2";

/// 基于 Spiget API 的资源获取器。
///
/// 通过 `NetClient` 向 Spiget API 发送 HTTP 请求，解析返回的 JSON 数据，
/// 并将其映射为内部统一的 `MarketResource`、`ResourceInfo`、`Version` 等模型。
pub struct SpigetFetcher {
    client: NetClient,
}

impl SpigetFetcher {
    /// 创建一个新的 `SpigetFetcher`。
    ///
    /// # Parameters
    /// - `client`: 用于发送 HTTP 请求的 `NetClient` 实例。
    ///
    /// # Returns
    /// 返回初始化完成的 `SpigetFetcher`。
    pub fn new(client: NetClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Fetcher for SpigetFetcher {
    /// 在 Spiget 市场中搜索资源。
    ///
    /// 调用 `GET /search/resources/{query}?size={page_size}&page={page}`，
    /// 返回匹配的资源列表。响应体为 JSON 数组，直接反序列化为 `Vec<SpigetSearchHit>`。
    ///
    /// # Parameters
    /// - `query`: 搜索关键词。
    /// - `page`: 页码，从 1 开始；传入 0 会返回错误。
    /// - `page_size`: 每页结果数。
    ///
    /// # Returns
    /// 包含分页信息和资源列表的 `SearchResult`。
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
        let url =
            format!("{}/search/resources/{}?size={}&page={}", SPIGET_BASE, urlencoding::encode(query), page_size, page);
        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .send()
            .await
            .map_err(|e| MarketError::http("search resources", "spiget", e.to_string()))?;

        // 直接反序列化为 Vec<SpigetSearchHit>
        let hits: Vec<SpigetSearchHit> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse search results", "spiget", e.to_string()))?;

        // 将搜索结果映射为统一的 MarketResource
        // tag 字段作为简要描述，downloads 字段作为下载量
        let items: Vec<MarketResource> = hits
            .into_iter()
            .map(|hit| MarketResource {
                id: hit.id.to_string(),
                name: hit.name,
                description: hit.tag,
                download_count: hit.downloads as u64,
                version_count: 0,
                source: MarketSource::Spiget,
            })
            .collect();

        observability::market_search_completed(query, items.len() as u64, "spiget");

        Ok(SearchResult {
            total: items.len() as u64,
            offset: (page * page_size) as u64,
            limit: page_size as u64,
            resources: items,
        })
    }

    /// 获取 Spiget 上指定资源的详细信息。
    ///
    /// 调用 `GET /resources/{id}`，返回单个资源的完整信息。
    /// 响应直接反序列化为 `SpigetResource`，再映射为 `ResourceInfo`。
    ///
    /// # Parameters
    /// - `id`: 资源的数字 ID（由 Spiget 分配）。
    ///
    /// # Returns
    /// 包含资源详细元数据的 `ResourceInfo`。
    async fn get_resource(&self, id: &str) -> Result<ResourceInfo, MarketError> {
        let url = format!("{}/resources/{}", SPIGET_BASE, id);
        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .send()
            .await
            .map_err(|e| MarketError::http("get resource details", "spiget", e.to_string()))?;

        let resource: SpigetResource = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse resource details", "spiget", e.to_string()))?;

        // 从 SpigetResource 构建 ResourceInfo
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
            resource_type: "plugin".to_string(),
            external: resource.external,
            download_url,
        })
    }

    /// 获取指定资源的所有版本列表。
    ///
    /// 调用 `GET /resources/{id}/versions?size=100`。
    /// Spiget 响应可能有两种格式：直接返回数组 `[...]`，或包裹在 `{"value": [...]}` 中。
    /// 本方法先尝试 `SpigetVersionList`（带 value 包裹），失败则回退到 `Vec<SpigetVersion>`。
    ///
    /// # Parameters
    /// - `id`: 资源的数字 ID。
    ///
    /// # Returns
    /// 版本对象列表，每个版本包含名称、下载量等信息。
    async fn get_resource_versions(&self, id: &str) -> Result<Vec<Version>, MarketError> {
        let url = format!("{}/resources/{}/versions?size=100", SPIGET_BASE, id);
        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .send()
            .await
            .map_err(|e| MarketError::http("get resource versions", "spiget", e.to_string()))?;

        let outer: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse version list", "spiget", e.to_string()))?;

        // 兼容两种响应格式：{ value: [...] } 或 [...]
        let versions_raw = if outer.get("value").and_then(|v| v.as_array()).is_some() {
            serde_json::from_value::<SpigetVersionList>(outer)
                .map_err(|e| MarketError::json("parse version list", "spiget", e.to_string()))?
                .value
        } else {
            serde_json::from_value::<Vec<SpigetVersion>>(outer)
                .map_err(|e| MarketError::json("parse version list", "spiget", e.to_string()))?
        };

        // 映射每个版本对象，version_number 复用 name 字段（Spiget 不单独提供版本号）
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
    /// 委托给 `fetcher::download_file` 执行实际下载，不涉及 Spiget API 调用。
    ///
    /// # Parameters
    /// - `url`: 文件的直接下载链接。
    /// - `destination`: 保存路径。
    ///
    /// # Returns
    /// 下载任务的状态信息。
    async fn download_resource(
        &self,
        url: &str,
        destination: &str,
    ) -> Result<Arc<sealantern_infra::download::DownloadStatus>, MarketError> {
        observability::market_download_started(url, "spiget");
        let status = fetcher::download_file(url, destination).await?;
        Ok(status)
    }

    async fn get_random_resources(&self, count: u32) -> Result<Vec<MarketResource>, MarketError> {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let page = (seed % 100) as u32 + 1;
        let limit = count.min(8);
        let url = format!("{}/resources?size={}&page={}&sort=-downloads", SPIGET_BASE, limit, page);
        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .send()
            .await
            .map_err(|e| MarketError::http("get random resources", "spiget", e.to_string()))?;
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
                version_count: 0,
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
    use crate::market::fetcher::Fetcher;
    use crate::market::models::MarketSource;

    use super::*;

    fn test_fetcher() -> SpigetFetcher {
        let client = sealantern_infra::net::NetClient::from_config(&Default::default()).unwrap();
        SpigetFetcher::new(client)
    }

    #[tokio::test]
    async fn test_search_returns_results() {
        let fetcher = test_fetcher();
        let result = fetcher.search("luckperms", 1, 5).await.unwrap();
        assert!(!result.resources.is_empty());
        assert!(result.total > 0);
        for r in &result.resources {
            assert_eq!(r.source, MarketSource::Spiget);
        }
    }

    #[tokio::test]
    async fn test_external_resource_66647() {
        let fetcher = test_fetcher();
        let info = fetcher.get_resource("66647").await.unwrap();
        assert_eq!(info.name, "Waypoints");
        assert!(info.external);
        assert!(!info.download_url.is_empty());
        assert!(info.download_url.contains("modrinth") || info.download_url.contains("http"));
    }

    #[tokio::test]
    async fn test_cdn_resource_28140() {
        let fetcher = test_fetcher();
        let info = fetcher.get_resource("28140").await.unwrap();
        assert_eq!(info.name, "LuckPerms");
        assert!(!info.external);
        assert!(info.download_url.contains("spiget.org"));
    }

    #[tokio::test]
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
}

