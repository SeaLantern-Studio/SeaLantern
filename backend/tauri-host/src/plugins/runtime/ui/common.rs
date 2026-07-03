// 公共助手函数

use crate::plugins::runtime::host_api::{
    host_emit_component_event, host_emit_permission_log, host_emit_ui_event, host_t_with_options,
};
use crate::utils::logger::log_error_ctx;
use mlua::{Function, Lua, String as LuaString, Table, Value};
use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;

pub(super) const VALID_INSERT_PLACEMENTS: &[&str] = &["before", "after", "prepend", "append"];
pub(super) const VALID_CONTEXT_MENU_CONTEXTS: &[&str] =
    &["server-list", "console", "plugin-list", "player-list", "global"];

pub(super) fn ui_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    host_t_with_options(key, &m)
}

pub(super) fn ui_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    host_t_with_options(key, &m)
}

pub(super) fn lua_str(s: LuaString) -> String {
    String::from_utf8_lossy(&s.as_bytes()).into_owned()
}

pub(super) fn lua_opt_str(s: Option<LuaString>) -> Option<String> {
    s.map(lua_str)
}

pub(super) fn lua_value_to_json(value: Value) -> JsonValue {
    match value {
        Value::Nil => JsonValue::Null,
        Value::Boolean(b) => JsonValue::Bool(b),
        Value::Integer(i) => serde_json::json!(i),
        Value::Number(n) => serde_json::json!(n),
        Value::String(s) => JsonValue::String(lua_str(s)),
        Value::Table(table) => table_to_json(table),
        _ => JsonValue::Null,
    }
}

pub(super) fn table_to_json(table: Table) -> JsonValue {
    let mut map = Map::new();
    for (key, value) in table.pairs::<LuaString, Value>().flatten() {
        map.insert(lua_str(key), lua_value_to_json(value));
    }
    JsonValue::Object(map)
}

pub(super) fn json_to_string(value: &JsonValue, ctx: &str) -> mlua::Result<String> {
    serde_json::to_string(value).map_err(|e| {
        mlua::Error::runtime(ui_t2("plugins.runtime.ui.serialize_failed", ctx, e.to_string()))
    })
}

pub(super) fn emit_component_action(
    lua: &Lua,
    pid: &str,
    ctx: &str,
    payload: JsonValue,
) -> mlua::Result<bool> {
    let payload = json_to_string(&payload, ctx)?;
    emit_result(lua, pid, ctx, host_emit_component_event(pid, &payload))
}

#[derive(Clone, Copy)]
pub(super) struct UiLogSpec<'a> {
    pub api_name: &'a str,
    pub target: &'a str,
}

pub(super) fn emit_ui_action(
    lua: &Lua,
    pid: &str,
    ctx: &str,
    event: &str,
    target: &str,
    payload: &str,
    log_spec: Option<UiLogSpec<'_>>,
) -> mlua::Result<bool> {
    if let Some(spec) = log_spec {
        let _ = host_emit_permission_log(pid, "api_call", spec.api_name, spec.target);
    }
    emit_result(lua, pid, ctx, host_emit_ui_event(pid, event, target, payload))
}

pub(super) fn validate_context_menu_context(context: &str) -> mlua::Result<()> {
    if VALID_CONTEXT_MENU_CONTEXTS.contains(&context) {
        Ok(())
    } else {
        Err(mlua::Error::runtime(ui_t2(
            "plugins.runtime.ui.invalid_context_type",
            context,
            format!("{:?}", VALID_CONTEXT_MENU_CONTEXTS),
        )))
    }
}

pub(super) fn register_callback(
    lua: &Lua,
    registry_key: &str,
    callback: Function,
) -> mlua::Result<bool> {
    lua.set_named_registry_value(registry_key, callback)
        .map_err(|e| {
            mlua::Error::runtime(ui_t1("plugins.runtime.ui.store_callback_failed", e.to_string()))
        })?;
    Ok(true)
}

const REG_PREFIX: &str = "_ui_error_mode_";

pub(super) fn set_error_mode(lua: &mlua::Lua, pid: &str, mode: &str) -> mlua::Result<()> {
    let mode = match mode {
        "compat" | "strict" => mode,
        other => {
            return Err(mlua::Error::runtime(ui_t1("plugins.runtime.ui.invalid_error_mode", other)))
        }
    };
    let key = format!("{}{}", REG_PREFIX, pid);
    lua.set_named_registry_value(&key, mode.to_string())
}

fn get_error_mode(lua: &mlua::Lua, pid: &str) -> String {
    let key = format!("{}{}", REG_PREFIX, pid);
    lua.named_registry_value::<String>(&key)
        .unwrap_or_else(|_| "compat".to_string())
}

pub(super) fn emit_result(
    lua: &mlua::Lua,
    pid: &str,
    ctx: &str,
    result: Result<(), String>,
) -> mlua::Result<bool> {
    match result {
        Ok(()) => Ok(true),
        Err(e) => {
            log_error_ctx(
                "plugins.runtime.ui.common",
                "emit_result",
                &ui_t2("plugins.runtime.ui.error_log", ctx, e.clone()),
            );
            if get_error_mode(lua, pid) == "strict" {
                Err(mlua::Error::runtime(ui_t2("plugins.runtime.ui.action_failed", ctx, e)))
            } else {
                Ok(false)
            }
        }
    }
}

pub(super) fn map_create_err<T>(res: mlua::Result<T>, fullname: &str) -> Result<T, String> {
    res.map_err(|e| ui_t2("plugins.runtime.ui.create_failed", fullname, e.to_string()))
}

pub(super) fn map_set_err(res: mlua::Result<()>, fullname: &str) -> Result<(), String> {
    res.map_err(|e| ui_t2("plugins.runtime.ui.set_failed", fullname, e.to_string()))
}

pub(super) fn register_single_string_ui_action(
    lua: &Lua,
    ui_table: &Table,
    pid: &str,
    table_key: &str,
    event: &'static str,
    ctx: &'static str,
    api_name: &'static str,
) -> Result<(), String> {
    let pid = pid.to_string();
    let function = map_create_err(
        lua.create_function(move |lua, target: mlua::String| {
            let target = lua_str(target);
            emit_ui_action(
                lua,
                &pid,
                ctx,
                event,
                &target,
                "",
                Some(UiLogSpec { api_name, target: &target }),
            )
        }),
        &format!("ui.{}", table_key),
    )?;

    map_set_err(ui_table.set(table_key, function), &format!("ui.{}", table_key))
}

#[cfg(test)]
#[path = "../../../../tests/unit/plugins_runtime_ui_common_tests.rs"]
mod tests;
