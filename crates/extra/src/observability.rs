//! 可观测性模块的日志事件。
//!
//! 为 `extra` 子模块定义 tracing 目标和事件名称常量，
//! 为日志收集和插件系统提供稳定的事件键。

use std::fmt::Display;

use crate::config::sealantern::types::SettingsGroup;

/// 服务器定时任务模块的 tracing 目标。
pub const SERVER_CRON_TASK_TARGET: &str = "sealantern.extra.server.cron_task";

/// Event: 定时任务开始执行。
pub const EVENT_SERVER_CRON_TASK_STARTED: &str = "server_cron_task_started";
/// Event: 定时任务执行成功。
pub const EVENT_SERVER_CRON_TASK_COMPLETED: &str = "server_cron_task_completed";
/// Event: 定时任务执行失败。
pub const EVENT_SERVER_CRON_TASK_FAILED: &str = "server_cron_task_failed";

pub(crate) fn server_cron_task_started(task_id: &str, server_id: &str, action: &str) {
    tracing::info!(
        target: SERVER_CRON_TASK_TARGET,
        event_name = EVENT_SERVER_CRON_TASK_STARTED,
        task_id,
        server_id,
        action,
        "server cron task started"
    );
}

pub(crate) fn server_cron_task_completed(task_id: &str, server_id: &str, action: &str) {
    tracing::info!(
        target: SERVER_CRON_TASK_TARGET,
        event_name = EVENT_SERVER_CRON_TASK_COMPLETED,
        task_id,
        server_id,
        action,
        "server cron task completed"
    );
}

pub(crate) fn server_cron_task_failed(
    task_id: &str,
    server_id: &str,
    action: &str,
    error: &dyn Display,
) {
    tracing::error!(
        target: SERVER_CRON_TASK_TARGET,
        event_name = EVENT_SERVER_CRON_TASK_FAILED,
        task_id,
        server_id,
        action,
        error = %error,
        "server cron task failed"
    );
}

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

// ---------------------------------------------------------------------------
// Java 环境检测模块
// ---------------------------------------------------------------------------

/// Java 环境检测模块的 tracing 目标。
pub const JAVA_TARGET: &str = "sealantern.extra.java";

/// Event: Java 检测开始。
pub const EVENT_JAVA_DETECTION_STARTED: &str = "java_detection_started";
/// Event: Java 检测完成。
pub const EVENT_JAVA_DETECTION_COMPLETED: &str = "java_detection_completed";
/// Event: Java 检测来源不可用。
pub const EVENT_JAVA_SEARCH_FAILED: &str = "java_search_failed";

/// Event: Java 检测来源完成。
pub const EVENT_JAVA_SEARCH_COMPLETED: &str = "java_search_completed";
/// Event: Java 候选被拒绝。
pub const EVENT_JAVA_CANDIDATE_REJECTED: &str = "java_candidate_rejected";
/// Event: Java 检测因深度来源不可用而回退。
pub const EVENT_JAVA_SEARCH_FALLBACK: &str = "java_search_fallback";
/// Event: Java 校验开始。
pub const EVENT_JAVA_VALIDATION_STARTED: &str = "java_validation_started";
/// Event: Java 校验完成。
pub const EVENT_JAVA_VALIDATION_COMPLETED: &str = "java_validation_completed";
/// Event: Java 校验失败。
pub const EVENT_JAVA_VALIDATION_FAILED: &str = "java_validation_failed";
/// Event: Java 全局搜索开始。
pub const EVENT_JAVA_GLOBAL_SEARCH_STARTED: &str = "java_global_search_started";
/// Event: Java 全局搜索完成。
pub const EVENT_JAVA_GLOBAL_SEARCH_COMPLETED: &str = "java_global_search_completed";

/// 记录一个 Java 检测来源失败；其它来源仍可继续返回结果。
pub fn java_search_failed(source: &str, error: &dyn Display) {
    tracing::warn!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_SEARCH_FAILED,
        source,
        error = %error,
        "java discovery source failed"
    );
}

/// 记录 Java 检测开始。
pub fn java_detection_started() {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_DETECTION_STARTED,
        "java detection started"
    );
}

/// 记录 Java 检测完成。
pub fn java_detection_completed(installation_count: usize, error_count: usize) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_DETECTION_COMPLETED,
        installation_count,
        error_count,
        "java detection completed"
    );
}

/// 记录 Java 检测来源完成。
pub fn java_search_completed(source: &str, installation_count: usize, error_count: usize) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_SEARCH_COMPLETED,
        source,
        installation_count,
        error_count,
        "java discovery source completed"
    );
}

/// 记录 Java 候选因元数据或路径错误被拒绝。
pub fn java_candidate_rejected(source: &str, path: &std::path::Path, error: &dyn Display) {
    tracing::warn!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_CANDIDATE_REJECTED,
        source,
        path = %path.display(),
        error = %error,
        "java discovery candidate rejected"
    );
}

/// 记录 Java 检测回退到另一个来源。
pub fn java_search_fallback(from: &str, to: &str) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_SEARCH_FALLBACK,
        from,
        to,
        "java discovery falling back to another source"
    );
}

/// 记录显式 Java 路径校验开始。
pub fn java_validation_started(path: &str) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_VALIDATION_STARTED,
        path,
        "java validation started"
    );
}

/// 记录显式 Java 路径校验完成。
pub fn java_validation_completed(path: &str, major_version: u32, confidence: u8) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_VALIDATION_COMPLETED,
        path,
        major_version,
        confidence,
        "java validation completed"
    );
}

/// 记录显式 Java 路径校验失败。
pub fn java_validation_failed(path: &str, error: &dyn Display) {
    tracing::warn!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_VALIDATION_FAILED,
        path,
        error = %error,
        "java validation failed"
    );
}

/// 记录 Java 全局搜索开始。
pub fn java_global_search_started(index_reused: bool, complete: bool) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_GLOBAL_SEARCH_STARTED,
        index_reused,
        complete,
        "java global search started"
    );
}

/// 记录 Java 全局搜索完成和索引规模。
pub fn java_global_search_completed(
    installation_count: usize,
    error_count: usize,
    indexed_directory_count: usize,
    indexed_candidate_count: usize,
) {
    tracing::info!(
        target: JAVA_TARGET,
        event_name = EVENT_JAVA_GLOBAL_SEARCH_COMPLETED,
        installation_count,
        error_count,
        indexed_directory_count,
        indexed_candidate_count,
        "java global search completed"
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

// ---------------------------------------------------------------------------
// 配置迁移模块
// ---------------------------------------------------------------------------

/// 配置迁移模块的 tracing 目标。
pub const CONFIG_TARGET: &str = "sealantern.extra.config";

/// Event: 配置迁移开始。
pub const EVENT_CONFIG_MIGRATION_STARTED: &str = "config_migration_started";
/// Event: 配置迁移完成。
pub const EVENT_CONFIG_MIGRATION_COMPLETED: &str = "config_migration_completed";
/// Event: 配置迁移失败。
pub const EVENT_CONFIG_MIGRATION_FAILED: &str = "config_migration_failed";
/// Event: 定位器文件不可读。
pub const EVENT_CONFIG_LOCATOR_UNREADABLE: &str = "config_locator_unreadable";
/// Event: 删除定位器文件失败。
pub const EVENT_CONFIG_LOCATOR_CLEANUP_FAILED: &str = "config_locator_cleanup_failed";

/// 记录配置迁移开始。
pub fn config_migration_started(old_dir: &std::path::Path, new_dir: &std::path::Path) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_STARTED,
        old_dir = %old_dir.display(),
        new_dir = %new_dir.display(),
        "config migration started"
    );
}

/// 记录配置迁移完成。
pub fn config_migration_completed() {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_COMPLETED,
        "config migration completed"
    );
}

/// 记录配置迁移失败。
pub fn config_migration_failed(error: &dyn Display) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_FAILED,
        error = %error,
        "config migration failed"
    );
}

/// 记录定位器文件不可读或格式错误。
pub fn config_locator_unreadable(path: &std::path::Path) {
    tracing::warn!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_LOCATOR_UNREADABLE,
        path = %path.display(),
        "locator file is unreadable or malformed"
    );
}

/// 记录删除定位器文件失败。
pub fn config_locator_cleanup_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_LOCATOR_CLEANUP_FAILED,
        path = %path.display(),
        error = %error,
        "failed to clean up locator file"
    );
}

/// Event: 迁移过程中目标目录已存在同名条目。
pub const EVENT_CONFIG_MIGRATION_CONFLICT: &str = "config_migration_conflict";

/// 记录迁移时目标路径已存在同名条目的覆盖/跳过冲突。
pub fn config_migration_conflict(path: &std::path::Path) {
    tracing::warn!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_CONFLICT,
        path = %path.display(),
        "migration found an existing entry at destination, overwriting"
    );
}

/// Event: 数据目录迁移完成（携带条目统计）。
pub const EVENT_CONFIG_MIGRATION_SUMMARY: &str = "config_migration_summary";

/// 记录数据目录迁移的条目统计（复制文件数 / 目录数）。
pub fn config_migration_summary(files_copied: usize, dirs_copied: usize) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_SUMMARY,
        files_copied,
        dirs_copied,
        "config migration finished"
    );
}

/// Event: 迁移路径无效（源与目标存在包含关系），迁移被拒绝。
pub const EVENT_CONFIG_MIGRATION_INVALID_PATH: &str = "config_migration_invalid_path";

/// 记录迁移路径存在包含关系、被拒绝迁移的事件。
///
/// 若旧目录是默认目录的祖先或子目录，递归复制会膨胀或误删数据树。
pub fn config_migration_invalid_path(old_dir: &std::path::Path, default_dir: &std::path::Path) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_INVALID_PATH,
        old_dir = %old_dir.display(),
        default_dir = %default_dir.display(),
        "migration rejected: source and destination overlap"
    );
}

/// Event: 迁移跳过符号链接条目。
pub const EVENT_CONFIG_MIGRATION_SYMLINK_SKIPPED: &str = "config_migration_symlink_skipped";

/// 记录迁移时跳过符号链接条目（避免跟随链接递归或复制链接指向的目录）。
pub fn config_migration_symlink_skipped(path: &std::path::Path) {
    tracing::warn!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_SYMLINK_SKIPPED,
        path = %path.display(),
        "migration skipped symbolic link entry"
    );
}

/// Event: 恢复上次中断迁移留下的残留目录。
pub const EVENT_CONFIG_MIGRATION_RESUMED: &str = "config_migration_resumed";

/// 记录上次迁移中断残留的迁移源已恢复为原目录名。
pub fn config_migration_resumed(staged: &std::path::Path, restored: &std::path::Path) {
    tracing::warn!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_RESUMED,
        staged = %staged.display(),
        restored = %restored.display(),
        "interrupted migration residue recovered to original name"
    );
}

/// Event: 清理迁移源失败（数据已迁移完成）。
pub const EVENT_CONFIG_MIGRATION_CLEANUP_FAILED: &str = "config_migration_cleanup_failed";

/// 记录迁移源清理失败——数据已完整复制到目标，残留目录可稍后手动清理。
pub fn config_migration_cleanup_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_CLEANUP_FAILED,
        path = %path.display(),
        error = %error,
        "failed to clean up migration source after successful copy"
    );
}

/// Event: 迁移失败后回滚重命名也失败。
pub const EVENT_CONFIG_MIGRATION_ROLLBACK_FAILED: &str = "config_migration_rollback_failed";

/// 记录迁移失败后回滚重命名失败——旧数据仍留在迁移源目录，需要人工介入。
pub fn config_migration_rollback_failed(
    staged: &std::path::Path,
    original: &std::path::Path,
    error: &dyn Display,
) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_MIGRATION_ROLLBACK_FAILED,
        staged = %staged.display(),
        original = %original.display(),
        error = %error,
        "failed to roll back migration source rename, user data may be left in staging directory"
    );
}

// ---------------------------------------------------------------------------
// 应用设置管理
// ---------------------------------------------------------------------------

/// Event: 设置加载完成。
pub const EVENT_CONFIG_SETTINGS_LOADED: &str = "config_settings_loaded";
/// Event: 设置全量更新成功。
pub const EVENT_CONFIG_SETTINGS_UPDATED: &str = "config_settings_updated";
/// Event: 设置部分更新成功。
pub const EVENT_CONFIG_SETTINGS_PARTIAL_UPDATED: &str = "config_settings_partial_updated";
/// Event: 设置持久化失败并已回滚。
pub const EVENT_CONFIG_SETTINGS_PERSIST_FAILED: &str = "config_settings_persist_failed";
/// Event: 设置重置为默认值。
pub const EVENT_CONFIG_SETTINGS_RESET: &str = "config_settings_reset";
/// Event: 设置导出为 JSON。
pub const EVENT_CONFIG_SETTINGS_EXPORTED: &str = "config_settings_exported";
/// Event: 设置从 JSON 导入。
pub const EVENT_CONFIG_SETTINGS_IMPORTED: &str = "config_settings_imported";
/// Event: 配置文件损坏，已备份并恢复默认配置。
pub const EVENT_CONFIG_SETTINGS_CORRUPT_RECOVERED: &str = "config_settings_corrupt_recovered";
/// Event: 设置版本升级成功。
pub const EVENT_CONFIG_SETTINGS_VERSION_UPGRADED: &str = "config_settings_version_upgraded";

/// 记录设置加载完成。
pub fn config_settings_loaded(path: &std::path::Path, version: u32) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_LOADED,
        path = %path.display(),
        version,
        "settings loaded"
    );
}

/// 记录设置全量更新成功。
pub fn config_settings_updated(path: &std::path::Path, changed_groups: &[SettingsGroup]) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_UPDATED,
        path = %path.display(),
        changed_groups = ?changed_groups,
        "settings updated"
    );
}

/// 记录设置部分更新成功。
pub fn config_settings_partial_updated(path: &std::path::Path, changed_groups: &[SettingsGroup]) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_PARTIAL_UPDATED,
        path = %path.display(),
        changed_groups = ?changed_groups,
        "settings partially updated"
    );
}

/// 记录设置持久化失败并已回滚内存状态。
pub fn config_settings_persist_failed(
    path: &std::path::Path,
    operation: &str,
    error: &dyn Display,
) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_PERSIST_FAILED,
        path = %path.display(),
        operation,
        error = %error,
        "settings persist failed, in-memory state rolled back"
    );
}

/// 记录设置重置为默认值。
pub fn config_settings_reset(path: &std::path::Path) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_RESET,
        path = %path.display(),
        "settings reset to defaults"
    );
}

/// 记录设置导出为 JSON。
pub fn config_settings_exported(path: &std::path::Path) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_EXPORTED,
        path = %path.display(),
        "settings exported as JSON"
    );
}

/// 记录设置从 JSON 导入。
pub fn config_settings_imported(path: &std::path::Path, changed_groups: &[SettingsGroup]) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_IMPORTED,
        path = %path.display(),
        changed_groups = ?changed_groups,
        "settings imported from JSON"
    );
}

/// 记录配置文件损坏、已备份并恢复默认配置的事件。
pub fn config_settings_corrupt_recovered(path: &std::path::Path, backup: &std::path::Path) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_CORRUPT_RECOVERED,
        path = %path.display(),
        backup = %backup.display(),
        "settings file corrupted, backed up and reset to defaults"
    );
}

/// 记录设置版本升级成功（升级前已备份原文件）。
pub fn config_settings_version_upgraded(
    path: &std::path::Path,
    from_version: u32,
    to_version: u32,
    backup: &std::path::Path,
) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_SETTINGS_VERSION_UPGRADED,
        path = %path.display(),
        from_version,
        to_version,
        backup = %backup.display(),
        "settings version upgraded"
    );
}

// ---------------------------------------------------------------------------
// 旧版嵌套配置迁移
// ---------------------------------------------------------------------------

/// Event: 旧版嵌套配置迁移成功。
pub const EVENT_CONFIG_LEGACY_MIGRATED: &str = "config_legacy_settings_migrated";
/// Event: 旧版配置迁移失败。
pub const EVENT_CONFIG_LEGACY_MIGRATE_FAILED: &str = "config_legacy_settings_migrate_failed";

/// 记录旧版嵌套配置迁移成功（不记录迁移前的敏感值）。
pub fn config_legacy_settings_migrated(path: &std::path::Path) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_LEGACY_MIGRATED,
        path = %path.display(),
        "legacy nested settings migrated to flat format"
    );
}

/// 记录旧版配置迁移失败。
pub fn config_legacy_settings_migrate_failed(path: &std::path::Path, error: &dyn Display) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_LEGACY_MIGRATE_FAILED,
        path = %path.display(),
        error = %error,
        "legacy settings migration failed"
    );
}

// ---------------------------------------------------------------------------
// 服务器注册表
// ---------------------------------------------------------------------------

/// Event: 注册表加载完成。
pub const EVENT_CONFIG_REGISTRY_LOADED: &str = "config_registry_loaded";
/// Event: 添加服务器。
pub const EVENT_CONFIG_REGISTRY_ADDED: &str = "config_registry_server_added";
/// Event: 更新服务器。
pub const EVENT_CONFIG_REGISTRY_UPDATED: &str = "config_registry_server_updated";
/// Event: 删除服务器。
pub const EVENT_CONFIG_REGISTRY_DELETED: &str = "config_registry_server_deleted";
/// Event: 拒绝重复 ID。
pub const EVENT_CONFIG_REGISTRY_DUPLICATE_ID: &str = "config_registry_duplicate_id";
/// Event: 服务器不存在，操作被跳过。
pub const EVENT_CONFIG_REGISTRY_NOT_FOUND: &str = "config_registry_server_not_found";
/// Event: 注册表操作失败。
pub const EVENT_CONFIG_REGISTRY_OPERATION_FAILED: &str = "config_registry_operation_failed";

/// 记录注册表加载完成。
pub fn config_registry_loaded(path: &std::path::Path, count: usize) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_LOADED,
        path = %path.display(),
        count,
        "server registry loaded"
    );
}

/// 记录添加服务器成功。
pub fn config_registry_server_added(path: &std::path::Path, id: &str, name: &str) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_ADDED,
        path = %path.display(),
        id,
        name,
        "server added to registry"
    );
}

/// 记录更新服务器成功。
pub fn config_registry_server_updated(path: &std::path::Path, id: &str) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_UPDATED,
        path = %path.display(),
        id,
        "server updated in registry"
    );
}

/// 记录删除服务器成功。
pub fn config_registry_server_deleted(path: &std::path::Path, id: &str) {
    tracing::info!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_DELETED,
        path = %path.display(),
        id,
        "server deleted from registry"
    );
}

/// 记录拒绝重复 ID。
pub fn config_registry_duplicate_id(path: &std::path::Path, id: &str) {
    tracing::warn!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_DUPLICATE_ID,
        path = %path.display(),
        id,
        "server add rejected: duplicate id"
    );
}

/// 记录按 ID 找不到服务器（更新/删除被静默跳过）。
pub fn config_registry_server_not_found(path: &std::path::Path, operation: &str, id: &str) {
    tracing::warn!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_NOT_FOUND,
        path = %path.display(),
        operation,
        id,
        "server not found, operation skipped"
    );
}

/// 记录注册表操作失败。
pub fn config_registry_operation_failed(
    path: &std::path::Path,
    operation: &str,
    id: Option<&str>,
    error: &dyn Display,
) {
    tracing::error!(
        target: CONFIG_TARGET,
        event_name = EVENT_CONFIG_REGISTRY_OPERATION_FAILED,
        path = %path.display(),
        operation,
        id,
        error = %error,
        "registry operation failed"
    );
}
