use super::super::PluginRuntime;
use crate::plugins::api::{component_mirror_list, emit_component_event};
use mlua::{Table, Value};

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    let component_table = runtime
        .lua
        .create_table()
        .map_err(|e| format!("创建 ui.component 表失败: {}", e))?;

    register_list(runtime, &component_table)?;
    register_get(runtime, &component_table)?;
    register_set(runtime, &component_table)?;
    register_call(runtime, &component_table)?;
    register_on(runtime, &component_table)?;
    register_create(runtime, &component_table)?;

    ui_table
        .set("component", component_table)
        .map_err(|e| format!("设置 ui.component 失败: {}", e))?;

    Ok(())
}

fn register_list(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let list_fn = runtime
        .lua
        .create_function(move |lua, page_filter: Option<mlua::String>| {
            let filter = page_filter.map(|s| String::from_utf8_lossy(&s.as_bytes()).into_owned());
            let entries = component_mirror_list(filter.as_deref());
            let result = lua.create_table()?;
            for (i, entry) in entries.iter().enumerate() {
                let item = lua.create_table()?;
                item.set("id", entry.id.clone())?;
                item.set("type", entry.component_type.clone())?;
                result.set(i + 1, item)?;
            }
            Ok(result)
        })
        .map_err(|e| format!("创建 ui.component.list 失败: {}", e))?;

    component_table
        .set("list", list_fn)
        .map_err(|e| format!("设置 ui.component.list 失败: {}", e))?;

    Ok(())
}

fn register_get(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let get_fn = runtime
        .lua
        .create_function(move |_, (cid, prop): (mlua::String, mlua::String)| {
            let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
            let prop = String::from_utf8_lossy(&prop.as_bytes()).into_owned();
            let payload = serde_json::json!({
                "action": "get",
                "component_id": cid,
                "prop": prop,
                "plugin_id": pid,
            });
            let _ = emit_component_event(&pid, &payload.to_string());
            Ok(Value::Nil)
        })
        .map_err(|e| format!("创建 ui.component.get 失败: {}", e))?;

    component_table
        .set("get", get_fn)
        .map_err(|e| format!("设置 ui.component.get 失败: {}", e))?;

    Ok(())
}

fn register_set(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let set_fn = runtime
        .lua
        .create_function(move |_, (cid, prop, val): (mlua::String, mlua::String, mlua::Value)| {
            let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
            let prop = String::from_utf8_lossy(&prop.as_bytes()).into_owned();
            let json_val = match val {
                mlua::Value::Boolean(b) => serde_json::Value::Bool(b),
                mlua::Value::Integer(i) => serde_json::json!(i),
                mlua::Value::Number(n) => serde_json::json!(n),
                mlua::Value::String(s) => {
                    serde_json::Value::String(String::from_utf8_lossy(&s.as_bytes()).into_owned())
                }
                _ => serde_json::Value::Null,
            };
            let payload = serde_json::json!({
                "action": "set",
                "component_id": cid,
                "prop": prop,
                "value": json_val,
                "plugin_id": pid,
            });
            let _ = emit_component_event(&pid, &payload.to_string());
            Ok(true)
        })
        .map_err(|e| format!("创建 ui.component.set 失败: {}", e))?;

    component_table
        .set("set", set_fn)
        .map_err(|e| format!("设置 ui.component.set 失败: {}", e))?;

    Ok(())
}

fn register_call(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let call_fn = runtime
        .lua
        .create_function(move |_, (cid, method): (mlua::String, mlua::String)| {
            let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
            let method = String::from_utf8_lossy(&method.as_bytes()).into_owned();
            let payload = serde_json::json!({
                "action": "call",
                "component_id": cid,
                "method": method,
                "plugin_id": pid,
            });
            let _ = emit_component_event(&pid, &payload.to_string());
            Ok(Value::Nil)
        })
        .map_err(|e| format!("创建 ui.component.call 失败: {}", e))?;

    component_table
        .set("call", call_fn)
        .map_err(|e| format!("设置 ui.component.call 失败: {}", e))?;

    Ok(())
}

fn register_on(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let on_fn = runtime
        .lua
        .create_function(move |_, (cid, event): (mlua::String, mlua::String)| {
            let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
            let event = String::from_utf8_lossy(&event.as_bytes()).into_owned();
            let payload = serde_json::json!({
                "action": "on",
                "component_id": cid,
                "prop": event,
                "plugin_id": pid,
            });
            let _ = emit_component_event(&pid, &payload.to_string());
            Ok(true)
        })
        .map_err(|e| format!("创建 ui.component.on 失败: {}", e))?;

    component_table
        .set("on", on_fn)
        .map_err(|e| format!("设置 ui.component.on 失败: {}", e))?;

    Ok(())
}

fn register_create(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let create_fn = runtime
        .lua
        .create_function(
            move |_,
                  (component_type, component_id, props): (
                mlua::String,
                mlua::String,
                mlua::Table,
            )| {
                let component_type =
                    String::from_utf8_lossy(&component_type.as_bytes()).into_owned();
                let component_id = String::from_utf8_lossy(&component_id.as_bytes()).into_owned();

                let mut props_map = serde_json::Map::new();
                for (key, value) in props.pairs::<mlua::String, mlua::Value>().flatten() {
                    let key_str = String::from_utf8_lossy(&key.as_bytes()).into_owned();
                    let json_val = match value {
                        mlua::Value::Boolean(b) => serde_json::Value::Bool(b),
                        mlua::Value::Integer(i) => serde_json::json!(i),
                        mlua::Value::Number(n) => serde_json::json!(n),
                        mlua::Value::String(s) => serde_json::Value::String(
                            String::from_utf8_lossy(&s.as_bytes()).into_owned(),
                        ),
                        _ => serde_json::Value::Null,
                    };
                    props_map.insert(key_str, json_val);
                }

                let payload = serde_json::json!({
                    "action": "create",
                    "component_type": component_type,
                    "component_id": component_id,
                    "props": props_map,
                    "plugin_id": pid,
                });
                let _ = emit_component_event(&pid, &payload.to_string());
                Ok(true)
            },
        )
        .map_err(|e| format!("创建 ui.component.create 失败: {}", e))?;

    component_table
        .set("create", create_fn)
        .map_err(|e| format!("设置 ui.component.create 失败: {}", e))?;

    Ok(())
}
