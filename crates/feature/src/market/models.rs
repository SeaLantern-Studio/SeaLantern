//! 市场数据模型定义。
//!
//! 本模块定义了与远程市场 API 交互所使用的核心数据结构，
//! 包括资源信息、项目详情、版本信息、搜索结果等。
//! 所有模型均实现了 [`Serialize`] 和 [`Deserialize`] trait，
//! 支持与 JSON 格式之间的相互转换。

use serde::{Deserialize, Serialize};

use crate::market::fetcher::models::VersionFile;

/// 市场资源的基本信息。
///
/// 代表在市场上（Spiget / Modrinth）发布的单个资源（插件或模组）
/// 的概要信息，通常用于资源列表或搜索结果中展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResource {
    /// 资源在市场上的唯一标识符。
    pub id: String,

    /// 资源的显示名称。
    pub name: String,

    /// 资源的简要描述文本。
    pub description: String,

    /// 资源的累计下载次数。
    pub download_count: u64,

    /// 资源已发布的版本数量。
    pub version_count: u64,

    /// 资源来源市场（Spiget 或 Modrinth）。
    pub source: MarketSource,
}

/// 市场来源枚举。
///
/// 标识资源来自哪个市场平台，用于区分不同的 API 调用策略
/// 和数据格式处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketSource {
    /// 资源来自 Spiget（SpigotMC 社区市场）。
    Spiget,

    /// 资源来自 Modrinth（模组与插件分发平台）。
    Modrinth,
}

/// 资源的详细项目信息。
///
/// 相较于 [`MarketResource`]，本结构体包含更丰富的资源元数据，
/// 如图标 URL、支持的游戏版本、加载器类型等。
/// 通常在查看资源详情时使用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// 资源在市场上的唯一标识符。
    pub id: String,

    /// 资源的显示名称。
    pub name: String,

    /// 资源的详细描述文本（可能包含 Markdown 格式）。
    pub description: String,

    /// 资源的累计下载次数。
    pub download_count: u64,

    /// 资源来源市场（Spiget 或 Modrinth）。
    pub source: MarketSource,

    /// 资源的图标 URL，若资源未设置图标则为 `None`。
    pub icon_url: Option<String>,

    /// 该资源支持的游戏版本列表（如 `["1.20", "1.21"]`）。
    pub game_versions: Vec<String>,

    /// 该资源支持的加载器/平台列表（如 `["bukkit", "paper"]` 或 `["fabric", "forge"]`）。
    pub loaders: Vec<String>,

    /// 资源类型标识（如 `"plugin"`、`"mod"`、`"datapack"` 等）。
    pub resource_type: String,

    /// 资源是否为外部托管（仅 Spiget 适用）。
    ///
    /// 为 `true` 时，`download_url` 指向外部站点（如 GitHub、Modrinth）；
    /// 为 `false` 时，`download_url` 为 Spiget CDN 下载路径。
    pub external: bool,

    /// 资源的下载链接。
    ///
    /// 对于 Spiget，此为完整的 CDN 下载 URL 或外部 URL；
    /// 对于 Modrinth，此为项目的 CDN 文件下载链接。
    pub download_url: String,
}

/// 资源的特定版本信息。
///
/// 代表某个资源下发布的单个版本，包含版本号、支持的游戏版本、
/// 加载器信息以及可下载的文件列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// 版本在市场上的唯一标识符。
    pub id: String,

    /// 版本的显示名称（可能是语义化版本号或自定义名称）。
    pub name: String,

    /// 版本号字符串（如 `"1.0.0"`、`"v2.3-beta"`）。
    pub version_number: String,

    /// 该版本兼容的游戏版本列表（如 `["1.20.1", "1.20.2"]`）。
    pub game_versions: Vec<String>,

    /// 该版本支持的加载器/平台列表。
    pub loaders: Vec<String>,

    /// 该版本的累计下载次数。
    pub downloads: u64,

    /// 该版本关联的可下载文件列表。
    pub files: Vec<VersionFile>,
}

/// 搜索结果分页数据。
///
/// 封装了搜索 API 的返回结果，包含匹配的资源列表
/// 以及分页相关的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 当前页面包含的资源列表。
    pub resources: Vec<MarketResource>,

    /// 匹配搜索条件的资源总数（用于分页计算）。
    pub total: u64,

    /// 当前结果的起始偏移量（从 0 开始）。
    pub offset: u64,

    /// 每页返回的最大资源数量。
    pub limit: u64,
}
