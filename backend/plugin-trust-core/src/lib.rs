//! Plugin trust, integrity, and enable-consent helpers shared by the host runtime.

mod permissions;
mod trust;

pub use permissions::{
    exceeds_standard_sandbox_ceiling, get_permission_info, get_plugin_permission_list,
    is_known_permission_or_alias, normalize_permission_id, normalize_permissions,
    permission_risk_group, requests_trusted_capabilities, requires_explicit_consent,
    PluginPermissionInfo, PluginPermissionRiskGroup,
};
pub use trust::*;
