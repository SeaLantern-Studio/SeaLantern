//! 可观测性模块的日志事件。
//!
//! 为 `infra` 子模块定义 tracing 目标和事件名称常量，
//! 为日志收集和插件系统提供稳定的事件键。

use std::fmt::Display;

// ── 持久化层 ──

/// 持久化基础设施模块的 tracing 目标。
pub const PERSISTENCE_TARGET: &str = "sealantern.infra.persistence";

/// Event: 持久化操作失败。
pub const EVENT_PERSISTENCE_OPERATION_FAILED: &str = "persistence_operation_failed";

/// 记录持久化操作失败，不记录 SQL 文本或参数值。
pub fn persistence_operation_failed(operation: &str, path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: PERSISTENCE_TARGET,
        event_name = EVENT_PERSISTENCE_OPERATION_FAILED,
        operation,
        path = %path.display(),
        error = %error,
        "persistence operation failed"
    );
}

// -- 平台层 --

/// 平台基础设施模块的 tracing 目标。
pub const PLATFORM_TARGET: &str = "sealantern.infra.platform";

/// Event: 平台操作失败。
pub const EVENT_PLATFORM_OPERATION_FAILED: &str = "platform_operation_failed";

/// 记录平台操作失败，不记录环境变量值或证书内容。
pub fn platform_operation_failed(operation: &str, error: &dyn Display) {
    tracing::error!(
        target: PLATFORM_TARGET,
        event_name = EVENT_PLATFORM_OPERATION_FAILED,
        operation,
        error = %error,
        "platform operation failed"
    );
}

// -- 文件系统层 --

/// 文件系统基础设施模块的 tracing 目标。
pub const FS_TARGET: &str = "sealantern.infra.fs";

/// Event: 原子文件替换失败。
pub const EVENT_ATOMIC_WRITE_FAILED: &str = "atomic_write_failed";
/// Event: 文件锁无法释放。
pub const EVENT_LOCK_RELEASE_FAILED: &str = "lock_release_failed";
/// Event: 文件系统操作失败。
pub const EVENT_OPERATION_FAILED: &str = "operation_failed";
/// Event: 缓存操作失败。
pub const EVENT_CACHE_OPERATION_FAILED: &str = "cache_operation_failed";
/// Event: 结构化文件数据无法编码或解码。
pub const EVENT_SERIALIZATION_FAILED: &str = "serialization_failed";
/// Event: 无法获取文件锁。
pub const EVENT_LOCK_ACQUIRE_FAILED: &str = "lock_acquire_failed";

// -- 归档层 --

/// 归档基础设施操作的 tracing 目标。
pub const ARCHIVE_TARGET: &str = "sealantern.infra.archive";

/// Event: ZIP 归档操作失败。
pub const EVENT_ARCHIVE_OPERATION_FAILED: &str = "archive_operation_failed";
/// Event: 发布后无法移除临时归档文件。
pub const EVENT_ARCHIVE_CLEANUP_FAILED: &str = "archive_cleanup_failed";

/// 记录归档创建或提取失败。
pub fn archive_operation_failed(operation: &str, archive: &std::path::Path, error: &dyn Display) {
    archive_operation_failed_with_context(operation, archive, None, None, error);
}

/// 记录归档失败，包含受影响的目标路径和条目（如有）。
pub fn archive_operation_failed_with_context(
    operation: &str,
    archive: &std::path::Path,
    destination: Option<&std::path::Path>,
    entry: Option<&str>,
    error: &dyn Display,
) {
    tracing::error!(
        target: ARCHIVE_TARGET,
        event_name = EVENT_ARCHIVE_OPERATION_FAILED,
        operation,
        archive = %archive.display(),
        destination = destination.map(|path| path.display().to_string()),
        entry,
        error = %error,
        "archive operation failed"
    );
}

/// 记录归档成功发布后，尽力清理临时文件失败的情况。
pub fn archive_cleanup_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: ARCHIVE_TARGET,
        event_name = EVENT_ARCHIVE_CLEANUP_FAILED,
        path = %path.display(),
        error = %error,
        "temporary archive cleanup failed"
    );
}

/// 记录原子写入失败及其目标路径。
pub fn atomic_write_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: FS_TARGET,
        event_name = EVENT_ATOMIC_WRITE_FAILED,
        path = %path.display(),
        error = %error,
        "atomic file write failed"
    );
}

/// 记录尽力而为的锁清理失败。
pub fn lock_release_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: FS_TARGET,
        event_name = EVENT_LOCK_RELEASE_FAILED,
        path = %path.display(),
        error = %error,
        "file lock release failed"
    );
}

/// 记录失败的文件系统操作。
pub fn operation_failed(operation: &str, path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: FS_TARGET,
        event_name = EVENT_OPERATION_FAILED,
        operation,
        path = %path.display(),
        error = %error,
        "file system operation failed"
    );
}

/// 记录失败的缓存操作。
pub fn cache_operation_failed(operation: &str, path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: FS_TARGET,
        event_name = EVENT_CACHE_OPERATION_FAILED,
        operation,
        path = %path.display(),
        error = %error,
        "cache operation failed"
    );
}

/// 记录结构化文件序列化失败。
pub fn serialization_failed(
    format: &str,
    operation: &str,
    path: &std::path::Path,
    error: &dyn Display,
) {
    tracing::error!(
        target: FS_TARGET,
        event_name = EVENT_SERIALIZATION_FAILED,
        format,
        operation,
        path = %path.display(),
        error = %error,
        "structured file serialization failed"
    );
}

/// 记录文件锁获取失败。
pub fn lock_acquire_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: FS_TARGET,
        event_name = EVENT_LOCK_ACQUIRE_FAILED,
        path = %path.display(),
        error = %error,
        "file lock acquisition failed"
    );
}

// ── 网络层 ──

/// 网络模块的 tracing 目标。
pub const NET_TARGET: &str = "sealantern.infra.net";

/// Event: HTTP 请求已开始。
pub const EVENT_REQUEST_STARTED: &str = "request_started";
/// Event: HTTP 请求已成功完成。
pub const EVENT_REQUEST_COMPLETED: &str = "request_completed";
/// Event: HTTP 请求正在重试。
pub const EVENT_REQUEST_RETRY: &str = "request_retry";
/// Event: 代理配置无效。
pub const EVENT_PROXY_CONFIG_INVALID: &str = "proxy_config_invalid";
/// Event: 无法应用代理设置。
pub const EVENT_PROXY_SETTINGS_INVALID: &str = "proxy_settings_invalid";
/// Event: 代理路由策略已评估。
pub const EVENT_PROXY_DECISION_UPDATED: &str = "proxy_decision_updated";
/// Event: 平台代理发现失败。
pub const EVENT_SYSTEM_PROXY_READ_FAILED: &str = "system_proxy_read_failed";
/// Event: 无法构建 HTTP 客户端。
pub const EVENT_HTTP_CLIENT_BUILD_FAILED: &str = "http_client_build_failed";

/// 记录代理配置无效事件。
pub fn proxy_config_invalid(scope: &str) {
    tracing::error!(
        target: NET_TARGET,
        event_name = EVENT_PROXY_CONFIG_INVALID,
        scope,
        "proxy config invalid"
    );
}

/// 记录在客户端构建前被拒绝的无效代理设置。
pub fn proxy_settings_invalid(error: &dyn Display) {
    tracing::warn!(
        target: NET_TARGET,
        event_name = EVENT_PROXY_SETTINGS_INVALID,
        error = %error,
        "proxy settings invalid"
    );
}

/// 记录已评估的代理决策，不暴露端点凭据。
pub fn proxy_decision_updated(source: &str, mode: &str, changed: bool) {
    tracing::info!(
        target: NET_TARGET,
        event_name = EVENT_PROXY_DECISION_UPDATED,
        source,
        mode,
        changed,
        "proxy decision updated"
    );
}

/// 记录平台代理读取失败，不记录供应商控制的文本。
pub fn system_proxy_read_failed() {
    tracing::warn!(
        target: NET_TARGET,
        event_name = EVENT_SYSTEM_PROXY_READ_FAILED,
        "system proxy read failed"
    );
}

/// 记录 HTTP 客户端构建失败，不记录配置值。
pub fn http_client_build_failed() {
    tracing::error!(
        target: NET_TARGET,
        event_name = EVENT_HTTP_CLIENT_BUILD_FAILED,
        "HTTP client build failed"
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

/// Event: HTTP 请求已失败。
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

/// Event: 下载已开始。
pub const EVENT_DOWNLOAD_STARTED: &str = "download_started";
/// Event: 下载已完成。
pub const EVENT_DOWNLOAD_COMPLETED: &str = "download_completed";
/// Event: 下载已失败。
pub const EVENT_DOWNLOAD_FAILED: &str = "download_failed";
/// Event: 分块下载已开始。
pub const EVENT_CHUNK_STARTED: &str = "chunk_started";
/// Event: 分块下载已完成。
pub const EVENT_CHUNK_COMPLETED: &str = "chunk_completed";
/// Event: 分块下载已失败。
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

/// 记录分块开始事件。
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

/// 记录分块完成事件。
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

/// 记录分块失败事件。
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

/// Event: 下载任务已创建。
pub const EVENT_TASK_CREATED: &str = "task_created";
/// Event: 下载任务已取消。
pub const EVENT_TASK_CANCELLED: &str = "task_cancelled";
/// Event: 下载已被用户取消。
pub const EVENT_DOWNLOAD_CANCELLED: &str = "download_cancelled";
/// Event: 下载遇到错误。
pub const EVENT_DOWNLOAD_ERROR: &str = "download_error";

/// 记录任务创建事件。
pub fn task_created(task_id: &uuid::Uuid, url: &str) {
    tracing::info!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_TASK_CREATED,
        task_id = %task_id,
        url,
        "download task created"
    );
}

/// 记录任务取消事件。
pub fn task_cancelled(task_id: &uuid::Uuid) {
    tracing::warn!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_TASK_CANCELLED,
        task_id = %task_id,
        "download task cancelled"
    );
}

/// 记录下载取消事件。
pub fn download_cancelled() {
    tracing::warn!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_CANCELLED,
        "download cancelled by user"
    );
}

/// 记录下载错误事件。
pub fn download_error(error: &dyn Display) {
    tracing::error!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_ERROR,
        error = %error,
        "download error"
    );
}
