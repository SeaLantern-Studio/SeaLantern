use std::collections::HashSet;

pub(crate) const PLUGIN_FOLDER_ACCESS_PERMISSION: &str = "plugin_folder_access";
pub(crate) const LEGACY_PLUGINS_PERMISSION: &str = "plugins";

const BASE_PERMISSIONS: &[&str] = &[
    "log",
    "fs",
    "fs.data",
    "fs.server",
    "fs.global",
    "api",
    "storage",
    "network",
    "system",
    "server",
    "console",
    "ui",
    "element",
    "execute_program",
    PLUGIN_FOLDER_ACCESS_PERMISSION,
    LEGACY_PLUGINS_PERMISSION,
    "ui.component.read",
    "ui.component.write",
    "ui.component.proxy",
    "ui.component.create",
];

const FS_SCOPES: &[&str] = &["data", "server", "global"];
const FS_ACTIONS: &[&str] = &["read", "write", "list", "meta", "delete", "transfer"];

pub(crate) fn normalize_permission(permission: &str) -> String {
    match permission {
        "fs" => "fs.data".to_string(),
        LEGACY_PLUGINS_PERMISSION => PLUGIN_FOLDER_ACCESS_PERMISSION.to_string(),
        _ => permission.to_string(),
    }
}

pub(crate) fn normalize_permissions<I>(permissions: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for permission in permissions {
        let permission = normalize_permission(&permission);
        if seen.insert(permission.clone()) {
            normalized.push(permission);
        }
    }

    normalized
}

pub(crate) fn is_valid_declared_permission(permission: &str) -> bool {
    BASE_PERMISSIONS.contains(&permission) || is_valid_fs_action_permission(permission)
}

pub(crate) fn is_any_fs_permission(permission: &str) -> bool {
    matches!(normalize_permission(permission).as_str(), "fs.data" | "fs.server" | "fs.global")
        || is_valid_fs_action_permission(permission)
}

pub(crate) fn has_plugin_folder_access_permission(permissions: &[String]) -> bool {
    permissions
        .iter()
        .any(|permission| normalize_permission(permission) == PLUGIN_FOLDER_ACCESS_PERMISSION)
}

pub(crate) fn valid_permission_summary() -> &'static str {
    "log, fs, fs.data, fs.server, fs.global, fs.{scope}.{action}, api, storage, network, system, server, console, ui, element, execute_program, plugin_folder_access/plugins, ui.component.{read,write,proxy,create}"
}

fn is_valid_fs_action_permission(permission: &str) -> bool {
    let mut parts = permission.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("fs"), Some(scope), Some(action), None)
            if FS_SCOPES.contains(&scope) && FS_ACTIONS.contains(&action)
    )
}
