use crate::services::global::i18n_service;
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::collections::HashMap;

use super::request::HttpMethod;

#[derive(Clone)]
pub(super) struct HttpContext {
    pub(super) plugin_id: String,
    pub(super) permissions: Vec<String>,
}

impl HttpContext {
    pub(super) fn new(plugin_id: String, permissions: Vec<String>) -> Self {
        Self { plugin_id, permissions }
    }
}

pub(super) fn http_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(super) fn http_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn http_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn create_http_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| http_t1("plugins.runtime.http.create_table_failed", e.to_string()))
}

pub(super) fn set_http_function(
    table: &Table,
    name: &str,
    function: mlua::Function,
    api_name: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| http_t2("plugins.runtime.http.set_api_failed", api_name, e.to_string()))
}

pub(super) fn set_http_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("http", table)
        .map_err(|e| http_t1("plugins.runtime.http.set_namespace_failed", e.to_string()))
}

pub(super) fn lua_error(lua: &Lua, msg: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_vec(vec![Value::Nil, Value::String(lua.create_string(msg)?)]))
}

pub(super) fn lua_success(table: Table) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_vec(vec![Value::Table(table), Value::Nil]))
}

pub(super) fn create_http_function(
    lua: &Lua,
    ctx: &HttpContext,
    method: HttpMethod,
) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: MultiValue| {
        super::execute_http_request(lua, &ctx, args, method)
    })
    .map_err(|e| {
        http_t2(
            "plugins.runtime.http.create_api_failed",
            format!("http.{}", method.as_str()),
            e.to_string(),
        )
    })
}
