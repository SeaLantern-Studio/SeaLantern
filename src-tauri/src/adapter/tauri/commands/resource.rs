//! Tauri 对资源管理服务的本地命令适配。

use sealantern_application::port::{ResolvedDownload, ResourceService};
use sealantern_application::services::AppServices;
use sealantern_contract::ResourceServiceError;
use sealantern_feature::resource::manager::{
    InstanceExtension, ManagedResource, ReconcileReport, ResourceProvenance,
};
use sealantern_feature::resource::market::{
    MarketSource, ResourceInfo, ResourceType, SearchResult, Version,
};
use tauri::State;

fn map_error(error: ResourceServiceError) -> String {
    error.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_list(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<ReconcileReport, String> {
    services
        .resource()
        .list(&instance_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_install(
    services: State<'_, AppServices>,
    instance_id: String,
    source_path: String,
    kind: ResourceType,
    provenance: Option<ResourceProvenance>,
) -> Result<ManagedResource, String> {
    services
        .resource()
        .install(&instance_id, &source_path, kind, provenance)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_remove(
    services: State<'_, AppServices>,
    instance_id: String,
    file_name: String,
) -> Result<(), String> {
    services
        .resource()
        .remove(&instance_id, &file_name)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_set_enabled(
    services: State<'_, AppServices>,
    instance_id: String,
    file_name: String,
    enabled: bool,
) -> Result<InstanceExtension, String> {
    services
        .resource()
        .set_enabled(&instance_id, &file_name, enabled)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_resources_sync(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<ReconcileReport, String> {
    services
        .resource()
        .sync(&instance_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_search(
    services: State<'_, AppServices>,
    source: MarketSource,
    query: String,
    page: u32,
    page_size: u32,
) -> Result<SearchResult, String> {
    services
        .resource()
        .market_search(source, &query, page, page_size)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_resource(
    services: State<'_, AppServices>,
    source: MarketSource,
    id: String,
) -> Result<ResourceInfo, String> {
    services
        .resource()
        .market_resource(source, &id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_versions(
    services: State<'_, AppServices>,
    source: MarketSource,
    id: String,
) -> Result<Vec<Version>, String> {
    services
        .resource()
        .market_versions(source, &id)
        .await
        .map_err(map_error)
}

/// 解析"从市场安装"的下载目标（不实际下载；由前端驱动下载后调 install）。
#[tauri::command(rename_all = "snake_case")]
pub async fn resource_market_resolve_download(
    services: State<'_, AppServices>,
    source: MarketSource,
    project_id: String,
    version_id: String,
) -> Result<ResolvedDownload, String> {
    services
        .resource()
        .market_resolve_download(source, &project_id, &version_id)
        .await
        .map_err(map_error)
}
