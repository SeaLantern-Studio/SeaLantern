use super::super::PluginRuntime;
use super::common::{
    emit_ui_action, json_to_string, lua_str, map_create_err, map_set_err,
    register_single_string_ui_action, UiLogSpec, VALID_INSERT_PLACEMENTS,
};
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
                emit_ui_action(
                    lua,
                    &pid,
                    "inject_css",
                    "inject_css",
                    &style_id,
                    &css,
                    Some(UiLogSpec {
                        api_name: "sl.ui.inject_css",
                        target: &style_id,
                    }),
                )
            }),
        "ui.inject_css",
    )?;
    map_set_err(ui_table.set("inject_css", inject_css_fn), "ui.inject_css")?;

    register_single_string_ui_action(
        &runtime.lua,
        ui_table,
        &runtime.plugin_id,
        "remove_css",
        "remove_css",
        "remove_css",
        "sl.ui.remove_css",
    )?;
    register_single_string_ui_action(
        &runtime.lua,
        ui_table,
        &runtime.plugin_id,
        "hide",
        "hide",
        "hide",
        "sl.ui.hide",
    )?;
    register_single_string_ui_action(
        &runtime.lua,
        ui_table,
        &runtime.plugin_id,
        "show",
        "show",
        "show",
        "sl.ui.show",
    )?;
    register_single_string_ui_action(
        &runtime.lua,
        ui_table,
        &runtime.plugin_id,
        "disable",
        "disable",
        "disable",
        "sl.ui.disable",
    )?;
    register_single_string_ui_action(
        &runtime.lua,
        ui_table,
        &runtime.plugin_id,
        "enable",
        "enable",
        "enable",
        "sl.ui.enable",
    )?;

    // sl.ui.insert(placement, selector, html)
    let pid = runtime.plugin_id.clone();
    let insert_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (placement, selector, html): (mlua::String, mlua::String, mlua::String)| {
                let placement = lua_str(placement);
                let selector = lua_str(selector);
                let html = lua_str(html);

                if !VALID_INSERT_PLACEMENTS.contains(&placement.as_str()) {
                    return Err(mlua::Error::runtime(format!(
                        "无效的 placement 参数: '{}', 必须是 {:?}",
                        placement, VALID_INSERT_PLACEMENTS
                    )));
                }

                let log_target = format!("{} {}", placement, selector);
                let combined = format!("{}|{}", placement, selector);
                emit_ui_action(
                    lua,
                    &pid,
                    "insert",
                    "insert",
                    &combined,
                    &html,
                    Some(UiLogSpec {
                        api_name: "sl.ui.insert",
                        target: &log_target,
                    }),
                )
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
                emit_ui_action(
                    lua,
                    &pid,
                    "remove",
                    "remove_selector",
                    &selector,
                    "",
                    Some(UiLogSpec {
                        api_name: "sl.ui.remove",
                        target: &selector,
                    }),
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
                    style_map.insert(lua_str(key), serde_json::Value::String(lua_str(value)));
                }
                let styles_json =
                    json_to_string(&serde_json::Value::Object(style_map), "set_style")?;

                emit_ui_action(
                    lua,
                    &pid,
                    "set_style",
                    "set_style",
                    &selector,
                    &styles_json,
                    Some(UiLogSpec {
                        api_name: "sl.ui.set_style",
                        target: &selector,
                    }),
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
                let log_target = format!("{} {}={}", selector, attr, value);
                let attr_json = serde_json::json!({
                    "attribute": attr,
                    "value": value
                })
                .to_string();

                emit_ui_action(
                    lua,
                    &pid,
                    "set_attribute",
                    "set_attribute",
                    &selector,
                    &attr_json,
                    Some(UiLogSpec {
                        api_name: "sl.ui.set_attribute",
                        target: &log_target,
                    }),
                )
            },
        ),
        "ui.set_attribute",
    )?;
    map_set_err(ui_table.set("set_attribute", set_attribute_fn), "ui.set_attribute")?;

    Ok(())
}
