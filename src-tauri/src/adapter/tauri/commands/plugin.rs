//! Tauri 对插件 v2 宿主服务的本地命令适配。

use std::path::PathBuf;

use sealantern_application::plugin::{
    AuditEntry, CorePluginService, PluginService, PluginServiceError, SessionApproval, SessionGrant,
};
use sealantern_application::services::AppServices;
use sealantern_core::app_plugin::{CapabilityInvocation, ScopeBinding, TrustSource};
use sealantern_feature::app_plugin::PluginInfo;
use tauri::State;

async fn plugin_service(
    services: &AppServices,
) -> Result<std::sync::Arc<CorePluginService>, String> {
    services
        .plugin()
        .await
        .map(Clone::clone)
        .map_err(|error| error.to_string())
}

fn map_error(error: PluginServiceError) -> String {
    error.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_discover(services: State<'_, AppServices>) -> Result<Vec<PathBuf>, String> {
    plugin_service(&services)
        .await?
        .discover()
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_load(
    services: State<'_, AppServices>,
    plugin_dir: PathBuf,
) -> Result<PluginInfo, String> {
    plugin_service(&services)
        .await?
        .load(&plugin_dir)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_enable(
    services: State<'_, AppServices>,
    plugin_id: String,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .enable(&plugin_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_disable(
    services: State<'_, AppServices>,
    plugin_id: String,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .disable(&plugin_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_unload(
    services: State<'_, AppServices>,
    plugin_id: String,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .unload(&plugin_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_plugins(
    services: State<'_, AppServices>,
) -> Result<Vec<PluginInfo>, String> {
    plugin_service(&services)
        .await?
        .plugins()
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_grant_persistent(
    services: State<'_, AppServices>,
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .policy()
        .grant_persistent(&plugin_id, &capability_id, scope.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_revoke_persistent(
    services: State<'_, AppServices>,
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .policy()
        .revoke_persistent(&plugin_id, &capability_id, scope.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_set_trust(
    services: State<'_, AppServices>,
    plugin_id: String,
    trust_source: TrustSource,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .policy()
        .set_trust(&plugin_id, trust_source)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_grant_session(
    services: State<'_, AppServices>,
    grant: SessionGrant,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .policy()
        .grant_session(grant)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_approve_session(
    services: State<'_, AppServices>,
    approval: SessionApproval,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .policy()
        .approve_session(approval)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_issue_approval_token(
    services: State<'_, AppServices>,
    session_id: String,
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
) -> Result<String, String> {
    plugin_service(&services)
        .await?
        .policy()
        .issue_single_use_token(&session_id, &plugin_id, &capability_id, scope)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_end_session(
    services: State<'_, AppServices>,
    session_id: String,
) -> Result<(), String> {
    plugin_service(&services)
        .await?
        .policy()
        .end_session(&session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_audit(
    services: State<'_, AppServices>,
    limit: u32,
) -> Result<Vec<AuditEntry>, String> {
    plugin_service(&services)
        .await?
        .policy()
        .audit_entries(limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_invoke(
    services: State<'_, AppServices>,
    invocation: CapabilityInvocation,
) -> Result<serde_json::Value, String> {
    plugin_service(&services)
        .await?
        .invoke(invocation)
        .await
        .map_err(map_error)
}
