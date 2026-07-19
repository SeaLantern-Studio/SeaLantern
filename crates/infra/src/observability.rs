//! 日志事件。
//!
//! 定义 `infra` 各模块的 tracing 目标（target）和事件名常量，
//! 为日志收集和插件系统提供稳定的事件键。

use std::fmt::Display;

// ── 网络层 ──

/// 网络模块的 tracing 目标。
pub const NET_TARGET: &str = "sealantern.infra.net";

/// 事件：HTTP 请求已开始。
pub const EVENT_REQUEST_STARTED: &str = "request_started";
/// 事件：HTTP 请求已成功完成。
pub const EVENT_REQUEST_COMPLETED: &str = "request_completed";
/// 事件：HTTP 请求正在重试。
pub const EVENT_REQUEST_RETRY: &str = "request_retry";
/// 事件：代理配置无效。
pub const EVENT_PROXY_CONFIG_INVALID: &str = "proxy_config_invalid";

/// 记录代理配置无效事件。
pub fn proxy_config_invalid(proxy_url: &str, error: &dyn Display) {
    tracing::error!(
        target: NET_TARGET,
        event_name = EVENT_PROXY_CONFIG_INVALID,
        proxy_url,
        error = %error,
        "proxy config invalid"
    );
}

/// 记录 HTTP 请求重试事件。
pub fn request_retry(url: &str, attempt: u32, error: &dyn Display) {
    tracing::warn!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_RETRY,
        url,
        attempt,
        error = %error,
        "HTTP request retry"
    );
}

/// 事件：HTTP 请求已失败。
pub const EVENT_REQUEST_FAILED: &str = "request_failed";

/// 记录请求开始事件。
pub fn request_started(url: &str) {
    tracing::info!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_STARTED,
        url,
        "HTTP request started"
    );
}

/// 记录请求完成事件。
pub fn request_completed(url: &str) {
    tracing::info!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_COMPLETED,
        url,
        "HTTP request completed"
    );
}

/// 记录请求失败事件。
pub fn request_failed(url: &str, error: &dyn Display) {
    tracing::error!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_FAILED,
        url,
        error = %error,
        "HTTP request failed"
    );
}

// ── 下载层 ──

/// 下载模块的 tracing 目标。
pub const DOWNLOAD_TARGET: &str = "sealantern.infra.download";

/// 事件：下载已开始。
pub const EVENT_DOWNLOAD_STARTED: &str = "download_started";
/// 事件：下载已完成。
pub const EVENT_DOWNLOAD_COMPLETED: &str = "download_completed";
/// 事件：下载已失败。
pub const EVENT_DOWNLOAD_FAILED: &str = "download_failed";
/// 事件：分片下载已开始。
pub const EVENT_CHUNK_STARTED: &str = "chunk_started";
/// 事件：分片下载已完成。
pub const EVENT_CHUNK_COMPLETED: &str = "chunk_completed";
/// 事件：分片下载已失败。
pub const EVENT_CHUNK_FAILED: &str = "chunk_failed";

/// 记录下载开始事件。
pub fn download_started(url: &str, total_size: u64, thread_count: usize) {
    tracing::info!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_STARTED,
        url,
        total_size,
        thread_count,
        "download started"
    );
}

/// 记录下载完成事件。
pub fn download_completed(url: &str, total_size: u64, elapsed: u64) {
    tracing::info!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_COMPLETED,
        url,
        total_size,
        elapsed_ms = elapsed,
        "download completed"
    );
}

/// 记录下载失败事件。
pub fn download_failed(url: &str, error: &dyn Display) {
    tracing::error!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_FAILED,
        url,
        error = %error,
        "download failed"
    );
}

/// 记录分片开始事件。
pub fn chunk_started(url: &str, start: u64, end: u64) {
    tracing::debug!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_CHUNK_STARTED,
        url,
        range_start = start,
        range_end = end,
        "chunk download started"
    );
}

/// 记录分片完成事件。
pub fn chunk_completed(url: &str, start: u64, end: u64) {
    tracing::debug!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_CHUNK_COMPLETED,
        url,
        range_start = start,
        range_end = end,
        "chunk download completed"
    );
}

/// 记录分片失败事件。
pub fn chunk_failed(url: &str, start: u64, end: u64, error: &dyn Display) {
    tracing::error!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_CHUNK_FAILED,
        url,
        range_start = start,
        range_end = end,
        error = %error,
        "chunk download failed"
    );
}
