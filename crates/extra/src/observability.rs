//! 可观测性模块的日志事件。
//!
//! 为 `extra` 子模块定义 tracing 目标和事件名称常量，
//! 为日志收集和插件系统提供稳定的事件键。

use std::fmt::Display;

/// 应用插件执行内核的 tracing 目标。
pub const APP_PLUGIN_TARGET: &str = "sealantern.extra.app_plugin";

/// Event: 发现的插件因 API 版本过旧被拒绝。
pub const EVENT_APP_PLUGIN_API_TOO_OLD: &str = "app_plugin_api_too_old";
/// Event: 插件已完成脚本加载。
pub const EVENT_APP_PLUGIN_LOADED: &str = "app_plugin_loaded";
/// Event: 插件生命周期回调失败。
pub const EVENT_APP_PLUGIN_LIFECYCLE_FAILED: &str = "app_plugin_lifecycle_failed";
/// Event: 插件私有存储操作失败。
pub const EVENT_APP_PLUGIN_STORAGE_FAILED: &str = "app_plugin_storage_failed";
/// Event: 插件输出日志。
pub const EVENT_APP_PLUGIN_LOG_EMITTED: &str = "app_plugin_log_emitted";
/// Event: 插件加载失败。
pub const EVENT_APP_PLUGIN_LOAD_FAILED: &str = "app_plugin_load_failed";
/// Event: 插件脚本耗尽执行预算。
pub const EVENT_APP_PLUGIN_EXECUTION_LIMIT_EXCEEDED: &str = "app_plugin_execution_limit_exceeded";

/// 记录因不支持的旧 API 而拒绝插件。
pub fn app_plugin_api_too_old(plugin_id: &str, found_api_version: Option<u32>) {
    tracing::warn!(
        target: APP_PLUGIN_TARGET,
        event_name = EVENT_APP_PLUGIN_API_TOO_OLD,
        plugin_id,
        found_api_version,
        "plugin rejected because its API version is too old"
    );
}

/// 记录插件脚本加载完成。
pub fn app_plugin_loaded(plugin_id: &str) {
    tracing::info!(
        target: APP_PLUGIN_TARGET,
        event_name = EVENT_APP_PLUGIN_LOADED,
        plugin_id,
        "plugin script loaded"
    );
}

/// 记录插件生命周期回调失败。
pub fn app_plugin_lifecycle_failed(plugin_id: &str, lifecycle: &str, error_kind: &str) {
    tracing::error!(
        target: APP_PLUGIN_TARGET,
        event_name = EVENT_APP_PLUGIN_LIFECYCLE_FAILED,
        plugin_id,
        lifecycle,
        error_kind,
        "plugin lifecycle callback failed"
    );
}

/// 记录插件私有存储失败。
pub fn app_plugin_storage_failed(plugin_id: &str, operation: &str) {
    tracing::error!(
        target: APP_PLUGIN_TARGET,
        event_name = EVENT_APP_PLUGIN_STORAGE_FAILED,
        plugin_id,
        operation,
        "plugin storage operation failed"
    );
}

/// 记录插件在加载过程中的失败，不记录脚本正文或错误详情。
pub fn app_plugin_load_failed(plugin_id: &str, phase: &'static str, error_kind: &str) {
    tracing::error!(
        target: APP_PLUGIN_TARGET,
        event_name = EVENT_APP_PLUGIN_LOAD_FAILED,
        plugin_id,
        phase,
        error_kind,
        "plugin load failed"
    );
}

/// 记录插件脚本耗尽执行预算。
pub fn app_plugin_execution_limit_exceeded(plugin_id: &str, operation: &'static str) {
    tracing::warn!(
        target: APP_PLUGIN_TARGET,
        event_name = EVENT_APP_PLUGIN_EXECUTION_LIMIT_EXCEEDED,
        plugin_id,
        operation,
        "plugin execution limit exceeded"
    );
}

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

/// 在线隧道模块的 tracing 目标。
pub const ONLINE_TARGET: &str = "sealantern.extra.online";

/// Event: 在线隧道已启动。
pub const EVENT_ONLINE_TUNNEL_STARTED: &str = "online_tunnel_started";
/// Event: 在线隧道已停止。
pub const EVENT_ONLINE_TUNNEL_STOPPED: &str = "online_tunnel_stopped";
/// Event: 在线隧道操作失败。
pub const EVENT_ONLINE_TUNNEL_FAILED: &str = "online_tunnel_failed";
/// Event: 在线隧道报告非致命错误。
pub const EVENT_ONLINE_TUNNEL_EVENT_ERROR: &str = "online_tunnel_event_error";

/// 记录在线隧道启动完成，不记录票据、密码或身份密钥。
pub fn online_tunnel_started(mode: &str) {
    tracing::info!(
        target: ONLINE_TARGET,
        event_name = EVENT_ONLINE_TUNNEL_STARTED,
        mode,
        "online tunnel started"
    );
}

/// 记录在线隧道停止完成。
pub fn online_tunnel_stopped(mode: &str) {
    tracing::info!(
        target: ONLINE_TARGET,
        event_name = EVENT_ONLINE_TUNNEL_STOPPED,
        mode,
        "online tunnel stopped"
    );
}

/// 记录在线隧道操作失败，不记录调用输入中的敏感字段。
pub fn online_tunnel_failed(operation: &str, error: &dyn Display) {
    tracing::error!(
        target: ONLINE_TARGET,
        event_name = EVENT_ONLINE_TUNNEL_FAILED,
        operation,
        error = %error,
        "online tunnel operation failed"
    );
}

/// 记录底层隧道报告的非致命错误事件。
pub fn online_tunnel_event_error(error: &str) {
    tracing::warn!(
        target: ONLINE_TARGET,
        event_name = EVENT_ONLINE_TUNNEL_EVENT_ERROR,
        error,
        "online tunnel reported a non-fatal error"
    );
}

// ---------------------------------------------------------------------------
// 更新检查模块
// ---------------------------------------------------------------------------

/// 更新检查模块的 tracing 目标。
pub const UPDATE_TARGET: &str = "sealantern.extra.update";

/// Event: 更新检查开始。
pub const EVENT_UPDATE_CHECK_STARTED: &str = "update_check_started";
/// Event: 更新检查完成。
pub const EVENT_UPDATE_CHECK_COMPLETED: &str = "update_check_completed";
/// Event: 更新下载开始。
pub const EVENT_UPDATE_DOWNLOAD_STARTED: &str = "update_download_started";
/// Event: 更新下载完成。
pub const EVENT_UPDATE_DOWNLOAD_COMPLETED: &str = "update_download_completed";
/// Event: 更新下载失败。
pub const EVENT_UPDATE_DOWNLOAD_FAILED: &str = "update_download_failed";
/// Event: 更新校验和验证通过。
pub const EVENT_UPDATE_HASH_VERIFIED: &str = "update_hash_verified";
/// Event: 更新校验和不匹配。
pub const EVENT_UPDATE_HASH_MISMATCH: &str = "update_hash_mismatch";
/// Event: 更新 API 请求失败。
pub const EVENT_UPDATE_API_REQUEST_FAILED: &str = "update_api_request_failed";

/// 记录更新检查开始。
pub fn update_check_started(source: &str, current_version: &str) {
    tracing::info!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_CHECK_STARTED,
        source,
        current_version,
        "update check started"
    );
}

/// 记录更新检查完成。
pub fn update_check_completed(source: &str, has_update: bool, latest_version: Option<&str>) {
    tracing::info!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_CHECK_COMPLETED,
        source,
        has_update,
        latest_version,
        "update check completed"
    );
}

/// 记录更新下载开始。
pub fn update_download_started(url: &str) {
    tracing::info!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_DOWNLOAD_STARTED,
        url,
        "update download started"
    );
}

/// 记录更新下载完成。
pub fn update_download_completed(file_path: &str) {
    tracing::info!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_DOWNLOAD_COMPLETED,
        file_path,
        "update download completed"
    );
}

/// 记录更新下载失败。
pub fn update_download_failed(url: &str, error: &dyn Display) {
    tracing::error!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_DOWNLOAD_FAILED,
        url,
        error = %error,
        "update download failed"
    );
}

/// 记录更新校验和验证通过。
pub fn update_hash_verified(file_path: &str) {
    tracing::info!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_HASH_VERIFIED,
        file_path,
        "update hash verified"
    );
}

/// 记录更新校验和不匹配——文件可能损坏或被篡改。
pub fn update_hash_mismatch(file_path: &str, expected: &str, got: &str) {
    tracing::error!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_HASH_MISMATCH,
        file_path,
        expected,
        got,
        "update hash mismatch"
    );
}

/// 记录更新 API 请求失败。
pub fn update_api_request_failed(
    source: &str,
    operation: &str,
    status: Option<u16>,
    error: &dyn Display,
) {
    tracing::error!(
        target: UPDATE_TARGET,
        event_name = EVENT_UPDATE_API_REQUEST_FAILED,
        source,
        operation,
        status,
        error = %error,
        "update API request failed"
    );
}
