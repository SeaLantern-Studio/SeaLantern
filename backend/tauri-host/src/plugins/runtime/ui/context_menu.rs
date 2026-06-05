use super::super::PluginRuntime;
use super::common::{
    emit_result, map_create_err, map_set_err, register_callback, validate_context_menu_context,
};
use crate::plugins::api::emit_context_menu_event;
use mlua::{Function, Table};

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.register_context_menu(context, items)
    let pid = runtime.plugin_id.clone();
    let register_context_menu_fn = map_create_err(
        runtime
            .lua
            .create_function(move |lua, (context, items): (String, Table)| {
                validate_context_menu_context(&context)?;

                let mut items_vec: Vec<serde_json::Value> = Vec::new();
                for pair in items.pairs::<i64, Table>() {
                    let (_, item) =
                        pair.map_err(|e| mlua::Error::runtime(format!("读取菜单项失败: {}", e)))?;

                    let id: String = item
                        .get("id")
                        .map_err(|_| mlua::Error::runtime("菜单项缺少必需的 'id' 字段"))?;
                    let label: String = item
                        .get("label")
                        .map_err(|_| mlua::Error::runtime("菜单项缺少必需的 'label' 字段"))?;

                    let icon: Option<String> = item.get("icon").ok();

                    let mut item_obj = serde_json::json!({
                        "id": id,
                        "label": label
                    });

                    if let Some(icon_val) = icon {
                        item_obj["icon"] = serde_json::Value::String(icon_val);
                    }

                    items_vec.push(item_obj);
                }

                let items_json = serde_json::to_string(&items_vec)
                    .map_err(|e| mlua::Error::runtime(format!("序列化菜单项失败: {}", e)))?;

                emit_result(
                    lua,
                    &pid,
                    "register_context_menu",
                    emit_context_menu_event(&pid, "register", &context, &items_json),
                )
            }),
        "ui.register_context_menu",
    )?;
    map_set_err(
        ui_table.set("register_context_menu", register_context_menu_fn),
        "ui.register_context_menu",
    )?;

    // sl.ui.unregister_context_menu(context)
    let pid = runtime.plugin_id.clone();
    let unregister_context_menu_fn = map_create_err(
        runtime.lua.create_function(move |lua, context: String| {
            validate_context_menu_context(&context)?;

            emit_result(
                lua,
                &pid,
                "unregister_context_menu",
                emit_context_menu_event(&pid, "unregister", &context, "[]"),
            )
        }),
        "ui.unregister_context_menu",
    )?;
    map_set_err(
        ui_table.set("unregister_context_menu", unregister_context_menu_fn),
        "ui.unregister_context_menu",
    )?;

    // sl.ui.on_context_menu_click(callback)
    let lua_weak = runtime.lua.clone();
    let pid = runtime.plugin_id.clone();
    let on_context_menu_click_fn = map_create_err(
        runtime.lua.create_function(move |_, callback: Function| {
            let registry_key = format!("_context_menu_callback_{}", pid);
            register_callback(&lua_weak, &registry_key, callback)
        }),
        "ui.on_context_menu_click",
    )?;
    map_set_err(
        ui_table.set("on_context_menu_click", on_context_menu_click_fn),
        "ui.on_context_menu_click",
    )?;

    // sl.ui.on_context_menu_show(callback)
    let lua_weak = runtime.lua.clone();
    let pid = runtime.plugin_id.clone();
    let on_context_menu_show_fn = map_create_err(
        runtime.lua.create_function(move |_, callback: Function| {
            let registry_key = format!("_context_menu_show_callback_{}", pid);
            register_callback(&lua_weak, &registry_key, callback)
        }),
        "ui.on_context_menu_show",
    )?;
    map_set_err(
        ui_table.set("on_context_menu_show", on_context_menu_show_fn),
        "ui.on_context_menu_show",
    )?;

    // sl.ui.on_context_menu_hide(callback)
    let lua_weak = runtime.lua.clone();
    let pid = runtime.plugin_id.clone();
    let on_context_menu_hide_fn = map_create_err(
        runtime.lua.create_function(move |_, callback: Function| {
            let registry_key = format!("_context_menu_hide_callback_{}", pid);
            register_callback(&lua_weak, &registry_key, callback)
        }),
        "ui.on_context_menu_hide",
    )?;
    map_set_err(
        ui_table.set("on_context_menu_hide", on_context_menu_hide_fn),
        "ui.on_context_menu_hide",
    )?;

    Ok(())
}
