use crate::models::server::ServerInstance;
use crate::plugins::runtime::shared::validate_server_path;
use crate::services::global::{i18n_service, server_manager};
use mlua::{Function, Lua, Table};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// 最大文件大小：128 MiB
pub(super) const MAX_FILE_SIZE: u64 = 128 * 1024 * 1024;

#[derive(Clone)]
pub(super) struct ServerContext {
    pub(super) permissions: Vec<String>,
}

impl ServerContext {
    pub(super) fn new(permissions: Vec<String>) -> Self {
        Self { permissions }
    }
}

pub(super) fn check_server_permission(perms: &[String]) -> Result<(), mlua::Error> {
    if !perms.iter().any(|p| p == "server") {
        return Err(mlua::Error::runtime(i18n_service().t("server.permission_denied")));
    }
    Ok(())
}

pub(super) fn find_server(server_id: &str) -> Result<ServerInstance, mlua::Error> {
    let servers = server_manager().get_server_list();
    servers
        .into_iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| {
            mlua::Error::runtime(i18n_service().t_with_options(
                "server.server_not_found",
                &crate::plugins::runtime::console::i18n_arg("0", server_id),
            ))
        })
}

pub(super) fn map_lua_err(key: &str, e: mlua::Error) -> String {
    format!("{}: {}", i18n_service().t(key), e)
}

pub(super) fn create_server_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| map_lua_err("server.create_table_failed", e))
}

pub(super) fn create_logs_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| map_lua_err("server.create_logs_table_failed", e))
}

pub(super) fn set_server_function(
    server_table: &Table,
    name: &str,
    function: Function,
    error_key: &str,
) -> Result<(), String> {
    server_table
        .set(name, function)
        .map_err(|e| map_lua_err(error_key, e))
}

pub(super) fn set_logs_function(
    logs_table: &Table,
    name: &str,
    function: Function,
    error_key: &str,
) -> Result<(), String> {
    logs_table
        .set(name, function)
        .map_err(|e| map_lua_err(error_key, e))
}

pub(super) fn set_logs_table(server_table: &Table, logs_table: Table) -> Result<(), String> {
    server_table
        .set("logs", logs_table)
        .map_err(|e| map_lua_err("server.set_logs_failed", e))
}

pub(super) fn set_server_table(sl: &Table, server_table: Table) -> Result<(), String> {
    sl.set("server", server_table)
        .map_err(|e| map_lua_err("server.set_server_failed", e))
}

pub(super) fn validated_server_path(
    server: &ServerInstance,
    relative_path: &str,
) -> Result<PathBuf, mlua::Error> {
    let server_dir = PathBuf::from(&server.path);
    validate_server_path(&server_dir, relative_path)
}

pub(super) fn with_server<T, F>(server_id: &str, f: F) -> Result<T, mlua::Error>
where
    F: FnOnce(&ServerInstance) -> Result<T, mlua::Error>,
{
    let server = find_server(server_id)?;
    f(&server)
}

pub(super) fn with_server_path<T, F>(
    server_id: &str,
    relative_path: &str,
    f: F,
) -> Result<T, mlua::Error>
where
    F: FnOnce(&ServerInstance, PathBuf) -> Result<T, mlua::Error>,
{
    with_server(server_id, |server| {
        let full_path = validated_server_path(server, relative_path)?;
        f(server, full_path)
    })
}

pub(super) fn create_server_entry(lua: &Lua, server: &ServerInstance) -> mlua::Result<Table> {
    let entry = lua.create_table()?;
    entry.set("id", server.id.clone())?;
    entry.set("name", server.name.clone())?;
    entry.set("path", server.path.clone())?;
    entry.set("version", server.mc_version.clone())?;
    entry.set("server_type", server.core_type.clone())?;
    Ok(entry)
}

pub(super) fn metadata_err(key: &str, error: &std::io::Error) -> mlua::Error {
    mlua::Error::runtime(
        i18n_service().t_with_options(
            key,
            &crate::plugins::runtime::console::i18n_arg("0", &error.to_string()),
        ),
    )
}

pub(super) fn checked_file_metadata(
    path: &std::path::Path,
) -> Result<std::fs::Metadata, mlua::Error> {
    let metadata =
        fs::metadata(path).map_err(|e| metadata_err("server.failed_to_get_metadata", &e))?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(mlua::Error::runtime(i18n_service().t("server.file_too_large")));
    }
    Ok(metadata)
}

pub(super) fn running_log_pairs(count: usize) -> Vec<(String, Vec<String>)> {
    let running_ids = server_manager().get_running_server_ids();
    let running_set: HashSet<String> = running_ids.into_iter().collect();
    let logs_pairs = crate::services::server_log_pipeline::get_all_logs();

    logs_pairs
        .into_iter()
        .filter(|(server_id, _)| running_set.contains(server_id))
        .map(|(server_id, logs)| {
            let start = logs.len().saturating_sub(count);
            (server_id, logs[start..].to_vec())
        })
        .collect()
}
