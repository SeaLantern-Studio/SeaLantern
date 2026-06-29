use super::common::{lock_manager, validate_plugin_id};
use crate::plugins::api::BufferedPermissionLog;
use crate::plugins::manager::PluginManager;
use std::sync::{Arc, Mutex};

/// 通知插件右键菜单已关闭
pub(super) fn context_menu_hide_notify(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let manager = lock_manager(&manager);
    manager.notify_context_menu_hide();
    Ok(())
}

/// 通知插件右键菜单即将显示
pub(super) fn context_menu_show_notify(
    context: String,
    target_data: serde_json::Value,
    x: f64,
    y: f64,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let manager = lock_manager(&manager);
    manager.notify_context_menu_show(&context, &target_data, x, y);
    Ok(())
}

/// 分发插件右键菜单点击回调
pub(super) fn context_menu_callback(
    plugin_id: String,
    context: String,
    item_id: String,
    target_data: serde_json::Value,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;

    let manager = lock_manager(&manager);
    manager.dispatch_context_menu_callback(&plugin_id, &context, &item_id, target_data)
}

/// 通知插件当前语言已切换
pub(super) fn on_locale_changed(
    locale: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    use crate::services::global::i18n_service;

    let i18n = i18n_service();
    i18n.set_locale(&locale);

    let manager = lock_manager(&manager);
    manager.notify_locale_changed(&locale);

    Ok(())
}

/// 通知插件当前页面已切换
pub(super) fn on_page_changed(
    path: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let manager = lock_manager(&manager);
    manager.notify_page_changed(&path);
    Ok(())
}

/// 读取插件权限日志
pub(super) fn get_plugin_permission_logs(
    plugin_id: String,
) -> Result<Vec<BufferedPermissionLog>, String> {
    validate_plugin_id(&plugin_id)?;
    Ok(crate::plugins::api::get_plugin_permission_logs(&plugin_id))
}
