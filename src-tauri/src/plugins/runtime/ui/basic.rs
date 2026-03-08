use super::super::PluginRuntime;
use super::common::{emit_result, lua_str, map_create_err, map_set_err};
use crate::plugins::api::{emit_permission_log, emit_ui_event};
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.inject_html(element_id, html)
    let pid = runtime.plugin_id.clone();
    let inject_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (element_id, html): (mlua::String, mlua::String)| {
                let element_id = lua_str(element_id);
                let html = lua_str(html);

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.inject_html", &element_id);
                emit_result(
                    lua,
                    &pid,
                    "inject_html",
                    emit_ui_event(&pid, "inject", &element_id, &html),
                )
            },
        ),
        "ui.inject_html",
    )?;
    map_set_err(ui_table.set("inject_html", inject_fn), "ui.inject_html")?;

    // sl.ui.remove_html(element_id)
    let pid = runtime.plugin_id.clone();
    let remove_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, element_id: mlua::String| {
                let element_id = lua_str(element_id);

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.remove_html", &element_id);
                emit_result(
                    lua,
                    &pid,
                    "remove_html",
                    emit_ui_event(&pid, "remove", &element_id, ""),
                )
            }),
        "ui.remove_html",
    )?;
    map_set_err(ui_table.set("remove_html", remove_fn), "ui.remove_html")?;

    // sl.ui.update_html(element_id, html)
    let pid = runtime.plugin_id.clone();
    let update_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (element_id, html): (mlua::String, mlua::String)| {
                let element_id = lua_str(element_id);
                let html = lua_str(html);

                let _ = emit_permission_log(&pid, "api_call", "sl.ui.update_html", &element_id);
                emit_result(
                    lua,
                    &pid,
                    "update_html",
                    emit_ui_event(&pid, "update", &element_id, &html),
                )
            },
        ),
        "ui.update_html",
    )?;
    map_set_err(ui_table.set("update_html", update_fn), "ui.update_html")?;

    // sl.ui.query(selector)
    let pid = runtime.plugin_id.clone();
    let query_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, selector: mlua::String| {
                let selector = lua_str(selector);
                emit_result(lua, &pid, "query", emit_ui_event(&pid, "query", &selector, ""))
            }),
        "ui.query",
    )?;
    map_set_err(ui_table.set("query", query_fn), "ui.query")?;

    Ok(())
}
