pub(crate) const PLUGIN_FOLDER_ACCESS_PERMISSION: &str = "plugin_folder_access";
pub(crate) const EXECUTE_PROGRAM_PERMISSION: &str = "execute_program";
pub(crate) const PROCESS_EXEC_PERMISSION: &str = "process.exec";
pub(crate) const UI_PERMISSION: &str = "ui";
pub(crate) const NETWORK_PERMISSION: &str = "network";

pub(crate) const PROCESS_INSPECT_PERMISSION: &str = "process.inspect";
pub(crate) const PROCESS_OUTPUT_READ_PERMISSION: &str = "process.output.read";
pub(crate) const PROCESS_KILL_PERMISSION: &str = "process.kill";
pub(crate) const PLUGINS_READ_PERMISSION: &str = "plugins.read";
pub(crate) const PLUGINS_WRITE_PERMISSION: &str = "plugins.write";

use std::collections::HashSet;

const FS_SCOPES: &[&str] = &["data", "server", "global"];
const FS_ACTIONS: &[&str] = &["read", "write", "list", "meta", "delete", "transfer"];

pub(crate) fn normalize_permission(permission: &str) -> String {
    crate::hardcode_data::plugin_permissions::normalize_permission_id(permission)
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
    crate::hardcode_data::plugin_permissions::is_known_permission_or_alias(permission)
}

pub(crate) fn is_any_fs_permission(permission: &str) -> bool {
    matches!(normalize_permission(permission).as_str(), "fs.data" | "fs.server" | "fs.global")
        || is_valid_fs_action_permission(permission)
}

pub(crate) fn has_any_process_permission(permissions: &[String]) -> bool {
    permissions.iter().any(|permission| {
        matches!(
            normalize_permission(permission).as_str(),
            EXECUTE_PROGRAM_PERMISSION
                | PROCESS_EXEC_PERMISSION
                | PROCESS_INSPECT_PERMISSION
                | PROCESS_OUTPUT_READ_PERMISSION
                | PROCESS_KILL_PERMISSION
        )
    })
}

pub(crate) fn has_any_plugins_permission(permissions: &[String]) -> bool {
    permissions.iter().any(|permission| {
        matches!(
            normalize_permission(permission).as_str(),
            PLUGIN_FOLDER_ACCESS_PERMISSION | PLUGINS_READ_PERMISSION | PLUGINS_WRITE_PERMISSION
        )
    })
}

pub(crate) fn valid_permission_summary() -> &'static str {
    "log, fs, fs.data, fs.server, fs.global, fs.{scope}.{action}, api, storage, network, system, server, console, ui, element, execute_program, process.exec, process.inspect, process.output.read, process.kill, plugin_folder_access/plugins, plugins.read, plugins.write, ui.component.{read,write,proxy,create}"
}

fn is_valid_fs_action_permission(permission: &str) -> bool {
    let mut parts = permission.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("fs"), Some(scope), Some(action), None)
            if FS_SCOPES.contains(&scope) && FS_ACTIONS.contains(&action)
    )
}
