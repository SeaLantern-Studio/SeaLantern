use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};

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

pub(super) fn create_http_table(lua: &Lua) -> Result<Table, String> {
    lua.create_table()
        .map_err(|e| format!("Failed to create http table: {}", e))
}

pub(super) fn set_http_function(
    table: &Table,
    name: &str,
    function: mlua::Function,
    api_name: &str,
) -> Result<(), String> {
    table
        .set(name, function)
        .map_err(|e| format!("Failed to set {}: {}", api_name, e))
}

pub(super) fn set_http_table(sl: &Table, table: Table) -> Result<(), String> {
    sl.set("http", table)
        .map_err(|e| format!("Failed to set sl.http: {}", e))
}

pub(super) fn lua_error(lua: &Lua, msg: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_vec(vec![Value::Nil, Value::String(lua.create_string(msg)?)]))
}

pub(super) fn lua_success(_lua: &Lua, table: Table) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_vec(vec![Value::Table(table), Value::Nil]))
}

pub(super) fn create_http_function(
    lua: &Lua,
    ctx: &HttpContext,
    method: HttpMethod,
) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, args: MultiValue| {
        let args_clone = args.clone();
        let body_arg = match method {
            HttpMethod::Post | HttpMethod::Put => args_clone.get(1),
            _ => None,
        };
        super::execute_http_request(lua, &ctx, args, method, body_arg)
    })
    .map_err(|e| format!("Failed to create http.{}: {}", method.as_str(), e))
}
