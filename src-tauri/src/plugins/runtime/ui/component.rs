use super::super::PluginRuntime;
use super::common::{
    emit_component_action, lua_opt_str, lua_str, lua_value_to_json, map_create_err, map_set_err,
    table_to_json,
};
use crate::plugins::api::component_mirror_list;
use mlua::Table;

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

fn component_payload(
    pid: &str,
    action: &str,
    fields: impl IntoIterator<Item = (&'static str, serde_json::Value)>,
) -> serde_json::Value {
    let mut payload = serde_json::Map::from_iter([
        ("action".to_string(), serde_json::Value::String(action.to_string())),
        ("plugin_id".to_string(), serde_json::Value::String(pid.to_string())),
    ]);

    for (key, value) in fields {
        payload.insert(key.to_string(), value);
    }

    serde_json::Value::Object(payload)
}

fn register_list(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let list_fn = runtime
        .lua
        .create_function(move |lua, page_filter: Option<mlua::String>| {
            let filter = lua_opt_str(page_filter);
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
    let get_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (cid, prop): (mlua::String, mlua::String)| {
                let payload = component_payload(
                    &pid,
                    "get",
                    [
                        ("component_id", serde_json::Value::String(lua_str(cid))),
                        ("prop", serde_json::Value::String(lua_str(prop))),
                    ],
                );
                emit_component_action(lua, &pid, "component.get", payload)
            }),
        "ui.component.get",
    )?;

    map_set_err(component_table.set("get", get_fn), "ui.component.get")?;

    Ok(())
}

fn register_set(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let set_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (cid, prop, val): (mlua::String, mlua::String, mlua::Value)| {
                let payload = component_payload(
                    &pid,
                    "set",
                    [
                        ("component_id", serde_json::Value::String(lua_str(cid))),
                        ("prop", serde_json::Value::String(lua_str(prop))),
                        ("value", lua_value_to_json(val)),
                    ],
                );
                emit_component_action(lua, &pid, "component.set", payload)
            },
        ),
        "ui.component.set",
    )?;

    map_set_err(component_table.set("set", set_fn), "ui.component.set")?;

    Ok(())
}

fn register_call(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let call_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (cid, method): (mlua::String, mlua::String)| {
                let payload = component_payload(
                    &pid,
                    "call",
                    [
                        ("component_id", serde_json::Value::String(lua_str(cid))),
                        ("method", serde_json::Value::String(lua_str(method))),
                    ],
                );
                emit_component_action(lua, &pid, "component.call", payload)
            }),
        "ui.component.call",
    )?;

    map_set_err(component_table.set("call", call_fn), "ui.component.call")?;

    Ok(())
}

fn register_on(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let on_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (cid, event): (mlua::String, mlua::String)| {
                let payload = component_payload(
                    &pid,
                    "on",
                    [
                        ("component_id", serde_json::Value::String(lua_str(cid))),
                        ("prop", serde_json::Value::String(lua_str(event))),
                    ],
                );
                emit_component_action(lua, &pid, "component.on", payload)
            }),
        "ui.component.on",
    )?;

    map_set_err(component_table.set("on", on_fn), "ui.component.on")?;

    Ok(())
}

fn register_create(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let create_fn = map_create_err(
        runtime.lua.create_function(
            move |lua,
                  (component_type, component_id, props): (
                mlua::String,
                mlua::String,
                mlua::Table,
            )| {
                let payload = component_payload(
                    &pid,
                    "create",
                    [
                        ("component_type", serde_json::Value::String(lua_str(component_type))),
                        ("component_id", serde_json::Value::String(lua_str(component_id))),
                        ("props", table_to_json(props)),
                    ],
                );
                emit_component_action(lua, &pid, "component.create", payload)
            },
        ),
        "ui.component.create",
    )?;

    map_set_err(component_table.set("create", create_fn), "ui.component.create")?;

    Ok(())
}
