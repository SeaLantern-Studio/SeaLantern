pub use plugin_trust::{PluginPermissionInfo, PluginPermissionRiskGroup};

pub fn get_plugin_permission_list() -> Vec<PluginPermissionInfo> {
    plugin_trust::get_plugin_permission_list()
}

#[allow(dead_code)]
pub fn get_permission_info(permission_id: &str) -> Option<PluginPermissionInfo> {
    plugin_trust::get_permission_info(permission_id)
}

pub fn normalize_permission_id(permission_id: &str) -> String {
    plugin_trust::normalize_permission_id(permission_id)
}

#[allow(dead_code)]
pub fn is_known_permission_or_alias(permission_id: &str) -> bool {
    plugin_trust::is_known_permission_or_alias(permission_id)
}

#[allow(dead_code)]
pub fn requires_explicit_consent(permissions: &[String]) -> bool {
    plugin_trust::requires_explicit_consent(permissions)
}

#[allow(dead_code)]
pub fn permission_risk_group(permission_id: &str) -> PluginPermissionRiskGroup {
    plugin_trust::permission_risk_group(permission_id)
}

#[allow(dead_code)]
pub fn exceeds_standard_sandbox_ceiling(permissions: &[String]) -> bool {
    plugin_trust::exceeds_standard_sandbox_ceiling(permissions)
}

pub fn requests_trusted_capabilities(permissions: &[String]) -> bool {
    plugin_trust::requests_trusted_capabilities(permissions)
}
