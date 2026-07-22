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

use sealantern_infra::net::NetClient;

use crate::market::error::MarketError;
use crate::market::fetcher;
use crate::market::fetcher::models::VersionFile;
use crate::market::fetcher::Fetcher;
use crate::market::models::*;
use crate::observability;

/// Modrinth API 的基础 URL。
const MODRINTH_BASE: &str = "https://api.modrinth.com/v2";

// ─── Modrinth API 响应结构体 ─────────────────────────────────────────────
//
// 以下结构体用于通过 `#[derive(Deserialize)]` 自动反序列化 Modrinth API
// 的 JSON 响应，替代原先的手动 `body["field"]` 方式。
//
// Modrinth API v2 使用 snake_case 命名，而 Rust 结构体字段惯例也是
// snake_case，因此字段名直接一一对应，无需 rename 配置。

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
/// 通过 `NetClient` 向 Modrinth API 发送 HTTP 请求（均携带 `User-Agent`），
/// 解析返回的 JSON 数据，映射为内部统一的 `MarketResource`、`ResourceInfo`、
/// `Version` 等模型。
pub struct ModrinthFetcher {
    client: NetClient,
}

impl ModrinthFetcher {
    /// 创建一个新的 `ModrinthFetcher`。
    ///
    /// # Parameters
    /// - `client`: 用于发送 HTTP 请求的 `NetClient` 实例。
    ///
    /// # Returns
    /// 返回初始化完成的 `ModrinthFetcher`。
    pub fn new(client: NetClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Fetcher for ModrinthFetcher {
    /// 在 Modrinth 市场中搜索资源。
    ///
    /// 调用 `GET /search?query={query}&limit={page_size}&offset={offset}`，
    /// 通过 `ModrinthSearchResponse` 自动反序列化响应，提取 `hits` 列表和
    /// 分页元数据。
    ///
    /// # Parameters
    /// - `query`: 搜索关键词。
    /// - `page`: 页码，从 1 开始；传入 0 会返回错误。
    /// - `page_size`: 每页结果数，对应 `limit` 参数。
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
        observability::market_search_started(query, page, page_size, "modrinth");
        let offset = (page - 1) * page_size;
        let url = format!(
            "{}/search?query={}&limit={}&offset={}",
            MODRINTH_BASE, urlencoding::encode(query), page_size, offset
        );

        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .header("User-Agent", super::USER_AGENT)
            .send()
            .await
            .map_err(|e| MarketError::http("search resources", e.to_string()))?;

        // 自动反序列化为 ModrinthSearchResponse
        let search_resp: ModrinthSearchResponse = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse search results", e.to_string()))?;

        // 将 Modrinth 的搜索命中转换为内部的 MarketResource
        let resources: Vec<MarketResource> = search_resp
            .hits
            .into_iter()
            .map(|hit| MarketResource {
                id: hit.project_id,
                name: hit.title,
                description: hit.description,
                download_count: hit.downloads,
                version_count: 0,
                source: MarketSource::Modrinth,
            })
            .collect();

        observability::market_search_completed(query, search_resp.total_hits, "modrinth");

        Ok(SearchResult {
            total: search_resp.total_hits,
            offset: search_resp.offset,
            // 使用请求时传入的 page_size，而非响应中的 limit（二者一致）
            limit: page_size as u64,
            resources,
        })
    }

    /// 获取 Modrinth 上指定项目的详细信息。
    ///
    /// 调用 `GET /project/{id}`，通过 `ModrinthProject` 自动反序列化响应。
    /// `game_versions` 和 `loaders` 为数组字段，分别记录支持的游戏版本和
    /// 加载器；`project_type` 标识资源类型（如 `mod`、`plugin`）。
    ///
    /// # Parameters
    /// - `id`: 项目的 ID（格式为字符串，如 `"A1b2C3d4"`）。
    ///
    /// # Returns
    /// 包含项目详细元数据的 `ResourceInfo`。
    async fn get_resource(&self, id: &str) -> Result<ResourceInfo, MarketError> {
        let url = format!("{}/project/{}", MODRINTH_BASE, id);

        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .header("User-Agent", super::USER_AGENT)
            .send()
            .await
            .map_err(|e| MarketError::http("get resource details", e.to_string()))?;

        // 自动反序列化为 ModrinthProject
        let project: ModrinthProject = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse resource details", e.to_string()))?;

        let info = ResourceInfo {
            id: project.id,
            name: project.title,
            description: project.description,
            download_count: project.downloads,
            source: MarketSource::Modrinth,
            icon_url: project.icon_url,
            game_versions: project.game_versions,
            loaders: project.loaders,
            resource_type: project.project_type,
            external: false,
            download_url: String::new(),
        };

        observability::market_resource_fetched(id, &info.name, "modrinth");
        Ok(info)
    }

    /// 获取指定项目的所有版本列表。
    ///
    /// 调用 `GET /project/{id}/version`，通过 `Vec<ModrinthVersion>` 自动
    /// 反序列化响应。每个版本包含 `files`（文件列表，含 URL、文件名、大小、
    /// 主文件标记）、`game_versions`（支持的游戏版本）和 `loaders`（支持的
    /// 加载器）。
    ///
    /// # Parameters
    /// - `id`: 项目的 ID。
    ///
    /// # Returns
    /// 版本对象列表，每个版本包含文件信息和兼容性元数据。
    async fn get_resource_versions(&self, id: &str) -> Result<Vec<Version>, MarketError> {
        let url = format!("{}/project/{}/version", MODRINTH_BASE, id);

        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .header("User-Agent", super::USER_AGENT)
            .send()
            .await
            .map_err(|e| MarketError::http("get resource versions", e.to_string()))?;

        // 自动反序列化为 Vec<ModrinthVersion>
        let versions: Vec<ModrinthVersion> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse version list", e.to_string()))?;

        // 将 Modrinth 版本转换为内部的 Version
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
    /// 委托给 `fetcher::download_file` 执行实际下载，不涉及 Modrinth API 调用。
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
        let status = fetcher::download_file(url, destination).await?;
        Ok(status)
    }

    async fn get_random_resources(&self, count: u32) -> Result<Vec<MarketResource>, MarketError> {
        let limit = count.min(10);
        let url = format!("{}/projects_random?count={}", MODRINTH_BASE, limit);
        let resp = self
            .client
            .get(&url)
            .map_err(|e| MarketError::config(e.to_string()))?
            .header("User-Agent", super::USER_AGENT)
            .send()
            .await
            .map_err(|e| MarketError::http("get random resources", e.to_string()))?;
        let projects: Vec<ModrinthProject> = resp
            .json()
            .await
            .map_err(|e| MarketError::json("parse random resources", e.to_string()))?;
        Ok(projects
            .into_iter()
            .map(|p| MarketResource {
                id: p.id,
                name: p.title,
                description: p.description,
                download_count: p.downloads,
                version_count: 0,
                source: MarketSource::Modrinth,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::market::fetcher::Fetcher;
    use crate::market::models::MarketSource;

    use super::*;

    fn test_fetcher() -> ModrinthFetcher {
        let client = sealantern_infra::net::NetClient::from_config(&Default::default()).unwrap();
        ModrinthFetcher::new(client)
    }

    #[tokio::test]
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
}
