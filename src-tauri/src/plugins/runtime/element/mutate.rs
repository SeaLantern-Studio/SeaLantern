use super::common::{
    convert_lua_string, element_create_error, element_set_error, log_element_action_error,
};
use super::PluginRuntime;
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, element_table: &Table) -> Result<(), String> {
    use crate::plugins::api::{emit_permission_log, emit_ui_event};

    let plugin_id = runtime.plugin_id.clone();

    let pid = plugin_id.clone();
    let click_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.click", &selector);
            let data = serde_json::json!({}).to_string();
            match emit_ui_event(&pid, "element_click", &selector, &data) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log_element_action_error("element.click_error", &e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_click_failed", &e))?;
    element_table
        .set("click", click_fn)
        .map_err(|e| element_set_error("element.set_click_failed", &e))?;

    let pid = plugin_id.clone();
    let set_value_fn = runtime
        .lua
        .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
            let selector = convert_lua_string(&selector);
            let value = convert_lua_string(&value);
            let _ = emit_permission_log(
                &pid,
                "api_call",
                "sl.element.set_value",
                &format!("{} = {}", selector, value),
            );
            let data = serde_json::json!({ "value": value }).to_string();
            match emit_ui_event(&pid, "element_set_value", &selector, &data) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log_element_action_error("element.set_value_error", &e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_set_value_failed", &e))?;
    element_table
        .set("set_value", set_value_fn)
        .map_err(|e| element_set_error("element.set_set_value_failed", &e))?;

    let pid = plugin_id.clone();
    let check_fn = runtime
        .lua
        .create_function(move |_, (selector, checked): (mlua::String, bool)| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(
                &pid,
                "api_call",
                "sl.element.check",
                &format!("{} = {}", selector, checked),
            );
            let data = serde_json::json!({ "checked": checked }).to_string();
            match emit_ui_event(&pid, "element_check", &selector, &data) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log_element_action_error("element.check_error", &e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_check_failed", &e))?;
    element_table
        .set("check", check_fn)
        .map_err(|e| element_set_error("element.set_check_failed", &e))?;

    let pid = plugin_id.clone();
    let select_fn = runtime
        .lua
        .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
            let selector = convert_lua_string(&selector);
            let value = convert_lua_string(&value);
            let _ = emit_permission_log(
                &pid,
                "api_call",
                "sl.element.select",
                &format!("{} = {}", selector, value),
            );
            let data = serde_json::json!({ "value": value }).to_string();
            match emit_ui_event(&pid, "element_select", &selector, &data) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log_element_action_error("element.select_error", &e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_select_failed", &e))?;
    element_table
        .set("select", select_fn)
        .map_err(|e| element_set_error("element.set_select_failed", &e))?;

    let pid = plugin_id.clone();
    let focus_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.focus", &selector);
            let data = serde_json::json!({}).to_string();
            match emit_ui_event(&pid, "element_focus", &selector, &data) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log_element_action_error("element.focus_error", &e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_focus_failed", &e))?;
    element_table
        .set("focus", focus_fn)
        .map_err(|e| element_set_error("element.set_focus_failed", &e))?;

    let pid = plugin_id.clone();
    let blur_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.blur", &selector);
            let data = serde_json::json!({}).to_string();
            match emit_ui_event(&pid, "element_blur", &selector, &data) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log_element_action_error("element.blur_error", &e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_blur_failed", &e))?;
    element_table
        .set("blur", blur_fn)
        .map_err(|e| element_set_error("element.set_blur_failed", &e))?;

    Ok(())
}
