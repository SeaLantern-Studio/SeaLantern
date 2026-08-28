//! Tauri 对插件 v2 宿主服务的本地命令适配。

use std::path::PathBuf;

use sealantern_application::plugin::{
    AuditEntry, CorePluginService, PluginService, PluginServiceError, SessionApproval, SessionGrant,
};
use sealantern_application::services::AppServices;
use sealantern_core::app_plugin::{CapabilityInvocation, ScopeBinding, TrustSource};
use sealantern_feature::app_plugin::PluginInfo;

async fn plugin_service() -> Result<std::sync::Arc<CorePluginService>, String> {
    AppServices::plugin_service()
        .await
        .map_err(|error| error.to_string())
}

fn map_error(error: PluginServiceError) -> String {
    error.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_discover() -> Result<Vec<PathBuf>, String> {
    plugin_service().await?.discover().await.map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_load(plugin_dir: PathBuf) -> Result<PluginInfo, String> {
    plugin_service()
        .await?
        .load(&plugin_dir)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_enable(plugin_id: String) -> Result<(), String> {
    plugin_service()
        .await?
        .enable(&plugin_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_disable(plugin_id: String) -> Result<(), String> {
    plugin_service()
        .await?
        .disable(&plugin_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_unload(plugin_id: String) -> Result<(), String> {
    plugin_service()
        .await?
        .unload(&plugin_id)
        .await
        .map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_plugins() -> Result<Vec<PluginInfo>, String> {
    plugin_service().await?.plugins().await.map_err(map_error)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_grant_persistent(
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
) -> Result<(), String> {
    plugin_service()
        .await?
        .policy()
        .grant_persistent(&plugin_id, &capability_id, scope.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_revoke_persistent(
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
) -> Result<(), String> {
    plugin_service()
        .await?
        .policy()
        .revoke_persistent(&plugin_id, &capability_id, scope.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_set_trust(
    plugin_id: String,
    trust_source: TrustSource,
) -> Result<(), String> {
    plugin_service()
        .await?
        .policy()
        .set_trust(&plugin_id, trust_source)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_grant_session(grant: SessionGrant) -> Result<(), String> {
    plugin_service()
        .await?
        .policy()
        .grant_session(grant)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_approve_session(approval: SessionApproval) -> Result<(), String> {
    plugin_service()
        .await?
        .policy()
        .approve_session(approval)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_issue_approval_token(
    session_id: String,
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
) -> Result<String, String> {
    plugin_service()
        .await?
        .policy()
        .issue_single_use_token(&session_id, &plugin_id, &capability_id, scope)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_end_session(session_id: String) -> Result<(), String> {
    plugin_service()
        .await?
        .policy()
        .end_session(&session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_audit(limit: u32) -> Result<Vec<AuditEntry>, String> {
    plugin_service()
        .await?
        .policy()
        .audit_entries(limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_v2_invoke(
    invocation: CapabilityInvocation,
) -> Result<serde_json::Value, String> {
    plugin_service()
        .await?
        .invoke(invocation)
        .await
        .map_err(map_error)
}
