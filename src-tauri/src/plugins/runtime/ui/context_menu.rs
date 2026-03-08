use super::super::PluginRuntime;
use crate::plugins::api::emit_context_menu_event;
use mlua::{Function, Table};

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.register_context_menu(context, items)
    let pid = runtime.plugin_id.clone();
    let register_context_menu_fn = runtime
        .lua
        .create_function(move |_, (context, items): (String, Table)| {
            let valid_contexts = ["server-list", "console", "plugin-list", "player-list", "global"];
            if !valid_contexts.contains(&context.as_str()) {
                return Err(mlua::Error::runtime(format!(
                    "无效的上下文类型 '{}', 允许的值: {:?}",
                    context, valid_contexts
                )));
            }

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

            match emit_context_menu_event(&pid, "register", &context, &items_json) {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] register_context_menu error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.register_context_menu: {}", e))?;
    ui_table
        .set("register_context_menu", register_context_menu_fn)
        .map_err(|e| format!("Failed to set ui.register_context_menu: {}", e))?;

    // sl.ui.unregister_context_menu(context)
    let pid = runtime.plugin_id.clone();
    let unregister_context_menu_fn = runtime
        .lua
        .create_function(move |_, context: String| {
            let valid_contexts = ["server-list", "console", "plugin-list", "player-list", "global"];
            if !valid_contexts.contains(&context.as_str()) {
                return Err(mlua::Error::runtime(format!(
                    "无效的上下文类型 '{}', 允许的值: {:?}",
                    context, valid_contexts
                )));
            }

            match emit_context_menu_event(&pid, "unregister", &context, "[]") {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] unregister_context_menu error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.unregister_context_menu: {}", e))?;
    ui_table
        .set("unregister_context_menu", unregister_context_menu_fn)
        .map_err(|e| format!("Failed to set ui.unregister_context_menu: {}", e))?;

    // sl.ui.on_context_menu_click(callback)
    let lua_weak = runtime.lua.clone();
    let pid = runtime.plugin_id.clone();
    let on_context_menu_click_fn = runtime
        .lua
        .create_function(move |_, callback: Function| {
            let registry_key = format!("_context_menu_callback_{}", pid);
            lua_weak
                .set_named_registry_value(&registry_key, callback)
                .map_err(|e| mlua::Error::runtime(format!("存储回调函数失败: {}", e)))?;
            Ok(true)
        })
        .map_err(|e| format!("Failed to create ui.on_context_menu_click: {}", e))?;
    ui_table
        .set("on_context_menu_click", on_context_menu_click_fn)
        .map_err(|e| format!("Failed to set ui.on_context_menu_click: {}", e))?;

    // sl.ui.on_context_menu_show(callback)
    let lua_weak = runtime.lua.clone();
    let pid = runtime.plugin_id.clone();
    let on_context_menu_show_fn = runtime
        .lua
        .create_function(move |_, callback: Function| {
            let registry_key = format!("_context_menu_show_callback_{}", pid);
            lua_weak
                .set_named_registry_value(&registry_key, callback)
                .map_err(|e| mlua::Error::runtime(format!("存储回调函数失败: {}", e)))?;
            Ok(true)
        })
        .map_err(|e| format!("Failed to create ui.on_context_menu_show: {}", e))?;
    ui_table
        .set("on_context_menu_show", on_context_menu_show_fn)
        .map_err(|e| format!("Failed to set ui.on_context_menu_show: {}", e))?;

    // sl.ui.on_context_menu_hide(callback)
    let lua_weak = runtime.lua.clone();
    let pid = runtime.plugin_id.clone();
    let on_context_menu_hide_fn = runtime
        .lua
        .create_function(move |_, callback: Function| {
            let registry_key = format!("_context_menu_hide_callback_{}", pid);
            lua_weak
                .set_named_registry_value(&registry_key, callback)
                .map_err(|e| mlua::Error::runtime(format!("存储回调函数失败: {}", e)))?;
            Ok(true)
        })
        .map_err(|e| format!("Failed to create ui.on_context_menu_hide: {}", e))?;
    ui_table
        .set("on_context_menu_hide", on_context_menu_hide_fn)
        .map_err(|e| format!("Failed to set ui.on_context_menu_hide: {}", e))?;

    Ok(())
}
