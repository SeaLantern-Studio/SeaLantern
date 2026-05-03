mod github;
mod install;
mod query;
mod shared;

use super::common::{lock_manager, trim_market_base_url, validate_plugin_id, PluginManagerState};
use crate::hardcode_data::plugin_market::PLUGIN_MARKET_BASE_URL;
use crate::models::plugin::{PluginInstallResult, PluginUpdateInfo};
pub(super) use install::InstallFromMarketRequest;

/// 检查单个插件更新
pub(super) async fn check_plugin_update(
    manager: PluginManagerState<'_>,
    plugin_id: String,
) -> Result<Option<PluginUpdateInfo>, String> {
    validate_plugin_id(&plugin_id)?;

    let current_version = {
        let manager = lock_manager(&manager);
        let plugin_info = manager
            .plugins()
            .get(&plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
        plugin_info.manifest.version.clone()
    };

    query::check_plugin_update(current_version, plugin_id).await
}

/// 批量检查全部插件更新
pub(super) async fn check_all_plugin_updates(
    manager: PluginManagerState<'_>,
) -> Result<Vec<PluginUpdateInfo>, String> {
    let plugin_versions: Vec<(String, String)> = {
        let manager = lock_manager(&manager);
        manager
            .plugins()
            .iter()
            .map(|(id, info)| (id.clone(), info.manifest.version.clone()))
            .collect()
    };

    query::check_all_plugin_updates(plugin_versions).await
}

/// 拉取插件市场列表
pub(super) async fn fetch_market_plugins(
    _manager: PluginManagerState<'_>,
    market_url: Option<String>,
) -> Result<Vec<crate::models::plugin::MarketPluginInfo>, String> {
    let base_url = trim_market_base_url(market_url, PLUGIN_MARKET_BASE_URL);
    query::fetch_market_plugins(base_url).await
}

/// 拉取插件市场分类
pub(super) async fn fetch_market_categories(
    market_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let base_url = trim_market_base_url(market_url, PLUGIN_MARKET_BASE_URL);
    query::fetch_market_categories(base_url).await
}

/// 拉取单个插件详情
pub(super) async fn fetch_market_plugin_detail(
    _manager: PluginManagerState<'_>,
    plugin_path: String,
    market_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let base_url = trim_market_base_url(market_url, PLUGIN_MARKET_BASE_URL);
    query::fetch_market_plugin_detail(base_url, plugin_path).await
}

/// 从插件市场安装插件
pub(super) async fn install_from_market(
    manager: PluginManagerState<'_>,
    req: InstallFromMarketRequest,
) -> Result<PluginInstallResult, String> {
    validate_plugin_id(&req.plugin_id)?;
    install::install_from_market(manager, req).await
}
