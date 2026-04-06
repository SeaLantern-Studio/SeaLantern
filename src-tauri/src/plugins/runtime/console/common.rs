use crate::services::global::{i18n_service, server_manager, settings_manager};
use mlua::{Lua, Table};
use std::collections::{HashMap, HashSet};

pub fn i18n_arg(key: &str, value: &str) -> HashMap<String, String> {
    HashMap::from([(key.to_string(), value.to_string())])
}

pub fn i18n_args(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[derive(Clone)]
pub(super) struct ConsoleContext {
    pub(super) plugin_id: String,
}

impl ConsoleContext {
    pub(super) fn new(plugin_id: String) -> Self {
        Self { plugin_id }
    }
}

pub(super) const DEFAULT_LOG_COUNT: usize = 100;
pub(super) const MAX_LOG_COUNT: usize = 1000;

pub(super) fn map_console_err(key: &str, err: impl std::fmt::Display) -> String {
    i18n_service().t_with_options(key, &i18n_arg("0", &err.to_string()))
}

pub(super) fn create_console_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| map_console_err("console.create_table_failed", e))
}

pub(super) fn set_console_function(
    table: &Table,
    name: &str,
    function: mlua::Function,
    error_key: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| i18n_service().t_with_options(error_key, &i18n_arg("0", &e.to_string())))
}

pub(super) fn set_console_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("console", table).map_err(|e| {
        i18n_service().t_with_options("console.set_console_failed", &i18n_arg("0", &e.to_string()))
    })
}

pub(super) fn validate_server_id(server_id: &str) -> Result<(), String> {
    let servers = server_manager().get_server_list();
    if !servers.iter().any(|s| s.id == server_id) {
        return Err(
            i18n_service().t_with_options("console.server_not_found", &i18n_arg("0", server_id))
        );
    }
    Ok(())
}

pub(super) fn with_valid_server<T, F>(server_id: &str, f: F) -> Result<T, mlua::Error>
where
    F: FnOnce() -> Result<T, mlua::Error>,
{
    validate_server_id(server_id).map_err(mlua::Error::runtime)?;
    f()
}

pub(super) fn sanitize_command(command: &str) -> Result<String, String> {
    const FORBIDDEN_CHARS: &[char] = &[
        '|', ';', '\n', '\r', '&', '$', '`', '<', '>', '\t', '\\', '(', ')', '[', ']', '{', '}',
    ];

    if command.contains(FORBIDDEN_CHARS) {
        return Err(i18n_service().t("console.command_has_forbidden_chars"));
    }

    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err(i18n_service().t("console.empty_command"));
    }

    Ok(trimmed.to_string())
}

pub(super) fn is_command_allowed(command: &str) -> Result<String, String> {
    let sanitized = sanitize_command(command)?;
    let cmd_lower = sanitized.to_lowercase();
    let cmd_first = cmd_lower.split_whitespace().next().unwrap_or("");

    let settings = settings_manager().get();
    let allowed: HashSet<&str> = settings
        .plugin_allowed_commands
        .iter()
        .map(|s| s.as_str())
        .collect();
    let blocked: HashSet<&str> = settings
        .plugin_blocked_commands
        .iter()
        .map(|s| s.as_str())
        .collect();

    if blocked.contains(cmd_first) {
        return Err(
            i18n_service().t_with_options("console.command_forbidden", &i18n_arg("0", command))
        );
    }

    if allowed.contains(cmd_first) {
        return Ok(sanitized);
    }

    Err(i18n_service().t_with_options(
        "console.command_not_allowed",
        &i18n_args(&[
            ("0", command),
            ("1", &allowed.iter().copied().collect::<Vec<_>>().join(", ")),
        ]),
    ))
}

pub(super) fn emit_console_log(plugin_id: &str, category: &str, api_name: &str, resource: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, category, api_name, resource);
}
