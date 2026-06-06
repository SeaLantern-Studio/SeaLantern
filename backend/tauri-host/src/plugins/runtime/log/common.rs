use crate::services::global::i18n_service;
use mlua::{Function, Lua, Table};

#[derive(Clone)]
pub(super) struct LogContext {
    pub(super) plugin_id: String,
    pub(super) lua: Lua,
}

impl LogContext {
    pub(super) fn new(plugin_id: String, lua: Lua) -> Self {
        Self { plugin_id, lua }
    }
}

pub(super) fn map_log_err(key: &str, err: mlua::Error) -> String {
    format!("{}: {}", i18n_service().t(key), err)
}

pub(super) fn convert_lua_string(s: &mlua::String) -> String {
    String::from_utf8_lossy(&s.as_bytes()).into_owned()
}

pub(super) fn create_log_table(lua: &Lua) -> mlua::Result<Table> {
    lua.create_table()
}

pub(super) fn set_log_function(table: &Table, name: &str, function: Function) -> mlua::Result<()> {
    table.set(name, function)
}

pub(super) fn set_log_table(sl: &Table, table: Table) -> mlua::Result<()> {
    sl.set("log", table)
}
