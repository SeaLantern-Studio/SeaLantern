use super::super::PluginRuntime;
use super::common::{emit_ui_action, lua_str, map_create_err, map_set_err, UiLogSpec};
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.inject_html(element_id, html)
    let pid = runtime.plugin_id.clone();
    let inject_fn = map_create_err(
        runtime.lua.create_function(
            move |lua, (element_id, html): (mlua::String, mlua::String)| {
                let element_id = lua_str(element_id);
                let html = lua_str(html);

                emit_ui_action(
                    lua,
                    &pid,
                    "inject_html",
                    "inject",
                    &element_id,
                    &html,
                    Some(UiLogSpec {
                        api_name: "sl.ui.inject_html",
                        target: &element_id,
                    }),
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

                emit_ui_action(
                    lua,
                    &pid,
                    "remove_html",
                    "remove",
                    &element_id,
                    "",
                    Some(UiLogSpec {
                        api_name: "sl.ui.remove_html",
                        target: &element_id,
                    }),
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

                emit_ui_action(
                    lua,
                    &pid,
                    "update_html",
                    "update",
                    &element_id,
                    &html,
                    Some(UiLogSpec {
                        api_name: "sl.ui.update_html",
                        target: &element_id,
                    }),
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
                emit_ui_action(lua, &pid, "query", "query", &selector, "", None)
            }),
        "ui.query",
    )?;
    map_set_err(ui_table.set("query", query_fn), "ui.query")?;

    Ok(())
}
