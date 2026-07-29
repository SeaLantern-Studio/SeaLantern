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
