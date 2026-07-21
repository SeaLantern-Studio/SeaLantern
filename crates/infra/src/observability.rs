//! Log events for the observability module.
//!
//! Defines tracing targets and event name constants for `infra` sub-modules,
//! providing stable event keys for log collection and the plugin system.

use std::fmt::Display;

// -- File system layer --

/// Tracing target for the file system infrastructure module.
pub const FS_TARGET: &str = "sealantern.infra.fs";

/// Event: an atomic file replacement failed.
pub const EVENT_ATOMIC_WRITE_FAILED: &str = "atomic_write_failed";
/// Event: a file lock could not be released.
pub const EVENT_LOCK_RELEASE_FAILED: &str = "lock_release_failed";
/// Event: a file system operation failed.
pub const EVENT_OPERATION_FAILED: &str = "operation_failed";
/// Event: a cache operation failed.
pub const EVENT_CACHE_OPERATION_FAILED: &str = "cache_operation_failed";
/// Event: structured file data could not be encoded or decoded.
pub const EVENT_SERIALIZATION_FAILED: &str = "serialization_failed";
/// Event: a file lock could not be acquired.
pub const EVENT_LOCK_ACQUIRE_FAILED: &str = "lock_acquire_failed";

// -- Archive layer --

/// Tracing target for archive infrastructure operations.
pub const ARCHIVE_TARGET: &str = "sealantern.infra.archive";

/// Event: a ZIP archive operation failed.
pub const EVENT_ARCHIVE_OPERATION_FAILED: &str = "archive_operation_failed";

/// Records an archive creation or extraction failure.
pub fn archive_operation_failed(operation: &str, archive: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: ARCHIVE_TARGET,
        event_name = EVENT_ARCHIVE_OPERATION_FAILED,
        operation,
        archive = %archive.display(),
        error = %error,
        "archive operation failed"
    );
}

/// Records an atomic write failure with its destination path.
pub fn atomic_write_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: FS_TARGET,
        event_name = EVENT_ATOMIC_WRITE_FAILED,
        path = %path.display(),
        error = %error,
        "atomic file write failed"
    );
}

/// Records a best-effort lock cleanup failure.
pub fn lock_release_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: FS_TARGET,
        event_name = EVENT_LOCK_RELEASE_FAILED,
        path = %path.display(),
        error = %error,
        "file lock release failed"
    );
}

/// Records a failed file system operation.
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

/// Records a failed cache operation.
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

/// Records a structured file serialization failure.
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

/// Records an unsuccessful file lock acquisition.
pub fn lock_acquire_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: FS_TARGET,
        event_name = EVENT_LOCK_ACQUIRE_FAILED,
        path = %path.display(),
        error = %error,
        "file lock acquisition failed"
    );
}

// ── Network layer ──

/// Tracing target for the network module.
pub const NET_TARGET: &str = "sealantern.infra.net";

/// Event: an HTTP request has started.
pub const EVENT_REQUEST_STARTED: &str = "request_started";
/// Event: an HTTP request has completed successfully.
pub const EVENT_REQUEST_COMPLETED: &str = "request_completed";
/// Event: an HTTP request is being retried.
pub const EVENT_REQUEST_RETRY: &str = "request_retry";
/// Event: the proxy configuration is invalid.
pub const EVENT_PROXY_CONFIG_INVALID: &str = "proxy_config_invalid";

/// Records a proxy configuration invalid event.
pub fn proxy_config_invalid(proxy_url: &str, error: &dyn Display) {
    tracing::error!(
        target: NET_TARGET,
        event_name = EVENT_PROXY_CONFIG_INVALID,
        proxy_url,
        error = %error,
        "proxy config invalid"
    );
}

/// Records an HTTP request retry event.
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

/// Event: an HTTP request has failed.
pub const EVENT_REQUEST_FAILED: &str = "request_failed";

/// Records a request start event.
pub fn request_started(url: &str) {
    tracing::info!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_STARTED,
        url,
        "HTTP request started"
    );
}

/// Records a request completion event.
pub fn request_completed(url: &str) {
    tracing::info!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_COMPLETED,
        url,
        "HTTP request completed"
    );
}

/// Records a request failure event.
pub fn request_failed(url: &str, error: &dyn Display) {
    tracing::error!(
        target: NET_TARGET,
        event_name = EVENT_REQUEST_FAILED,
        url,
        error = %error,
        "HTTP request failed"
    );
}

// ── Download layer ──

/// Tracing target for the download module.
pub const DOWNLOAD_TARGET: &str = "sealantern.infra.download";

/// Event: download has started.
pub const EVENT_DOWNLOAD_STARTED: &str = "download_started";
/// Event: download has completed.
pub const EVENT_DOWNLOAD_COMPLETED: &str = "download_completed";
/// Event: download has failed.
pub const EVENT_DOWNLOAD_FAILED: &str = "download_failed";
/// Event: chunk download has started.
pub const EVENT_CHUNK_STARTED: &str = "chunk_started";
/// Event: chunk download has completed.
pub const EVENT_CHUNK_COMPLETED: &str = "chunk_completed";
/// Event: chunk download has failed.
pub const EVENT_CHUNK_FAILED: &str = "chunk_failed";

/// Records a download start event.
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

/// Records a download completion event.
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

/// Records a download failure event.
pub fn download_failed(url: &str, error: &dyn Display) {
    tracing::error!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_FAILED,
        url,
        error = %error,
        "download failed"
    );
}

/// Records a chunk start event.
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

/// Records a chunk completion event.
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

/// Records a chunk failure event.
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

/// Event: a download task has been created.
pub const EVENT_TASK_CREATED: &str = "task_created";
/// Event: a download task has been cancelled.
pub const EVENT_TASK_CANCELLED: &str = "task_cancelled";
/// Event: download cancelled by user.
pub const EVENT_DOWNLOAD_CANCELLED: &str = "download_cancelled";
/// Event: download encountered an error.
pub const EVENT_DOWNLOAD_ERROR: &str = "download_error";

/// Records a task creation event.
pub fn task_created(task_id: &uuid::Uuid, url: &str) {
    tracing::info!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_TASK_CREATED,
        task_id = %task_id,
        url,
        "download task created"
    );
}

/// Records a task cancellation event.
pub fn task_cancelled(task_id: &uuid::Uuid) {
    tracing::warn!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_TASK_CANCELLED,
        task_id = %task_id,
        "download task cancelled"
    );
}

/// Records a download cancelled event.
pub fn download_cancelled() {
    tracing::warn!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_CANCELLED,
        "download cancelled by user"
    );
}

/// Records a download error event.
pub fn download_error(error: &dyn Display) {
    tracing::error!(
        target: DOWNLOAD_TARGET,
        event_name = EVENT_DOWNLOAD_ERROR,
        error = %error,
        "download error"
    );
}
