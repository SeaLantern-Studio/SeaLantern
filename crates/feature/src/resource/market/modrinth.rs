//! Modrinth 市场数据获取器。
//!
//! 对应 [Modrinth API v2](https://api.modrinth.com/v2)，用于从 Modrinth 平台
//! 搜索和获取 Mod / 插件资源信息。所有请求均需携带 `User-Agent` 头。
//! Modrinth 的响应结构较为规范，支持分页、多加载器（Fabric、Forge、Quilt 等）
//! 以及多游戏版本，资源类型包括 `mod`、`plugin`、`datapack` 等。
//!
//! # 关于字段命名
//!
//! Modrinth API v2 响应字段统一使用 **snake_case** 风格（如 `project_id`、
//! `total_hits`、`game_versions`），因此本模块中的反序列化结构体直接使用
//! 同名 Rust 字段，无需 `#[serde(rename_all)]` 转换。

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

/// Modrinth API 的基础 URL。
const MODRINTH_BASE: &str = "https://api.modrinth.com/v2";

// ─── Modrinth API 响应结构体 ─────────────────────────────────────────────

/// 搜索 API (`GET /search`) 的顶层响应。
#[derive(Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthSearchHit>,
    total_hits: u64,
    offset: u64,
}

/// 搜索命中的单个项目摘要。
#[derive(Deserialize)]
struct ModrinthSearchHit {
    project_id: String,
    title: String,
    description: String,
    downloads: u64,
}

/// 项目详细信息 (`GET /project/{id}`)。
#[derive(Deserialize)]
struct ModrinthProject {
    id: String,
    title: String,
    description: String,
    downloads: u64,
    icon_url: Option<String>,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    project_type: String,
}

/// 项目的单个版本 (`GET /project/{id}/version`)。
#[derive(Deserialize)]
struct ModrinthVersion {
    id: String,
    name: String,
    version_number: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    downloads: u64,
    files: Vec<ModrinthVersionFile>,
}

/// 版本中关联的文件信息。
#[derive(Deserialize)]
struct ModrinthVersionFile {
    url: String,
    filename: String,
    size: u64,
    primary: bool,
}

// ─── ModrinthFetcher ─────────────────────────────────────────────────────

/// 基于 Modrinth API 的资源获取器。
///
/// 持有客户端获取器（provider）与显式注入的下载管理器，避免全局单例：
/// 每次请求前获取当前全局客户端，保证代理更新即时生效。
pub struct ModrinthFetcher {
    client_provider: ClientProvider,
    download: DownloadManager,
}

impl ModrinthFetcher {
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
    ///
    /// # Parameters
    /// - `client_provider`: 返回当前 `NetClient` 的获取器。
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
impl Fetcher for ModrinthFetcher {
    /// 在 Modrinth 市场中搜索资源。
    ///
    /// 调用 `GET /search?query={query}&limit={page_size}&offset={offset}`。
    async fn search(
        &self,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, MarketError> {
        if page == 0 {
            return Err(MarketError::config("page must be 1 or greater"));
        }
        observability::market_search_started(query, page, page_size, "modrinth");

        let offset = (page - 1) * page_size;
        let url = format!(
            "{}/search?query={}&limit={}&offset={}",
            MODRINTH_BASE,
            urlencoding::encode(query),
            page_size,
            offset
        );

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "search resources", "modrinth").await?;
        let search_resp: ModrinthSearchResponse = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse search results", "modrinth", e.to_string()))?;

        let resources: Vec<MarketResource> = search_resp
            .hits
            .into_iter()
            .map(|hit| MarketResource {
                id: hit.project_id,
                name: hit.title,
                description: hit.description,
                download_count: hit.downloads,
                source: MarketSource::Modrinth,
            })
            .collect();

        observability::market_search_completed(query, search_resp.total_hits, "modrinth");

        Ok(SearchResult {
            total: search_resp.total_hits,
            offset: search_resp.offset,
            limit: page_size as u64,
            resources,
        })
    }

    /// 获取 Modrinth 上指定项目的详细信息。
    ///
    /// 调用 `GET /project/{id}`。注意 Modrinth 详情接口**不返回文件下载 URL**，
    /// 因此 [`ResourceInfo::download_url`] 保持为空，下载请走版本列表
    /// （[`Fetcher::get_resource_versions`] 的 `files[].url`）。
    async fn get_resource(&self, id: &str) -> Result<ResourceInfo, MarketError> {
        let url = format!("{}/project/{}", MODRINTH_BASE, id);

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "get resource details", "modrinth").await?;
        let project: ModrinthProject = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse resource details", "modrinth", e.to_string()))?;

        let info = ResourceInfo {
            id: project.id,
            name: project.title,
            description: project.description,
            download_count: project.downloads,
            source: MarketSource::Modrinth,
            icon_url: project.icon_url,
            game_versions: project.game_versions,
            loaders: project.loaders,
            resource_type: ResourceType::from_platform_value(&project.project_type),
            external: false,
            download_url: String::new(),
        };

        observability::market_resource_fetched(id, &info.name, "modrinth");
        Ok(info)
    }

    /// 获取指定项目的所有版本列表。
    ///
    /// 调用 `GET /project/{id}/version`。
    async fn get_resource_versions(&self, id: &str) -> Result<Vec<Version>, MarketError> {
        let url = format!("{}/project/{}/version", MODRINTH_BASE, id);

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "get resource versions", "modrinth").await?;
        let versions: Vec<ModrinthVersion> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse version list", "modrinth", e.to_string()))?;

        let result: Vec<Version> = versions
            .into_iter()
            .map(|v| {
                let files: Vec<VersionFile> = v
                    .files
                    .into_iter()
                    .map(|f| VersionFile {
                        url: f.url,
                        filename: f.filename,
                        size: f.size,
                        primary: f.primary,
                    })
                    .collect();

                Version {
                    id: v.id,
                    name: v.name,
                    version_number: v.version_number,
                    game_versions: v.game_versions,
                    loaders: v.loaders,
                    downloads: v.downloads,
                    files,
                }
            })
            .collect();

        Ok(result)
    }

    /// 下载资源文件。
    ///
    /// 委托给显式注入的下载管理器执行，不使用全局单例。
    async fn download_resource(
        &self,
        url: &str,
        destination: &str,
    ) -> Result<Arc<DownloadStatus>, MarketError> {
        observability::market_download_started(url, "modrinth");
        download_file(&self.download, url, destination).await
    }

    /// 获取随机资源列表（Modrinth 原生支持该接口）。
    async fn get_random_resources(&self, count: u32) -> Result<Vec<MarketResource>, MarketError> {
        let limit = count.min(10);
        let url = format!("{}/projects_random?count={}", MODRINTH_BASE, limit);

        let client = (self.client_provider)().map_err(|e| MarketError::config(e.to_string()))?;
        let resp = send_get(client, &url, "get random resources", "modrinth").await?;
        let projects: Vec<ModrinthProject> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse random resources", "modrinth", e.to_string()))?;

        Ok(projects
            .into_iter()
            .map(|p| MarketResource {
                id: p.id,
                name: p.title,
                description: p.description,
                download_count: p.downloads,
                source: MarketSource::Modrinth,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::resource::market::models::MarketSource;

    use super::*;

    fn test_fetcher() -> ModrinthFetcher {
        let client = sealantern_infra::net::NetClient::from_config(&Default::default()).unwrap();
        ModrinthFetcher::new(client)
    }

    /// 需要真实网络与第三方 API 可用性。CI / 离线环境请用 `--ignored` 显式运行。
    #[tokio::test]
    #[ignore = "依赖真实 Modrinth API 与网络"]
    async fn test_search_returns_results() {
        let fetcher = test_fetcher();
        let result = fetcher.search("sodium", 1, 5).await.unwrap();
        assert!(!result.resources.is_empty());
        assert!(result.total > 0);
        for r in &result.resources {
            assert_eq!(r.source, MarketSource::Modrinth);
        }
    }

    #[tokio::test]
    #[ignore = "依赖真实 Modrinth API 与网络"]
    async fn test_get_resource_sodium() {
        let fetcher = test_fetcher();
        let info = fetcher.get_resource("sodium").await.unwrap();
        assert_eq!(info.name, "Sodium");
        assert!(!info.external);
        assert!(info.download_count > 0);
        assert!(!info.game_versions.is_empty());
        assert!(!info.loaders.is_empty());
    }

    #[tokio::test]
    #[ignore = "依赖真实 Modrinth API 与网络"]
    async fn test_get_resource_versions_returns_list() {
        let fetcher = test_fetcher();
        let versions = fetcher.get_resource_versions("sodium").await.unwrap();
        assert!(!versions.is_empty());
        for v in &versions {
            assert!(!v.version_number.is_empty());
            assert!(!v.files.is_empty());
        }
    }

    #[tokio::test]
    #[ignore = "依赖真实 Modrinth API 与网络"]
    async fn test_get_random_resources() {
        let fetcher = test_fetcher();
        let resources = fetcher.get_random_resources(3).await.unwrap();
        assert!(!resources.is_empty());
        assert!(resources.len() <= 3);
        for r in &resources {
            assert_eq!(r.source, MarketSource::Modrinth);
            assert!(!r.name.is_empty());
        }
    }

    #[tokio::test]
    async fn provider_failure_propagates_without_network() {
        // 假 provider：每次请求前被调用并返回失败，验证请求不会发出网络调用。
        let fetcher = ModrinthFetcher::with_provider(Box::new(|| {
            Err(sealantern_infra::net::NetError::Config("模拟获取客户端失败".into()))
        }));
        let error = fetcher.search("sodium", 1, 5).await.unwrap_err();
        assert!(matches!(error, MarketError::Config(_)));
    }
}
