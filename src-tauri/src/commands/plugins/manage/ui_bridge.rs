use super::common::{lock_manager, validate_plugin_id, PluginManagerState};
use crate::plugins::api::{
    BufferedComponentEvent, BufferedContextMenuEvent, BufferedPermissionLog, BufferedSidebarEvent,
    BufferedUiEvent,
};

/// 通知插件右键菜单已关闭
pub(super) fn context_menu_hide_notify(manager: PluginManagerState<'_>) -> Result<(), String> {
    let manager = lock_manager(&manager);
    let runtimes = manager.get_shared_runtimes();
    let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

    for runtime in runtimes_guard.values() {
        let _ = runtime.call_context_menu_hide_callback();
    }

    Ok(())
}

/// 通知插件右键菜单即将显示
pub(super) fn context_menu_show_notify(
    context: String,
    target_data: serde_json::Value,
    x: f64,
    y: f64,
    manager: PluginManagerState<'_>,
) -> Result<(), String> {
    let manager = lock_manager(&manager);
    let runtimes = manager.get_shared_runtimes();
    let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

    for runtime in runtimes_guard.values() {
        let _ = runtime.call_context_menu_show_callback(&context, target_data.clone(), x, y);
    }

    Ok(())
}

/// 分发插件右键菜单点击回调
pub(super) fn context_menu_callback(
    plugin_id: String,
    context: String,
    item_id: String,
    target_data: serde_json::Value,
    manager: PluginManagerState<'_>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;

    let manager = lock_manager(&manager);
    let runtimes = manager.get_shared_runtimes();
    let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

    let runtime = runtimes_guard
        .get(&plugin_id)
        .ok_or_else(|| format!("插件 '{}' 的运行时不存在", plugin_id))?;

    runtime.call_context_menu_callback(&context, &item_id, target_data)
}

/// 通知插件当前语言已切换
pub(super) fn on_locale_changed(
    locale: String,
    manager: PluginManagerState<'_>,
) -> Result<(), String> {
    use crate::services::global::i18n_service;

    let i18n = i18n_service();
    i18n.set_locale(&locale);

    let manager = lock_manager(&manager);
    manager.notify_locale_changed(&locale);

    Ok(())
}

/// 注册组件镜像
pub(super) fn component_mirror_register(id: String, component_type: String) {
    crate::plugins::api::component_mirror_register(&id, &component_type);
}

/// 注销组件镜像
pub(super) fn component_mirror_unregister(id: String) {
    crate::plugins::api::component_mirror_unregister(&id);
}

/// 清空组件镜像
pub(super) fn component_mirror_clear() {
    crate::plugins::api::component_mirror_clear();
}

/// 通知插件当前页面已切换
pub(super) fn on_page_changed(
    path: String,
    manager: PluginManagerState<'_>,
) -> Result<(), String> {
    let manager = lock_manager(&manager);
    manager.notify_page_changed(&path);
    Ok(())
}

/// 读取组件事件快照
pub(super) fn get_plugin_component_snapshot() -> Vec<BufferedComponentEvent> {
    crate::plugins::api::take_component_event_snapshot()
}

/// 读取 UI 事件快照
pub(super) fn get_plugin_ui_snapshot() -> Vec<BufferedUiEvent> {
    crate::plugins::api::take_ui_event_snapshot()
}

/// 读取侧边栏事件快照
pub(super) fn get_plugin_sidebar_snapshot() -> Vec<BufferedSidebarEvent> {
    crate::plugins::api::take_sidebar_event_snapshot()
}

/// 读取右键菜单事件快照
pub(super) fn get_plugin_context_menu_snapshot() -> Vec<BufferedContextMenuEvent> {
    crate::plugins::api::take_context_menu_snapshot()
}

/// 读取插件权限日志
pub(super) fn get_plugin_permission_logs(
    plugin_id: String,
) -> Result<Vec<BufferedPermissionLog>, String> {
    validate_plugin_id(&plugin_id)?;
    Ok(crate::plugins::api::get_plugin_permission_logs(&plugin_id))
}
