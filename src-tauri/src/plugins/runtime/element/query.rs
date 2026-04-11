use super::common::{
    convert_lua_string, json_string_to_lua_value, map_element_error, set_table_value,
    wait_for_element_response, wait_for_element_response_with,
};
use super::PluginRuntime;
use mlua::{Lua, Table, Value};

struct StringQueryRegistration<'a, F>
where
    F: Fn(&str) -> String + Clone + Send + 'static,
{
    name: &'a str,
    log_action: &'static str,
    event_action: &'static str,
    create_error_key: &'static str,
    set_error_key: &'static str,
    detail_fn: F,
}

fn emit_query(
    pid: &str,
    selector: &str,
    action: &str,
    data: String,
) -> Result<std::sync::mpsc::Receiver<String>, String> {
    use crate::plugins::api::{element_response_create, emit_ui_event};

    let (req_id, rx) = element_response_create();
    let payload = if data.is_empty() {
        serde_json::json!({ "request_id": req_id }).to_string()
    } else {
        let mut parsed = serde_json::from_str::<serde_json::Value>(&data)
            .unwrap_or_else(|_| serde_json::json!({}));
        if let serde_json::Value::Object(ref mut map) = parsed {
            map.insert("request_id".to_string(), serde_json::json!(req_id));
        }
        parsed.to_string()
    };

    emit_ui_event(pid, action, selector, &payload)?;
    Ok(rx)
}

fn register_string_query<F>(
    runtime: &PluginRuntime,
    element_table: &Table,
    registration: StringQueryRegistration<'_, F>,
) -> Result<(), String>
where
    F: Fn(&str) -> String + Clone + Send + 'static,
{
    use crate::plugins::api::emit_permission_log;

    let StringQueryRegistration {
        name,
        log_action,
        event_action,
        create_error_key,
        set_error_key,
        detail_fn,
    } = registration;

    let pid = runtime.plugin_id.clone();
    let detail_fn_for_lua = detail_fn.clone();
    let func = runtime
        .lua
        .create_function(move |lua, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ =
                emit_permission_log(&pid, "api_call", log_action, &detail_fn_for_lua(&selector));

            match emit_query(&pid, &selector, event_action, String::new()) {
                Ok(rx) => wait_for_element_response(lua, rx),
                Err(_) => Ok(Value::Nil),
            }
        })
        .map_err(|e| map_element_error(create_error_key, &e))?;

    set_table_value(element_table, name, func, set_error_key)
}

pub(super) fn register(runtime: &PluginRuntime, element_table: &Table) -> Result<(), String> {
    use crate::plugins::api::{element_response_create, emit_permission_log, emit_ui_event};

    register_string_query(
        runtime,
        element_table,
        StringQueryRegistration {
            name: "get_text",
            log_action: "sl.element.get_text",
            event_action: "element_get_text",
            create_error_key: "element.create_get_text_failed",
            set_error_key: "element.set_get_text_failed",
            detail_fn: |selector| selector.to_string(),
        },
    )?;

    register_string_query(
        runtime,
        element_table,
        StringQueryRegistration {
            name: "get_value",
            log_action: "sl.element.get_value",
            event_action: "element_get_value",
            create_error_key: "element.create_get_value_failed",
            set_error_key: "element.set_get_value_failed",
            detail_fn: |selector| selector.to_string(),
        },
    )?;

    register_string_query(
        runtime,
        element_table,
        StringQueryRegistration {
            name: "exists",
            log_action: "sl.element.exists",
            event_action: "element_exists",
            create_error_key: "element.create_exists_failed",
            set_error_key: "element.set_exists_failed",
            detail_fn: |selector| selector.to_string(),
        },
    )?;

    register_string_query(
        runtime,
        element_table,
        StringQueryRegistration {
            name: "is_visible",
            log_action: "sl.element.is_visible",
            event_action: "element_is_visible",
            create_error_key: "element.create_is_visible_failed",
            set_error_key: "element.set_is_visible_failed",
            detail_fn: |selector| selector.to_string(),
        },
    )?;

    register_string_query(
        runtime,
        element_table,
        StringQueryRegistration {
            name: "is_enabled",
            log_action: "sl.element.is_enabled",
            event_action: "element_is_enabled",
            create_error_key: "element.create_is_enabled_failed",
            set_error_key: "element.set_is_enabled_failed",
            detail_fn: |selector| selector.to_string(),
        },
    )?;

    let pid = runtime.plugin_id.clone();
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
                Err(_) => Ok(Value::Nil),
            }
        })
        .map_err(|e| map_element_error("element.create_get_attribute_failed", &e))?;
    set_table_value(
        element_table,
        "get_attribute",
        get_attribute_fn,
        "element.set_get_attribute_failed",
    )?;

    let pid = runtime.plugin_id.clone();
    let get_attributes_fn = runtime
        .lua
        .create_function(move |lua: &Lua, selector: mlua::String| {
            let selector = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.get_attributes", &selector);

            let (req_id, rx) = element_response_create();
            let data = serde_json::json!({ "request_id": req_id }).to_string();
            match emit_ui_event(&pid, "element_get_attributes", &selector, &data) {
                Ok(()) => wait_for_element_response_with(lua, rx, |lua, json| {
                    json_string_to_lua_value(lua, json)
                }),
                Err(_) => Ok(Value::Nil),
            }
        })
        .map_err(|e| map_element_error("element.create_get_attributes_failed", &e))?;
    set_table_value(
        element_table,
        "get_attributes",
        get_attributes_fn,
        "element.set_get_attributes_failed",
    )?;

    Ok(())
}
