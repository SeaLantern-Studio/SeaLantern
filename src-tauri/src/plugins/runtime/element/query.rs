use super::common::{
    convert_lua_string, element_create_error, element_set_error, wait_for_element_response,
};
use super::PluginRuntime;
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, element_table: &Table) -> Result<(), String> {
    use crate::plugins::api::{element_response_create, emit_permission_log, emit_ui_event};

    let plugin_id = runtime.plugin_id.clone();

    let pid = plugin_id.clone();
    let get_text_fn = runtime
        .lua
        .create_function(move |lua, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.get_text", &selector);

            let (req_id, rx) = element_response_create();
            let data = serde_json::json!({ "request_id": req_id }).to_string();
            match emit_ui_event(&pid, "element_get_text", &selector, &data) {
                Ok(()) => wait_for_element_response(lua, rx),
                Err(_) => Ok(mlua::Value::Nil),
            }
        })
        .map_err(|e| element_create_error("element.create_get_text_failed", &e))?;
    element_table
        .set("get_text", get_text_fn)
        .map_err(|e| element_set_error("element.set_get_text_failed", &e))?;

    let pid = plugin_id.clone();
    let get_value_fn = runtime
        .lua
        .create_function(move |lua, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.get_value", &selector);

            let (req_id, rx) = element_response_create();
            let data = serde_json::json!({ "request_id": req_id }).to_string();
            match emit_ui_event(&pid, "element_get_value", &selector, &data) {
                Ok(()) => wait_for_element_response(lua, rx),
                Err(_) => Ok(mlua::Value::Nil),
            }
        })
        .map_err(|e| element_create_error("element.create_get_value_failed", &e))?;
    element_table
        .set("get_value", get_value_fn)
        .map_err(|e| element_set_error("element.set_get_value_failed", &e))?;

    let pid = plugin_id.clone();
    let get_attribute_fn = runtime
        .lua
        .create_function(move |lua, (selector, attr): (mlua::String, mlua::String)| {
            let selector = convert_lua_string(&selector);
            let attr = convert_lua_string(&attr);
            let _ = emit_permission_log(
                &pid,
                "api_call",
                "sl.element.get_attribute",
                &format!("{} {}", selector, attr),
            );

            let (req_id, rx) = element_response_create();
            let data = serde_json::json!({ "attr": attr, "request_id": req_id }).to_string();
            match emit_ui_event(&pid, "element_get_attribute", &selector, &data) {
                Ok(()) => wait_for_element_response(lua, rx),
                Err(_) => Ok(mlua::Value::Nil),
            }
        })
        .map_err(|e| element_create_error("element.create_get_attribute_failed", &e))?;
    element_table
        .set("get_attribute", get_attribute_fn)
        .map_err(|e| element_set_error("element.set_get_attribute_failed", &e))?;

    let pid = plugin_id.clone();
    let get_attributes_fn = runtime
        .lua
        .create_function(move |lua, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.get_attributes", &selector);

            let (req_id, rx) = element_response_create();
            let data = serde_json::json!({ "request_id": req_id }).to_string();
            match emit_ui_event(&pid, "element_get_attributes", &selector, &data) {
                Ok(()) => wait_for_element_response(lua, rx),
                Err(_) => Ok(mlua::Value::Nil),
            }
        })
        .map_err(|e| element_create_error("element.create_get_attributes_failed", &e))?;
    element_table
        .set("get_attributes", get_attributes_fn)
        .map_err(|e| element_set_error("element.set_get_attributes_failed", &e))?;

    Ok(())
}
