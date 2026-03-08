use super::super::PluginRuntime;
use crate::plugins::api::{emit_permission_log, emit_ui_event};
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.inject_html(element_id, html)
    let pid = runtime.plugin_id.clone();
    let inject_fn = runtime
        .lua
        .create_function(move |_, (element_id, html): (mlua::String, mlua::String)| {
            let element_id = String::from_utf8_lossy(&element_id.as_bytes()).into_owned();
            let html = String::from_utf8_lossy(&html.as_bytes()).into_owned();

            let _ = emit_permission_log(&pid, "api_call", "sl.ui.inject_html", &element_id);
            match emit_ui_event(&pid, "inject", &element_id, &html) {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] inject_html error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.inject_html: {}", e))?;
    ui_table
        .set("inject_html", inject_fn)
        .map_err(|e| format!("Failed to set ui.inject_html: {}", e))?;

    // sl.ui.remove_html(element_id)
    let pid = runtime.plugin_id.clone();
    let remove_fn = runtime
        .lua
        .create_function(move |_, element_id: mlua::String| {
            let element_id = String::from_utf8_lossy(&element_id.as_bytes()).into_owned();

            let _ = emit_permission_log(&pid, "api_call", "sl.ui.remove_html", &element_id);
            match emit_ui_event(&pid, "remove", &element_id, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] remove_html error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.remove_html: {}", e))?;
    ui_table
        .set("remove_html", remove_fn)
        .map_err(|e| format!("Failed to set ui.remove_html: {}", e))?;

    // sl.ui.update_html(element_id, html)
    let pid = runtime.plugin_id.clone();
    let update_fn = runtime
        .lua
        .create_function(move |_, (element_id, html): (mlua::String, mlua::String)| {
            let element_id = String::from_utf8_lossy(&element_id.as_bytes()).into_owned();
            let html = String::from_utf8_lossy(&html.as_bytes()).into_owned();

            let _ = emit_permission_log(&pid, "api_call", "sl.ui.update_html", &element_id);
            match emit_ui_event(&pid, "update", &element_id, &html) {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] update_html error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.update_html: {}", e))?;
    ui_table
        .set("update_html", update_fn)
        .map_err(|e| format!("Failed to set ui.update_html: {}", e))?;

    // sl.ui.query(selector)
    let pid = runtime.plugin_id.clone();
    let query_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
            match emit_ui_event(&pid, "query", &selector, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] query error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.query: {}", e))?;
    ui_table
        .set("query", query_fn)
        .map_err(|e| format!("Failed to set ui.query: {}", e))?;

    Ok(())
}
