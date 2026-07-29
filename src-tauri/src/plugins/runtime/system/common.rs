use crate::services::global::i18n_service;
use mlua::{Lua, Table};

#[derive(Clone)]
pub(super) struct SystemContext {
    pub(super) plugin_id: String,
}

impl SystemContext {
    pub(super) fn new(plugin_id: String) -> Self {
        Self { plugin_id }
    }
}

pub(super) fn emit_system_log(plugin_id: &str, api_name: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", api_name, "");
}

pub(super) fn map_system_err(key: &str, err: impl std::fmt::Display) -> String {
    format!("{}: {}", i18n_service().t(key), err)
}

pub(super) fn create_system_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| map_system_err("system.create_table_failed", e))
}

pub(super) fn set_system_function(
    table: &Table,
    name: &str,
    function: mlua::Function,
    error_key: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| map_system_err(error_key, e))
}

pub(super) fn set_system_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("system", table)
        .map_err(|e| map_system_err("system.set_system_failed", e))
}
