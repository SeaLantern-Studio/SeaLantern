use super::common::{lock_manager, validate_plugin_id, PermissionInfo, PluginManagerState};
use crate::hardcode_data::plugin_permissions::get_plugin_permission_list;

pub(super) fn get_permission_list() -> Vec<PermissionInfo> {
    get_plugin_permission_list()
}

// 获取插件已申请的权限列表
pub(super) fn get_plugin_permissions(
    plugin_id: String,
    manager: PluginManagerState<'_>,
) -> Result<Vec<PermissionInfo>, String> {
    validate_plugin_id(&plugin_id)?;

    let manager = lock_manager(&manager);
    let plugin_list = manager.get_plugin_list();

    let plugin = plugin_list
        .iter()
        .find(|plugin| plugin.manifest.id == plugin_id)
        .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

    let plugin_permissions = get_permission_list()
        .into_iter()
        .filter(|permission| plugin.manifest.permissions.contains(&permission.id))
        .collect();

    Ok(plugin_permissions)
}
