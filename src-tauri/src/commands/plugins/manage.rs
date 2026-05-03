//! 插件管理命令入口
//!
//! 这里按基础管理、市场、权限和前端桥接拆到子模块，对外保留 Tauri 命令入口

mod basic;
mod common;
mod market;
mod permissions;
mod ui_bridge;

/// 前端显示用的权限信息
pub type PermissionInfo = common::PermissionInfo;

#[tauri::command]
/// 读取插件列表
pub fn list_plugins(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<crate::models::plugin::PluginInfo>, String> {
    basic::list_plugins(manager)
}

#[tauri::command]
/// 重新扫描插件目录
pub fn scan_plugins(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<crate::models::plugin::PluginInfo>, String> {
    basic::scan_plugins(manager)
}

#[tauri::command]
/// 启用插件
pub fn enable_plugin(
    plugin_id: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    basic::enable_plugin(plugin_id, manager)
}

#[tauri::command]
/// 禁用插件
pub fn disable_plugin(
    plugin_id: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<String>, String> {
    basic::disable_plugin(plugin_id, manager)
}

#[tauri::command]
pub fn get_plugin_nav_items(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<serde_json::Value>, String> {
    basic::get_plugin_nav_items(manager)
}

#[tauri::command]
pub fn install_plugin(
    path: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<crate::models::plugin::PluginInstallResult, String> {
    basic::install_plugin(path, manager)
}

#[tauri::command]
pub fn get_plugin_icon(
    plugin_id: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<String, String> {
    basic::get_plugin_icon(plugin_id, manager)
}

#[tauri::command]
pub fn get_plugin_settings(
    plugin_id: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<serde_json::Value, String> {
    basic::get_plugin_settings(plugin_id, manager)
}

#[tauri::command]
pub fn set_plugin_settings(
    plugin_id: String,
    settings: serde_json::Value,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    basic::set_plugin_settings(plugin_id, settings, manager)
}

#[tauri::command]
pub fn get_plugin_css(
    plugin_id: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<String, String> {
    basic::get_plugin_css(plugin_id, manager)
}

#[tauri::command]
pub fn get_all_plugin_css(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<(String, String)>, String> {
    basic::get_all_plugin_css(manager)
}

#[tauri::command]
pub async fn delete_plugin(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
    plugin_id: String,
    delete_data: Option<bool>,
) -> Result<(), String> {
    basic::delete_plugin(manager, plugin_id, delete_data).await
}

#[tauri::command]
pub async fn delete_plugins(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
    plugin_ids: Vec<String>,
    delete_data: Option<bool>,
) -> Result<(), String> {
    basic::delete_plugins(manager, plugin_ids, delete_data).await
}

#[tauri::command]
pub async fn check_plugin_update(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
    plugin_id: String,
) -> Result<Option<crate::models::plugin::PluginUpdateInfo>, String> {
    market::check_plugin_update(manager, plugin_id).await
}

#[tauri::command]
pub async fn check_all_plugin_updates(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<crate::models::plugin::PluginUpdateInfo>, String> {
    market::check_all_plugin_updates(manager).await
}

#[tauri::command]
pub async fn fetch_market_plugins(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
    market_url: Option<String>,
) -> Result<Vec<crate::models::plugin::MarketPluginInfo>, String> {
    market::fetch_market_plugins(manager, market_url).await
}

#[tauri::command]
pub async fn fetch_market_categories(
    market_url: Option<String>,
) -> Result<serde_json::Value, String> {
    market::fetch_market_categories(market_url).await
}

#[tauri::command]
pub async fn fetch_market_plugin_detail(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
    plugin_path: String,
    market_url: Option<String>,
) -> Result<serde_json::Value, String> {
    market::fetch_market_plugin_detail(manager, plugin_path, market_url).await
}

#[tauri::command]
/// 从插件市场安装插件
pub async fn install_from_market(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
    req: market::InstallFromMarketRequest,
) -> Result<crate::models::plugin::PluginInstallResult, String> {
    market::install_from_market(manager, req).await
}

#[tauri::command]
pub fn install_plugins_batch(
    paths: Vec<String>,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<crate::models::plugin::BatchInstallResult, String> {
    basic::install_plugins_batch(paths, manager)
}

#[tauri::command]
pub fn context_menu_hide_notify(
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    ui_bridge::context_menu_hide_notify(manager)
}

#[tauri::command]
pub fn context_menu_show_notify(
    context: String,
    target_data: serde_json::Value,
    x: f64,
    y: f64,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    ui_bridge::context_menu_show_notify(context, target_data, x, y, manager)
}

#[tauri::command]
pub fn context_menu_callback(
    plugin_id: String,
    context: String,
    item_id: String,
    target_data: serde_json::Value,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    ui_bridge::context_menu_callback(plugin_id, context, item_id, target_data, manager)
}

#[tauri::command]
pub fn on_locale_changed(
    locale: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    ui_bridge::on_locale_changed(locale, manager)
}

#[tauri::command]
pub fn component_mirror_register(id: String, component_type: String) {
    ui_bridge::component_mirror_register(id, component_type)
}

#[tauri::command]
pub fn component_mirror_unregister(id: String) {
    ui_bridge::component_mirror_unregister(id)
}

#[tauri::command]
pub fn component_mirror_clear() {
    ui_bridge::component_mirror_clear()
}

#[tauri::command]
pub fn on_page_changed(
    path: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<(), String> {
    ui_bridge::on_page_changed(path, manager)
}

#[tauri::command]
pub fn get_plugin_component_snapshot() -> Vec<crate::plugins::api::BufferedComponentEvent> {
    ui_bridge::get_plugin_component_snapshot()
}

#[tauri::command]
pub fn get_plugin_ui_snapshot() -> Vec<crate::plugins::api::BufferedUiEvent> {
    ui_bridge::get_plugin_ui_snapshot()
}

#[tauri::command]
pub fn get_plugin_sidebar_snapshot() -> Vec<crate::plugins::api::BufferedSidebarEvent> {
    ui_bridge::get_plugin_sidebar_snapshot()
}

#[tauri::command]
pub fn get_plugin_context_menu_snapshot() -> Vec<crate::plugins::api::BufferedContextMenuEvent> {
    ui_bridge::get_plugin_context_menu_snapshot()
}

#[tauri::command]
pub fn get_plugin_permission_logs(
    plugin_id: String,
) -> Result<Vec<crate::plugins::api::BufferedPermissionLog>, String> {
    ui_bridge::get_plugin_permission_logs(plugin_id)
}

#[tauri::command]
pub fn get_permission_list() -> Vec<PermissionInfo> {
    permissions::get_permission_list()
}

#[tauri::command]
pub fn get_plugin_permissions(
    plugin_id: String,
    manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>,
) -> Result<Vec<PermissionInfo>, String> {
    permissions::get_plugin_permissions(plugin_id, manager)
}
