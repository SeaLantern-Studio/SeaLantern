// 公共助手函数

use crate::plugins::api::{emit_component_event, emit_permission_log, emit_ui_event};
use crate::utils::logger::log_error;
use mlua::{Function, Lua, String as LuaString, Table, Value};
use serde_json::{Map, Value as JsonValue};

pub(super) const VALID_INSERT_PLACEMENTS: &[&str] = &["before", "after", "prepend", "append"];
pub(super) const VALID_CONTEXT_MENU_CONTEXTS: &[&str] =
    &["server-list", "console", "plugin-list", "player-list", "global"];

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
    serde_json::to_string(value)
        .map_err(|e| mlua::Error::runtime(format!("序列化 {} 失败: {}", ctx, e)))
}

pub(super) fn emit_component_action(
    lua: &Lua,
    pid: &str,
    ctx: &str,
    payload: JsonValue,
) -> mlua::Result<bool> {
    let payload = json_to_string(&payload, ctx)?;
    emit_result(lua, pid, ctx, emit_component_event(pid, &payload))
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
        let _ = emit_permission_log(pid, "api_call", spec.api_name, spec.target);
    }
    emit_result(lua, pid, ctx, emit_ui_event(pid, event, target, payload))
}

pub(super) fn validate_context_menu_context(context: &str) -> mlua::Result<()> {
    if VALID_CONTEXT_MENU_CONTEXTS.contains(&context) {
        Ok(())
    } else {
        Err(mlua::Error::runtime(format!(
            "无效的上下文类型 '{}', 允许的值: {:?}",
            context, VALID_CONTEXT_MENU_CONTEXTS
        )))
    }
}

pub(super) fn register_callback(
    lua: &Lua,
    registry_key: &str,
    callback: Function,
) -> mlua::Result<bool> {
    lua.set_named_registry_value(registry_key, callback)
        .map_err(|e| mlua::Error::runtime(format!("存储回调函数失败: {}", e)))?;
    Ok(true)
}

const REG_PREFIX: &str = "_ui_error_mode_";

pub(super) fn set_error_mode(lua: &mlua::Lua, pid: &str, mode: &str) -> mlua::Result<()> {
    let mode = match mode {
        "compat" | "strict" => mode,
        other => {
            return Err(mlua::Error::runtime(format!(
                "无效的错误模式: {}（仅支持 'compat' 或 'strict'）",
                other
            )))
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
            log_error(&format!("[UI] {} 错误: {}", ctx, e));
            if get_error_mode(lua, pid) == "strict" {
                Err(mlua::Error::runtime(format!("UI {} 失败: {}", ctx, e)))
            } else {
                Ok(false)
            }
        }
    }
}

pub(super) fn map_create_err<T>(res: mlua::Result<T>, fullname: &str) -> Result<T, String> {
    res.map_err(|e| format!("创建 {} 失败: {}", fullname, e))
}

pub(super) fn map_set_err(res: mlua::Result<()>, fullname: &str) -> Result<(), String> {
    res.map_err(|e| format!("设置 {} 失败: {}", fullname, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_lua_str() {
        let lua = Lua::new();
        let s = lua.create_string(b"hello").unwrap();
        assert_eq!(lua_str(s), "hello".to_string());
    }

    #[test]
    fn test_lua_opt_str() {
        let lua = Lua::new();
        let s = lua.create_string(b"world").unwrap();
        assert_eq!(lua_opt_str(Some(s)), Some("world".to_string()));
        assert_eq!(lua_opt_str(None), None);
    }

    #[test]
    fn test_lua_value_to_json() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set("name", "sea").unwrap();
        table.set("enabled", true).unwrap();

        let value = lua_value_to_json(mlua::Value::Table(table));
        assert_eq!(value["name"], serde_json::Value::String("sea".to_string()));
        assert_eq!(value["enabled"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_emit_result_compat() {
        let lua = Lua::new();
        set_error_mode(&lua, "pid1", "compat").unwrap();
        let ok = emit_result(&lua, "pid1", "ctx", Err("boom".to_string())).unwrap();
        assert!(!ok);
    }

    #[test]
    fn test_emit_result_strict() {
        let lua = Lua::new();
        set_error_mode(&lua, "pid2", "strict").unwrap();
        let res = emit_result(&lua, "pid2", "ctx", Err("boom".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_set_error_mode_invalid() {
        let lua = Lua::new();
        let res = set_error_mode(&lua, "pid3", "bad-mode");
        assert!(res.is_err());
    }
}
