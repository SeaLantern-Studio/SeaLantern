use super::super::PluginRuntime;
use super::common::{emit_result, lua_str, map_create_err, map_set_err};
use crate::plugins::api::{emit_permission_log, emit_ui_event};
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.inject_css(style_id, css)
    let pid = runtime.plugin_id.clone();
    let inject_css_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (style_id, css): (mlua::String, mlua::String)| {
                let style_id = lua_str(style_id);
                let css = lua_str(css);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.inject_css", &style_id);
                emit_result(
                    lua,
                    &pid,
                    "inject_css",
                    emit_ui_event(&pid, "inject_css", &style_id, &css),
                )
            }),
        "ui.inject_css",
    )?;
    map_set_err(ui_table.set("inject_css", inject_css_fn), "ui.inject_css")?;

    // sl.ui.remove_css(style_id)
    let pid = runtime.plugin_id.clone();
    let remove_css_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, style_id: mlua::String| {
                let style_id = lua_str(style_id);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.remove_css", &style_id);
                emit_result(
                    lua,
                    &pid,
                    "remove_css",
                    emit_ui_event(&pid, "remove_css", &style_id, ""),
                )
            }),
        "ui.remove_css",
    )?;
    map_set_err(ui_table.set("remove_css", remove_css_fn), "ui.remove_css")?;

    // sl.ui.hide(selector)
    let pid = runtime.plugin_id.clone();
    let hide_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = lua_str(selector);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.hide", &selector);
                emit_result(lua, &pid, "hide", emit_ui_event(&pid, "hide", &selector, ""))
            }),
        "ui.hide",
    )?;
    map_set_err(ui_table.set("hide", hide_fn), "ui.hide")?;

    // sl.ui.show(selector)
    let pid = runtime.plugin_id.clone();
    let show_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = lua_str(selector);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.show", &selector);
                emit_result(lua, &pid, "show", emit_ui_event(&pid, "show", &selector, ""))
            }),
        "ui.show",
    )?;
    map_set_err(ui_table.set("show", show_fn), "ui.show")?;

    // sl.ui.disable(selector)
    let pid = runtime.plugin_id.clone();
    let disable_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = lua_str(selector);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.disable", &selector);
                emit_result(lua, &pid, "disable", emit_ui_event(&pid, "disable", &selector, ""))
            }),
        "ui.disable",
    )?;
    map_set_err(ui_table.set("disable", disable_fn), "ui.disable")?;

    // sl.ui.enable(selector)
    let pid = runtime.plugin_id.clone();
    let enable_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = lua_str(selector);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.enable", &selector);
                emit_result(lua, &pid, "enable", emit_ui_event(&pid, "enable", &selector, ""))
            }),
        "ui.enable",
    )?;
    map_set_err(ui_table.set("enable", enable_fn), "ui.enable")?;

    // sl.ui.insert(placement, selector, html)
    let pid = runtime.plugin_id.clone();
    let insert_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (placement, selector, html): (mlua::String, mlua::String, mlua::String)| {
                let placement = lua_str(placement);
                let selector = lua_str(selector);
                let html = lua_str(html);

                if !["before", "after", "prepend", "append"].contains(&placement.as_str()) {
                    return Err(mlua::Error::runtime(format!(
                        "无效的 placement 参数: '{}', 必须是 'before', 'after', 'prepend' 或 'append'",
                        placement
                    )));
                }

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.insert", &format!("{} {}", placement, selector));

                let combined = format!("{}|{}", placement, selector);
                emit_result(lua, &pid, "insert", emit_ui_event(&pid, "insert", &combined, &html))
            },
        ),
        "ui.insert",
    )?;
    map_set_err(ui_table.set("insert", insert_fn), "ui.insert")?;

    // sl.ui.remove(selector)
    let pid = runtime.plugin_id.clone();
    let remove_selector_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = lua_str(selector);

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.remove", &selector);
                emit_result(
                    lua,
                    &pid,
                    "remove",
                    emit_ui_event(&pid, "remove_selector", &selector, ""),
                )
            }),
        "ui.remove",
    )?;
    map_set_err(ui_table.set("remove", remove_selector_fn), "ui.remove")?;

    // sl.ui.set_style(selector, styles)
    let pid = runtime.plugin_id.clone();
    let set_style_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (selector, styles): (mlua::String, Table)| {
                let selector = lua_str(selector);

                let mut style_map = serde_json::Map::new();
                for (key, value) in styles.pairs::<mlua::String, mlua::String>().flatten() {
                    let key = lua_str(key);
                    let value = lua_str(value);
                    style_map.insert(key, serde_json::Value::String(value));
                }
                let styles_json = serde_json::to_string(&style_map).unwrap_or_default();

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.set_style", &selector);

                emit_result(
                    lua,
                    &pid,
                    "set_style",
                    emit_ui_event(&pid, "set_style", &selector, &styles_json),
                )
            }),
        "ui.set_style",
    )?;
    map_set_err(ui_table.set("set_style", set_style_fn), "ui.set_style")?;

    // sl.ui.set_attribute(selector, attr, value)
    let pid = runtime.plugin_id.clone();
    let set_attribute_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (selector, attr, value): (mlua::String, mlua::String, mlua::String)| {
                let selector = lua_str(selector);
                let attr = lua_str(attr);
                let value = lua_str(value);

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

                emit_result(
                    lua,
                    &pid,
                    "set_attribute",
                    emit_ui_event(&pid, "set_attribute", &selector, &attr_json),
                )
            },
        ),
        "ui.set_attribute",
    )?;
    map_set_err(ui_table.set("set_attribute", set_attribute_fn), "ui.set_attribute")?;

    Ok(())
}
