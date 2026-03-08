use super::super::PluginRuntime;
use crate::plugins::api::{emit_permission_log, emit_ui_event};
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.inject_css(style_id, css)
    let pid = runtime.plugin_id.clone();
    let inject_css_fn = runtime
        .lua
        .create_function(move |_, (style_id, css): (mlua::String, mlua::String)| {
            let style_id = String::from_utf8_lossy(&style_id.as_bytes()).into_owned();
            let css = String::from_utf8_lossy(&css.as_bytes()).into_owned();
            let _ = emit_permission_log(&pid, "api_call", "sl.ui.inject_css", &style_id);
            match emit_ui_event(&pid, "inject_css", &style_id, &css) {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] inject_css error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.inject_css: {}", e))?;
    ui_table
        .set("inject_css", inject_css_fn)
        .map_err(|e| format!("Failed to set ui.inject_css: {}", e))?;

    // sl.ui.remove_css(style_id)
    let pid = runtime.plugin_id.clone();
    let remove_css_fn = runtime
        .lua
        .create_function(move |_, style_id: mlua::String| {
            let style_id = String::from_utf8_lossy(&style_id.as_bytes()).into_owned();
            let _ = emit_permission_log(&pid, "api_call", "sl.ui.remove_css", &style_id);
            match emit_ui_event(&pid, "remove_css", &style_id, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] remove_css error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.remove_css: {}", e))?;
    ui_table
        .set("remove_css", remove_css_fn)
        .map_err(|e| format!("Failed to set ui.remove_css: {}", e))?;

    // sl.ui.hide(selector)
    let pid = runtime.plugin_id.clone();
    let hide_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
            let _ = emit_permission_log(&pid, "api_call", "sl.ui.hide", &selector);
            match emit_ui_event(&pid, "hide", &selector, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] hide error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.hide: {}", e))?;
    ui_table
        .set("hide", hide_fn)
        .map_err(|e| format!("Failed to set ui.hide: {}", e))?;

    // sl.ui.show(selector)
    let pid = runtime.plugin_id.clone();
    let show_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
            let _ = emit_permission_log(&pid, "api_call", "sl.ui.show", &selector);
            match emit_ui_event(&pid, "show", &selector, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] show error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.show: {}", e))?;
    ui_table
        .set("show", show_fn)
        .map_err(|e| format!("Failed to set ui.show: {}", e))?;

    // sl.ui.disable(selector)
    let pid = runtime.plugin_id.clone();
    let disable_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
            let _ = emit_permission_log(&pid, "api_call", "sl.ui.disable", &selector);
            match emit_ui_event(&pid, "disable", &selector, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] disable error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.disable: {}", e))?;
    ui_table
        .set("disable", disable_fn)
        .map_err(|e| format!("Failed to set ui.disable: {}", e))?;

    // sl.ui.enable(selector)
    let pid = runtime.plugin_id.clone();
    let enable_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
            let _ = emit_permission_log(&pid, "api_call", "sl.ui.enable", &selector);
            match emit_ui_event(&pid, "enable", &selector, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] enable error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.enable: {}", e))?;
    ui_table
        .set("enable", enable_fn)
        .map_err(|e| format!("Failed to set ui.enable: {}", e))?;

    // sl.ui.insert(placement, selector, html)
    let pid = runtime.plugin_id.clone();
    let insert_fn = runtime
        .lua
        .create_function(
            move |_, (placement, selector, html): (mlua::String, mlua::String, mlua::String)| {
                let placement = String::from_utf8_lossy(&placement.as_bytes()).into_owned();
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let html = String::from_utf8_lossy(&html.as_bytes()).into_owned();

                if !["before", "after", "prepend", "append"].contains(&placement.as_str()) {
                    return Err(mlua::Error::runtime(format!(
                        "无效的 placement 参数: '{}', 必须是 'before', 'after', 'prepend' 或 'append'",
                        placement
                    )));
                }

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.insert", &format!("{} {}", placement, selector));

                let combined = format!("{}|{}", placement, selector);
                match emit_ui_event(&pid, "insert", &combined, &html) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[UI] insert error: {}", e);
                        Ok(false)
                    }
                }
            },
        )
        .map_err(|e| format!("Failed to create ui.insert: {}", e))?;
    ui_table
        .set("insert", insert_fn)
        .map_err(|e| format!("Failed to set ui.insert: {}", e))?;

    // sl.ui.remove(selector)
    let pid = runtime.plugin_id.clone();
    let remove_selector_fn = runtime
        .lua
        .create_function(move |_, selector: mlua::String| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();

            let _ = emit_permission_log(&pid, "api_call", "sl.ui.remove", &selector);
            match emit_ui_event(&pid, "remove_selector", &selector, "") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] remove error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.remove: {}", e))?;
    ui_table
        .set("remove", remove_selector_fn)
        .map_err(|e| format!("Failed to set ui.remove: {}", e))?;

    // sl.ui.set_style(selector, styles)
    let pid = runtime.plugin_id.clone();
    let set_style_fn = runtime
        .lua
        .create_function(move |_, (selector, styles): (mlua::String, Table)| {
            let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();

            let mut style_map = serde_json::Map::new();
            for (key, value) in styles.pairs::<mlua::String, mlua::String>().flatten() {
                let key = String::from_utf8_lossy(&key.as_bytes()).into_owned();
                let value = String::from_utf8_lossy(&value.as_bytes()).into_owned();
                style_map.insert(key, serde_json::Value::String(value));
            }
            let styles_json = serde_json::to_string(&style_map).unwrap_or_default();

            let _ = emit_permission_log(&pid, "api_call", "sl.ui.set_style", &selector);

            match emit_ui_event(&pid, "set_style", &selector, &styles_json) {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] set_style error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.set_style: {}", e))?;
    ui_table
        .set("set_style", set_style_fn)
        .map_err(|e| format!("Failed to set ui.set_style: {}", e))?;

    // sl.ui.set_attribute(selector, attr, value)
    let pid = runtime.plugin_id.clone();
    let set_attribute_fn = runtime
        .lua
        .create_function(
            move |_, (selector, attr, value): (mlua::String, mlua::String, mlua::String)| {
                let selector = String::from_utf8_lossy(&selector.as_bytes()).into_owned();
                let attr = String::from_utf8_lossy(&attr.as_bytes()).into_owned();
                let value = String::from_utf8_lossy(&value.as_bytes()).into_owned();

                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.ui.set_attribute",
                    &format!("{} {}={}", selector, attr, value),
                );
                let attr_json = serde_json::json!({
                    "attribute": attr,
                    "value": value
                })
                .to_string();

                match emit_ui_event(&pid, "set_attribute", &selector, &attr_json) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[UI] set_attribute error: {}", e);
                        Ok(false)
                    }
                }
            },
        )
        .map_err(|e| format!("Failed to create ui.set_attribute: {}", e))?;
    ui_table
        .set("set_attribute", set_attribute_fn)
        .map_err(|e| format!("Failed to set ui.set_attribute: {}", e))?;

    Ok(())
}
