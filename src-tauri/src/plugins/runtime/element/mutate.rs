use super::common::{
    convert_lua_string, element_create_error, log_element_action_error, set_table_value,
};
use super::PluginRuntime;
use mlua::{LuaSerdeExt, Table, Value};

fn scalar_value_to_string(value: Value) -> mlua::Result<String> {
    match value {
        Value::Nil => Ok(String::new()),
        Value::Boolean(v) => Ok(v.to_string()),
        Value::Integer(v) => Ok(v.to_string()),
        Value::Number(v) => Ok(v.to_string()),
        Value::String(v) => Ok(convert_lua_string(&v)),
        _ => Err(mlua::Error::external("attribute value must be scalar")),
    }
}

fn emit_action(
    plugin_id: &str,
    log_action: &str,
    detail: &str,
    event_action: &str,
    selector: &str,
    data: String,
    action_error_key: &str,
) -> bool {
    use crate::plugins::api::{emit_permission_log, emit_ui_event};

    let _ = emit_permission_log(plugin_id, "api_call", log_action, detail);
    match emit_ui_event(plugin_id, event_action, selector, &data) {
        Ok(()) => true,
        Err(e) => {
            log_element_action_error(action_error_key, &e);
            false
        }
    }
}

pub(super) fn register(runtime: &PluginRuntime, element_table: &Table) -> Result<(), String> {
    let pid = runtime.plugin_id.clone();
    let click_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            Ok(emit_action(
                &pid,
                "sl.element.click",
                &selector,
                "element_click",
                &selector,
                serde_json::json!({}).to_string(),
                "element.click_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_click_failed", &e))?;
    set_table_value(element_table, "click", click_fn, "element.set_click_failed")?;

    let pid = runtime.plugin_id.clone();
    let set_value_fn = runtime
        .lua
        .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
            let selector = convert_lua_string(&selector);
            let value = convert_lua_string(&value);
            Ok(emit_action(
                &pid,
                "sl.element.set_value",
                &format!("{} = {}", selector, value),
                "element_set_value",
                &selector,
                serde_json::json!({ "value": value }).to_string(),
                "element.set_value_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_set_value_failed", &e))?;
    set_table_value(element_table, "set_value", set_value_fn, "element.set_set_value_failed")?;

    let pid = runtime.plugin_id.clone();
    let check_fn = runtime
        .lua
        .create_function(move |_, (selector, checked): (mlua::String, bool)| {
            let selector = convert_lua_string(&selector);
            Ok(emit_action(
                &pid,
                "sl.element.check",
                &format!("{} = {}", selector, checked),
                "element_check",
                &selector,
                serde_json::json!({ "checked": checked }).to_string(),
                "element.check_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_check_failed", &e))?;
    set_table_value(element_table, "check", check_fn, "element.set_check_failed")?;

    let pid = runtime.plugin_id.clone();
    let select_fn = runtime
        .lua
        .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
            let selector = convert_lua_string(&selector);
            let value = convert_lua_string(&value);
            Ok(emit_action(
                &pid,
                "sl.element.select",
                &format!("{} = {}", selector, value),
                "element_select",
                &selector,
                serde_json::json!({ "value": value }).to_string(),
                "element.select_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_select_failed", &e))?;
    set_table_value(element_table, "select", select_fn, "element.set_select_failed")?;

    let pid = runtime.plugin_id.clone();
    let set_attribute_fn = runtime
        .lua
        .create_function(
            move |_, (selector, attribute, value): (mlua::String, mlua::String, Value)| {
                let selector = convert_lua_string(&selector);
                let attribute = convert_lua_string(&attribute);
                let value = scalar_value_to_string(value)?;
                Ok(emit_action(
                    &pid,
                    "sl.element.set_attribute",
                    &format!("{} {}", selector, attribute),
                    "set_attribute",
                    &selector,
                    serde_json::json!({ "attribute": attribute, "value": value }).to_string(),
                    "element.set_attribute_error",
                ))
            },
        )
        .map_err(|e| element_create_error("element.create_set_attribute_failed", &e))?;
    set_table_value(
        element_table,
        "set_attribute",
        set_attribute_fn,
        "element.set_set_attribute_failed",
    )?;

    let pid = runtime.plugin_id.clone();
    let set_style_fn = runtime
        .lua
        .create_function(move |lua, (selector, styles): (mlua::String, Table)| {
            let selector = convert_lua_string(&selector);
            let styles_value = lua.from_value::<serde_json::Value>(Value::Table(styles))?;
            let styles_json =
                serde_json::to_string(&styles_value).map_err(mlua::Error::external)?;
            Ok(emit_action(
                &pid,
                "sl.element.set_style",
                &selector,
                "set_style",
                &selector,
                styles_json,
                "element.set_style_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_set_style_failed", &e))?;
    set_table_value(element_table, "set_style", set_style_fn, "element.set_set_style_failed")?;

    let pid = runtime.plugin_id.clone();
    let form_fill_fn = runtime
        .lua
        .create_function(move |lua, (selector, fields): (mlua::String, Table)| {
            let selector = convert_lua_string(&selector);
            let fields_value = lua.from_value::<serde_json::Value>(Value::Table(fields))?;
            let payload = serde_json::json!({ "fields": fields_value }).to_string();
            Ok(emit_action(
                &pid,
                "sl.form.fill",
                &selector,
                "element_form_fill",
                &selector,
                payload,
                "element.form_fill_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_form_fill_failed", &e))?;
    set_table_value(element_table, "form_fill", form_fill_fn, "element.set_form_fill_failed")?;

    let pid = runtime.plugin_id.clone();
    let focus_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            Ok(emit_action(
                &pid,
                "sl.element.focus",
                &selector,
                "element_focus",
                &selector,
                serde_json::json!({}).to_string(),
                "element.focus_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_focus_failed", &e))?;
    set_table_value(element_table, "focus", focus_fn, "element.set_focus_failed")?;

    let pid = runtime.plugin_id.clone();
    let blur_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            Ok(emit_action(
                &pid,
                "sl.element.blur",
                &selector,
                "element_blur",
                &selector,
                serde_json::json!({}).to_string(),
                "element.blur_error",
            ))
        })
        .map_err(|e| element_create_error("element.create_blur_failed", &e))?;
    set_table_value(element_table, "blur", blur_fn, "element.set_blur_failed")?;

    Ok(())
}
