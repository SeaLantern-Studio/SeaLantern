//! 可观测性模块的日志事件。
//!
//! 为 `extra` 子模块定义 tracing 目标和事件名称常量，
//! 为日志收集和插件系统提供稳定的事件键。

use std::fmt::Display;

/// 市场模块的 tracing 目标。
pub const MARKET_TARGET: &str = "sealantern.extra.market";

/// Event: 搜索开始。
pub const EVENT_MARKET_SEARCH_STARTED: &str = "market_search_started";
/// Event: 搜索完成。
pub const EVENT_MARKET_SEARCH_COMPLETED: &str = "market_search_completed";
/// Event: 资源详情获取成功。
pub const EVENT_MARKET_RESOURCE_FETCHED: &str = "market_resource_fetched";
/// Event: 版本列表获取成功。
pub const EVENT_MARKET_VERSIONS_FETCHED: &str = "market_versions_fetched";
/// Event: 下载开始。
pub const EVENT_MARKET_DOWNLOAD_STARTED: &str = "market_download_started";
/// Event: 市场 API 请求失败。
pub const EVENT_MARKET_REQUEST_FAILED: &str = "market_request_failed";

/// 记录搜索开始事件。
pub fn market_search_started(query: &str, page: u32, page_size: u32, source: &str) {
    tracing::info!(
        target: MARKET_TARGET,
        event_name = EVENT_MARKET_SEARCH_STARTED,
        query,
        page,
        page_size,
        source,
        "market search started"
    );
}

/// 记录搜索完成事件。
pub fn market_search_completed(query: &str, total: u64, source: &str) {
    tracing::info!(
        target: MARKET_TARGET,
        event_name = EVENT_MARKET_SEARCH_COMPLETED,
        query,
        total,
        source,
        "market search completed"
    );
}

/// 记录资源详情获取成功事件。
pub fn market_resource_fetched(id: &str, name: &str, source: &str) {
    tracing::info!(
        target: MARKET_TARGET,
        event_name = EVENT_MARKET_RESOURCE_FETCHED,
        id,
        name,
        source,
        "market resource fetched"
    );
}

/// 记录版本列表获取成功事件。
pub fn market_versions_fetched(id: &str, count: usize, source: &str) {
    tracing::info!(
        target: MARKET_TARGET,
        event_name = EVENT_MARKET_VERSIONS_FETCHED,
        id,
        count,
        source,
        "market versions fetched"
    );
}

/// 记录下载开始事件。
pub fn market_download_started(url: &str, source: &str) {
    tracing::info!(
        target: MARKET_TARGET,
        event_name = EVENT_MARKET_DOWNLOAD_STARTED,
        url,
        source,
        "market download started"
    );
}

/// 记录市场 API 请求失败事件。
pub fn market_request_failed(operation: &str, source: &str, error: &dyn Display) {
    tracing::error!(
        target: MARKET_TARGET,
        event_name = EVENT_MARKET_REQUEST_FAILED,
        operation,
        source,
        error = %error,
        "market request failed"
    );
}
