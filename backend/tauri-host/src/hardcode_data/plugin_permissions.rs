pub use sea_lantern_plugin_trust_core::{PluginPermissionInfo, PluginPermissionRiskGroup};

pub fn get_plugin_permission_list() -> Vec<PluginPermissionInfo> {
    sea_lantern_plugin_trust_core::get_plugin_permission_list()
}

#[allow(dead_code)]
pub fn get_permission_info(permission_id: &str) -> Option<PluginPermissionInfo> {
    sea_lantern_plugin_trust_core::get_permission_info(permission_id)
}

pub fn normalize_permission_id(permission_id: &str) -> String {
    sea_lantern_plugin_trust_core::normalize_permission_id(permission_id)
}

#[allow(dead_code)]
pub fn is_known_permission_or_alias(permission_id: &str) -> bool {
    sea_lantern_plugin_trust_core::is_known_permission_or_alias(permission_id)
}

#[allow(dead_code)]
pub fn requires_explicit_consent(permissions: &[String]) -> bool {
    sea_lantern_plugin_trust_core::requires_explicit_consent(permissions)
}

#[allow(dead_code)]
pub fn permission_risk_group(permission_id: &str) -> PluginPermissionRiskGroup {
    sea_lantern_plugin_trust_core::permission_risk_group(permission_id)
}

#[allow(dead_code)]
pub fn exceeds_standard_sandbox_ceiling(permissions: &[String]) -> bool {
    sea_lantern_plugin_trust_core::exceeds_standard_sandbox_ceiling(permissions)
}

pub fn requests_trusted_capabilities(permissions: &[String]) -> bool {
    sea_lantern_plugin_trust_core::requests_trusted_capabilities(permissions)
}
