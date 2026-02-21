use super::PluginRuntime;
use mlua::Table;
use std::time::{Duration, Instant};

const ELEMENT_GET_TIMEOUT_MS: u64 = 500;
const POLL_INTERVAL_MS: u64 = 10;

impl PluginRuntime {
    pub(super) fn setup_element_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::{element_response_create, emit_permission_log, emit_ui_event};

        let element_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create element table: {}", e))?;

        let plugin_id = self.plugin_id.clone();

        let pid = plugin_id.clone();
        let get_text_fn = self
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.get_text", &selector);

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_text", &selector, &data) {
                    Ok(()) => {
                        let start = Instant::now();
                        let timeout = Duration::from_millis(ELEMENT_GET_TIMEOUT_MS);

                        loop {
                            match rx.try_recv() {
                                Ok(val) => {
                                    return Ok(mlua::Value::String(
                                        lua.create_string(&val).map_err(mlua::Error::external)?,
                                    ));
                                }
                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                    if start.elapsed() > timeout {
                                        return Ok(mlua::Value::Nil);
                                    }
                                    std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                                }
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    return Ok(mlua::Value::Nil);
                                }
                            }
                        }
                    }
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| format!("Failed to create element.get_text: {}", e))?;
        element_table
            .set("get_text", get_text_fn)
            .map_err(|e| format!("Failed to set element.get_text: {}", e))?;

        let pid = plugin_id.clone();
        let get_value_fn = self
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.get_value", &selector);

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_value", &selector, &data) {
                    Ok(()) => {
                        let start = Instant::now();
                        let timeout = Duration::from_millis(ELEMENT_GET_TIMEOUT_MS);

                        loop {
                            match rx.try_recv() {
                                Ok(val) => {
                                    return Ok(mlua::Value::String(
                                        lua.create_string(&val).map_err(mlua::Error::external)?,
                                    ));
                                }
                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                    if start.elapsed() > timeout {
                                        return Ok(mlua::Value::Nil);
                                    }
                                    std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                                }
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    return Ok(mlua::Value::Nil);
                                }
                            }
                        }
                    }
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| format!("Failed to create element.get_value: {}", e))?;
        element_table
            .set("get_value", get_value_fn)
            .map_err(|e| format!("Failed to set element.get_value: {}", e))?;

        let pid = plugin_id.clone();
        let get_attribute_fn = self
            .lua
            .create_function(move |lua, (selector, attr): (mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let attr = String::from_utf8_lossy(&attr.as_bytes()).into_owned();
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.element.get_attribute",
                    &format!("{} {}", selector, attr),
                );

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "attr": attr, "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_attribute", &selector, &data) {
                    Ok(()) => {
                        let start = Instant::now();
                        let timeout = Duration::from_millis(ELEMENT_GET_TIMEOUT_MS);

                        loop {
                            match rx.try_recv() {
                                Ok(val) => {
                                    return Ok(mlua::Value::String(
                                        lua.create_string(&val).map_err(mlua::Error::external)?,
                                    ));
                                }
                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                    if start.elapsed() > timeout {
                                        return Ok(mlua::Value::Nil);
                                    }
                                    std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                                }
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    return Ok(mlua::Value::Nil);
                                }
                            }
                        }
                    }
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| format!("Failed to create element.get_attribute: {}", e))?;
        element_table
            .set("get_attribute", get_attribute_fn)
            .map_err(|e| format!("Failed to set element.get_attribute: {}", e))?;

        let pid = plugin_id.clone();
        let get_attributes_fn = self
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ =
                    emit_permission_log(&pid, "api_call", "sl.element.get_attributes", &selector);

                let (req_id, rx) = element_response_create();
                let data = serde_json::json!({ "request_id": req_id }).to_string();
                match emit_ui_event(&pid, "element_get_attributes", &selector, &data) {
                    Ok(()) => {
                        let start = Instant::now();
                        let timeout = Duration::from_millis(ELEMENT_GET_TIMEOUT_MS);

                        loop {
                            match rx.try_recv() {
                                Ok(val) => {
                                    return Ok(mlua::Value::String(
                                        lua.create_string(&val).map_err(mlua::Error::external)?,
                                    ));
                                }
                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                    if start.elapsed() > timeout {
                                        return Ok(mlua::Value::Nil);
                                    }
                                    std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                                }
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    return Ok(mlua::Value::Nil);
                                }
                            }
                        }
                    }
                    Err(_) => Ok(mlua::Value::Nil),
                }
            })
            .map_err(|e| format!("Failed to create element.get_attributes: {}", e))?;
        element_table
            .set("get_attributes", get_attributes_fn)
            .map_err(|e| format!("Failed to set element.get_attributes: {}", e))?;

        let pid = plugin_id.clone();
        let click_fn = self
            .lua
            .create_function(move |_, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.click", &selector);
                let data = serde_json::json!({}).to_string();
                match emit_ui_event(&pid, "element_click", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[Element] click error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.click: {}", e))?;
        element_table
            .set("click", click_fn)
            .map_err(|e| format!("Failed to set element.click: {}", e))?;

        let pid = plugin_id.clone();
        let set_value_fn = self
            .lua
            .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let value = String::from_utf8_lossy(&value.as_bytes()).into_owned();
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
                        eprintln!("[Element] set_value error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.set_value: {}", e))?;
        element_table
            .set("set_value", set_value_fn)
            .map_err(|e| format!("Failed to set element.set_value: {}", e))?;

        let pid = plugin_id.clone();
        let check_fn = self
            .lua
            .create_function(move |_, (selector, checked): (mlua::String, bool)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
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
                        eprintln!("[Element] check error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.check: {}", e))?;
        element_table
            .set("check", check_fn)
            .map_err(|e| format!("Failed to set element.check: {}", e))?;

        let pid = plugin_id.clone();
        let select_fn = self
            .lua
            .create_function(move |_, (selector, value): (mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let value = String::from_utf8_lossy(&value.as_bytes()).into_owned();
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
                        eprintln!("[Element] select error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.select: {}", e))?;
        element_table
            .set("select", select_fn)
            .map_err(|e| format!("Failed to set element.select: {}", e))?;

        let pid = plugin_id.clone();
        let focus_fn = self
            .lua
            .create_function(move |_, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.focus", &selector);
                let data = serde_json::json!({}).to_string();
                match emit_ui_event(&pid, "element_focus", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[Element] focus error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.focus: {}", e))?;
        element_table
            .set("focus", focus_fn)
            .map_err(|e| format!("Failed to set element.focus: {}", e))?;

        let pid = plugin_id.clone();
        let blur_fn = self
            .lua
            .create_function(move |_, selector: mlua::String| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ = emit_permission_log(&pid, "api_call", "sl.element.blur", &selector);
                let data = serde_json::json!({}).to_string();
                match emit_ui_event(&pid, "element_blur", &selector, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[Element] blur error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.blur: {}", e))?;
        element_table
            .set("blur", blur_fn)
            .map_err(|e| format!("Failed to set element.blur: {}", e))?;

        let lua_weak = self.lua.clone();
        let pid = plugin_id.clone();
        let on_change_fn = self
            .lua
            .create_function(move |_, (selector, callback): (mlua::String, mlua::Function)| {
                let selector_str = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let _ =
                    emit_permission_log(&pid, "api_call", "sl.element.on_change", &selector_str);

                let registry_key = format!("_element_change_callback_{}_{}", pid, selector_str);
                lua_weak
                    .set_named_registry_value(&registry_key, callback)
                    .map_err(|e| {
                        mlua::Error::runtime(format!("Failed to store callback: {}", e))
                    })?;

                let data = serde_json::json!({ "selector": selector_str }).to_string();
                match emit_ui_event(&pid, "element_on_change", &selector_str, &data) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[Element] on_change error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create element.on_change: {}", e))?;
        element_table
            .set("on_change", on_change_fn)
            .map_err(|e| format!("Failed to set element.on_change: {}", e))?;

        sl.set("element", element_table)
            .map_err(|e| format!("Failed to set sl.element: {}", e))?;

        Ok(())
    }
}
