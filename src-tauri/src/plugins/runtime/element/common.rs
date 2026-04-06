use crate::services::global::i18n_service;
use mlua::{IntoLua, Lua, Table, Value};
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub(super) const ELEMENT_GET_TIMEOUT_MS: u64 = 500;
pub(super) const ELEMENT_CALLBACK_REGISTRY_PREFIX: &str = "_element_change_callback_";

pub(super) fn convert_lua_string(s: &mlua::String) -> String {
    String::from_utf8_lossy(&s.as_bytes()).into_owned()
}

pub(super) fn element_watch_registry_key(plugin_id: &str, selector: &str) -> String {
    format!("{}{}_{}", ELEMENT_CALLBACK_REGISTRY_PREFIX, plugin_id, selector)
}

pub(super) fn map_element_error(key: &str, err: &(impl std::fmt::Display + ?Sized)) -> String {
    i18n_service()
        .t_with_options(key, &crate::plugins::runtime::console::i18n_arg("0", &err.to_string()))
}

pub(super) fn log_element_action_error(key: &str, err: &dyn std::fmt::Display) {
    eprintln!("[Element] {}", map_element_error(key, err));
}

pub(super) fn element_create_error(key: &str, err: &mlua::Error) -> String {
    map_element_error(key, err)
}

pub(super) fn set_table_value<T>(
    table: &Table,
    key: &str,
    value: T,
    error_key: &str,
) -> Result<(), String>
where
    T: IntoLua,
{
    table
        .set(key, value)
        .map_err(|e| map_element_error(error_key, &e))
}

pub(super) fn wait_for_element_response(
    lua: &Lua,
    rx: Receiver<String>,
) -> Result<Value, mlua::Error> {
    wait_for_element_response_with(lua, rx, |lua, val| {
        Ok(Value::String(lua.create_string(&val).map_err(mlua::Error::external)?))
    })
}

pub(super) fn wait_for_element_response_with<F>(
    lua: &Lua,
    rx: Receiver<String>,
    map: F,
) -> Result<Value, mlua::Error>
where
    F: FnOnce(&Lua, String) -> Result<Value, mlua::Error>,
{
    match rx.recv_timeout(Duration::from_millis(ELEMENT_GET_TIMEOUT_MS)) {
        Ok(val) => map(lua, val),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(Value::Nil),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(Value::Nil),
    }
}

pub(super) fn json_string_to_lua_value(lua: &Lua, json: String) -> Result<Value, mlua::Error> {
    let parsed = serde_json::from_str::<serde_json::Value>(&json).map_err(mlua::Error::external)?;
    json_to_lua_value(lua, parsed)
}

pub(super) fn json_to_lua_value(lua: &Lua, value: serde_json::Value) -> Result<Value, mlua::Error> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(v) => Ok(Value::Boolean(v)),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = v.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(v) => {
            Ok(Value::String(lua.create_string(&v).map_err(mlua::Error::external)?))
        }
        serde_json::Value::Array(items) => {
            let table = lua.create_table()?;
            for (idx, item) in items.into_iter().enumerate() {
                table.set(idx + 1, json_to_lua_value(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (key, item) in map {
                table.set(key, json_to_lua_value(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}
