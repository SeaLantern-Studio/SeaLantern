use super::super::PluginRuntime;
use super::common::{
    emit_component_action, lua_opt_str, lua_str, lua_value_to_json, map_create_err, map_set_err,
    table_to_json, ui_t2,
};
use crate::plugins::api::component_mirror_list;
use crate::plugins::runtime::permissions::UI_PERMISSION;
use mlua::Table;

fn require_component_permission(permissions: &[String], fine_permission: &str) -> mlua::Result<()> {
    if permissions
        .iter()
        .any(|permission| permission == fine_permission || permission == UI_PERMISSION)
    {
        Ok(())
    } else {
        Err(mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
            "plugins.runtime.permissions.permission_required",
            &std::collections::HashMap::from([(
                "0".to_string(),
                format!("{} | {}", fine_permission, UI_PERMISSION),
            )]),
        )))
    }
}

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    let component_table = runtime
        .lua
        .create_table()
        .map_err(|e| ui_t2("plugins.runtime.ui.create_failed", "ui.component", e.to_string()))?;

    register_list(runtime, &component_table)?;
    register_get(runtime, &component_table)?;
    register_set(runtime, &component_table)?;
    register_call(runtime, &component_table)?;
    register_on(runtime, &component_table)?;
    register_create(runtime, &component_table)?;

    ui_table
        .set("component", component_table)
        .map_err(|e| ui_t2("plugins.runtime.ui.set_failed", "ui.component", e.to_string()))?;

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
    let permissions = runtime.permissions.clone();
    let list_fn = runtime
        .lua
        .create_function(move |lua, page_filter: Option<mlua::String>| {
            require_component_permission(&permissions, "ui.component.read")?;
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
        .map_err(|e| {
            ui_t2("plugins.runtime.ui.create_failed", "ui.component.list", e.to_string())
        })?;

    component_table
        .set("list", list_fn)
        .map_err(|e| ui_t2("plugins.runtime.ui.set_failed", "ui.component.list", e.to_string()))?;

    Ok(())
}

fn register_get(runtime: &PluginRuntime, component_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let permissions = runtime.permissions.clone();
    let get_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (cid, prop): (mlua::String, mlua::String)| {
                require_component_permission(&permissions, "ui.component.read")?;
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
    let permissions = runtime.permissions.clone();
    let set_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (cid, prop, val): (mlua::String, mlua::String, mlua::Value)| {
                require_component_permission(&permissions, "ui.component.write")?;
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
    let permissions = runtime.permissions.clone();
    let call_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (cid, method): (mlua::String, mlua::String)| {
                require_component_permission(&permissions, "ui.component.write")?;
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
    let permissions = runtime.permissions.clone();
    let on_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (cid, event): (mlua::String, mlua::String)| {
                require_component_permission(&permissions, "ui.component.write")?;
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
    let permissions = runtime.permissions.clone();
    let create_fn = map_create_err(
        runtime.lua.create_function(
            move |lua,
                  (component_type, component_id, props): (
                mlua::String,
                mlua::String,
                mlua::Table,
            )| {
                require_component_permission(&permissions, "ui.component.create")?;
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
