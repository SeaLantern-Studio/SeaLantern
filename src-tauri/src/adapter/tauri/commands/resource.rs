//! Tauri 对资源管理服务的本地命令适配。
//!
//! 错误统一返回契约枚举 [`ResourceServiceError`]（`snake_case` 序列化），
//! 前端可按结构化错误码分支处理，与 instance / cron 等命令的约定一致。

use sealantern_application::port::{ResolvedDownload, ResourceService};
use sealantern_application::services::AppServices;
use sealantern_contract::ResourceServiceError;
use sealantern_feature::resource::manager::{
    InstanceExtension, ManagedResource, ReconcileReport, ResourceProvenance, ResourceTargets,
};
use sealantern_feature::resource::market::{
    MarketSource, ResourceInfo, ResourceType, SearchResult, Version,
};
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_list(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<ReconcileReport, ResourceServiceError> {
    services.resource().list(&instance_id).await
}

/// 列出实例可管理的资源目录与主要种类（只读）。
///
/// 资源列表为空时前端无法从条目反推该实例支持什么，用本命令决定展示哪些
/// 资源分类。
#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_targets(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<ResourceTargets, ResourceServiceError> {
    services.resource().targets(&instance_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_install(
    services: State<'_, AppServices>,
    instance_id: String,
    source_path: String,
    kind: ResourceType,
    provenance: Option<ResourceProvenance>,
) -> Result<ManagedResource, ResourceServiceError> {
    services
        .resource()
        .install(&instance_id, &source_path, kind, provenance)
        .await
}

/// 卸载实例资源。
///
/// `kind` 取自 [`instance_resources_list`] 返回条目的 `kind`；`file_name`
/// 为磁盘当前文件名（禁用态带 `.disabled` 后缀）。
#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_remove(
    services: State<'_, AppServices>,
    instance_id: String,
    kind: ResourceType,
    file_name: String,
) -> Result<(), ResourceServiceError> {
    services
        .resource()
        .remove(&instance_id, kind, &file_name)
        .await
}

/// 启用 / 禁用实例资源。
///
/// `kind` 取自 [`instance_resources_list`] 返回条目的 `kind`；`file_name`
/// 为磁盘当前文件名。
#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_set_enabled(
    services: State<'_, AppServices>,
    instance_id: String,
    kind: ResourceType,
    file_name: String,
    enabled: bool,
) -> Result<InstanceExtension, ResourceServiceError> {
    services
        .resource()
        .set_enabled(&instance_id, kind, &file_name, enabled)
        .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_sync(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<ReconcileReport, ResourceServiceError> {
    services.resource().sync(&instance_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_search(
    services: State<'_, AppServices>,
    source: MarketSource,
    query: String,
    page: u32,
    page_size: u32,
) -> Result<SearchResult, ResourceServiceError> {
    services
        .resource()
        .market_search(source, &query, page, page_size)
        .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_resource(
    services: State<'_, AppServices>,
    source: MarketSource,
    id: String,
) -> Result<ResourceInfo, ResourceServiceError> {
    services.resource().market_resource(source, &id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_versions(
    services: State<'_, AppServices>,
    source: MarketSource,
    id: String,
) -> Result<Vec<Version>, ResourceServiceError> {
    services.resource().market_versions(source, &id).await
}

/// 解析"从市场安装"的下载目标（不实际下载；由前端驱动下载后调 install）。
#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_resolve_download(
    services: State<'_, AppServices>,
    source: MarketSource,
    project_id: String,
    version_id: String,
) -> Result<ResolvedDownload, ResourceServiceError> {
    services
        .resource()
        .market_resolve_download(source, &project_id, &version_id)
        .await
}
