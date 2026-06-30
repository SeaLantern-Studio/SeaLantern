use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SHARED_PLUGIN_PERMISSIONS_JSON: &str =
    include_str!("../../../../shared/plugin-permissions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub danger_level: String,
    pub category: String,
    pub icon: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default = "default_risk_group")]
    pub risk_group: String,
    #[serde(default)]
    pub trusted_only: bool,
    #[serde(default = "default_within_standard_ceiling")]
    pub within_standard_ceiling: bool,
    #[serde(default)]
    pub requires_explicit_consent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginPermissionRiskGroup {
    StandardSandboxAllowed,
    EscalatedSandbox,
    TrustedOnly,
    Unknown,
}

static PERMISSIONS: Lazy<Vec<PluginPermissionInfo>> = Lazy::new(|| {
    serde_json::from_str(SHARED_PLUGIN_PERMISSIONS_JSON)
        .expect("shared/plugin-permissions.json must stay valid")
});

static ALIAS_TO_ID: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for item in PERMISSIONS.iter() {
        map.insert(item.id.clone(), item.id.clone());
        for alias in &item.aliases {
            map.insert(alias.clone(), item.id.clone());
        }
    }
    map
});

pub fn get_plugin_permission_list() -> Vec<PluginPermissionInfo> {
    PERMISSIONS.clone()
}

pub fn get_permission_info(permission_id: &str) -> Option<PluginPermissionInfo> {
    let canonical = normalize_permission_id(permission_id);
    PERMISSIONS
        .iter()
        .find(|item| item.id == canonical)
        .cloned()
}

pub fn normalize_permission_id(permission_id: &str) -> String {
    ALIAS_TO_ID
        .get(permission_id.trim())
        .cloned()
        .unwrap_or_else(|| permission_id.trim().to_string())
}

pub fn is_known_permission_or_alias(permission_id: &str) -> bool {
    ALIAS_TO_ID.contains_key(permission_id.trim())
        || is_valid_fine_grained_fs_permission(permission_id)
}

pub fn requires_explicit_consent(permissions: &[String]) -> bool {
    permissions.iter().any(|permission| {
        if let Some(info) = get_permission_info(permission) {
            info.requires_explicit_consent
        } else {
            false
        }
    })
}

pub fn permission_risk_group(permission_id: &str) -> PluginPermissionRiskGroup {
    let canonical = normalize_permission_id(permission_id);

    if let Some(info) = get_permission_info(&canonical) {
        return match info.risk_group.as_str() {
            "standard_sandbox_allowed" => PluginPermissionRiskGroup::StandardSandboxAllowed,
            "escalated_sandbox" => PluginPermissionRiskGroup::EscalatedSandbox,
            "trusted_only" => PluginPermissionRiskGroup::TrustedOnly,
            _ => PluginPermissionRiskGroup::Unknown,
        };
    }

    if is_valid_standard_sandbox_fs_permission(&canonical) {
        return PluginPermissionRiskGroup::StandardSandboxAllowed;
    }

    PluginPermissionRiskGroup::Unknown
}

pub fn exceeds_standard_sandbox_ceiling(permissions: &[String]) -> bool {
    permissions.iter().any(|permission| {
        !matches!(
            permission_risk_group(permission),
            PluginPermissionRiskGroup::StandardSandboxAllowed
        )
    })
}

pub fn requests_trusted_capabilities(permissions: &[String]) -> bool {
    permissions.iter().any(|permission| {
        let canonical = normalize_permission_id(permission);
        get_permission_info(&canonical)
            .map(|info| info.trusted_only)
            .unwrap_or(false)
            || matches!(permission_risk_group(&canonical), PluginPermissionRiskGroup::TrustedOnly)
    })
}

fn default_risk_group() -> String {
    "unknown".to_string()
}

fn default_within_standard_ceiling() -> bool {
    true
}

fn is_valid_standard_sandbox_fs_permission(permission_id: &str) -> bool {
    let mut parts = permission_id.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("fs"), Some("data"), Some(action), None)
            if matches!(action, "read" | "write" | "list" | "meta" | "delete" | "transfer")
    )
}

fn is_valid_fine_grained_fs_permission(permission_id: &str) -> bool {
    let mut parts = permission_id.trim().split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("fs"), Some(scope), Some(action), None)
            if matches!(scope, "data" | "server" | "global")
                && matches!(action, "read" | "write" | "list" | "meta" | "delete" | "transfer")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        get_permission_info, permission_risk_group, requests_trusted_capabilities,
        PluginPermissionRiskGroup,
    };

    #[test]
    fn shared_permission_metadata_drives_risk_group() {
        assert_eq!(permission_risk_group("process.exec"), PluginPermissionRiskGroup::TrustedOnly);
        assert_eq!(permission_risk_group("network"), PluginPermissionRiskGroup::EscalatedSandbox);
        assert_eq!(permission_risk_group("log"), PluginPermissionRiskGroup::StandardSandboxAllowed);
    }

    #[test]
    fn shared_permission_metadata_exposes_trusted_only_flag() {
        let process_exec = get_permission_info("process.exec").expect("known permission");
        assert!(process_exec.trusted_only);
        assert!(!process_exec.within_standard_ceiling);

        let log = get_permission_info("log").expect("known permission");
        assert!(!log.trusted_only);
        assert!(log.within_standard_ceiling);
    }

    #[test]
    fn trusted_capability_detection_uses_shared_metadata() {
        assert!(requests_trusted_capabilities(&["process.exec".to_string()]));
        assert!(!requests_trusted_capabilities(&["network".to_string()]));
    }
}
