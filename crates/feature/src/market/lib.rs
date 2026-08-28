//! 资源市场（Market）模块。
//!
//! 本模块提供了与 Minecraft 插件/模组资源市场交互的能力，
//! 支持从 Spiget 和 Modrinth 等源获取资源信息、搜索资源、
//! 查询版本详情及下载资源文件等功能。
//!
//! 模块包含以下子模块：
//! - [`error`]：市场操作相关的错误类型定义。
//! - [`fetcher`]：与远程市场 API 交互的抓取器实现。
//! - [`models`]：资源、版本、搜索结果等数据模型定义。

pub mod error;
pub mod fetcher;
pub mod models;

pub use error::MarketError;
pub use fetcher::{Fetcher, ModrinthFetcher, SpigetFetcher, VersionFile};
pub use models::{MarketResource, MarketSource, ResourceInfo, SearchResult, Version};
