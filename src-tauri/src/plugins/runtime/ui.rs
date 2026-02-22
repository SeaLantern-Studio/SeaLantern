use super::PluginRuntime;
use mlua::{Function, Table};

impl PluginRuntime {
    pub(super) fn setup_ui_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::{emit_permission_log, emit_ui_event};

        let ui_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create ui table: {}", e))?;

        let plugin_id = self.plugin_id.clone();

        let pid = plugin_id.clone();
        let inject_fn = self
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

        let pid = plugin_id.clone();
        let remove_fn = self
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

        let pid = plugin_id.clone();
        let update_fn = self
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

        let pid = plugin_id.clone();
        let register_context_menu_fn = self
            .lua
            .create_function(move |_, (context, items): (String, Table)| {
                use crate::plugins::api::emit_context_menu_event;

                let valid_contexts =
                    ["server-list", "console", "plugin-list", "player-list", "global"];
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

        let pid = plugin_id.clone();
        let unregister_context_menu_fn = self
            .lua
            .create_function(move |_, context: String| {
                use crate::plugins::api::emit_context_menu_event;

                let valid_contexts =
                    ["server-list", "console", "plugin-list", "player-list", "global"];
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

        let lua_weak = self.lua.clone();
        let pid = plugin_id.clone();
        let on_context_menu_click_fn = self
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

        let lua_weak = self.lua.clone();
        let pid = plugin_id.clone();
        let on_context_menu_show_fn = self
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

        let lua_weak = self.lua.clone();
        let pid = plugin_id.clone();
        let on_context_menu_hide_fn = self
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

        let pid = plugin_id.clone();
        let inject_css_fn = self
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

        let pid = plugin_id.clone();
        let remove_css_fn = self
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

        let pid = plugin_id.clone();
        let toast_fn = self
            .lua
            .create_function(
                move |_,
                      (toast_type, message, duration): (
                    mlua::String,
                    mlua::String,
                    Option<u32>,
                )| {
                    let toast_type = String::from_utf8_lossy(&toast_type.as_bytes()).into_owned();
                    let message = String::from_utf8_lossy(&message.as_bytes()).into_owned();
                    let dur = duration.unwrap_or(3000);
                    let _ = emit_permission_log(&pid, "api_call", "sl.ui.toast", &toast_type);
                    let json = serde_json::json!({
                        "type": toast_type,
                        "message": message,
                        "duration": dur
                    })
                    .to_string();
                    match emit_ui_event(&pid, "toast", "toast", &json) {
                        Ok(()) => Ok(true),
                        Err(e) => {
                            eprintln!("[UI] toast error: {}", e);
                            Ok(false)
                        }
                    }
                },
            )
            .map_err(|e| format!("Failed to create ui.toast: {}", e))?;
        ui_table
            .set("toast", toast_fn)
            .map_err(|e| format!("Failed to set ui.toast: {}", e))?;

        let pid = plugin_id.clone();
        let register_sidebar_fn = self
            .lua
            .create_function(move |_, config: Table| {
                use crate::plugins::api::emit_sidebar_event;

                let label: String = config
                    .get("label")
                    .map_err(|_| mlua::Error::runtime("侧边栏配置缺少必需的 'label' 字段"))?;

                let icon: String = config.get("icon").unwrap_or_default();

                match emit_sidebar_event(&pid, "register", &label, &icon) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[UI] register_sidebar error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create ui.register_sidebar: {}", e))?;
        ui_table
            .set("register_sidebar", register_sidebar_fn)
            .map_err(|e| format!("Failed to set ui.register_sidebar: {}", e))?;

        let pid = plugin_id.clone();
        let unregister_sidebar_fn = self
            .lua
            .create_function(move |_, ()| {
                use crate::plugins::api::emit_sidebar_event;

                match emit_sidebar_event(&pid, "unregister", "", "") {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[UI] unregister_sidebar error: {}", e);
                        Ok(false)
                    }
                }
            })
            .map_err(|e| format!("Failed to create ui.unregister_sidebar: {}", e))?;
        ui_table
            .set("unregister_sidebar", unregister_sidebar_fn)
            .map_err(|e| format!("Failed to set ui.unregister_sidebar: {}", e))?;

        let pid = plugin_id.clone();
        let hide_fn = self
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

        let pid = plugin_id.clone();
        let show_fn = self
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

        let pid = plugin_id.clone();
        let disable_fn = self
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

        let pid = plugin_id.clone();
        let enable_fn = self
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

        let pid = plugin_id.clone();
        let insert_fn = self
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

                    let _ = emit_permission_log(
                        &pid,
                        "api_call",
                        "sl.ui.insert",
                        &format!("{} {}", placement, selector),
                    );

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

        let pid = plugin_id.clone();
        let remove_selector_fn = self
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

        let pid = plugin_id.clone();
        let set_style_fn = self
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

        let pid = plugin_id.clone();
        let set_attribute_fn = self
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

        let pid = plugin_id.clone();
        let query_fn = self
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
        {
            use crate::plugins::api::emit_component_event;

            let component_table = self
                .lua
                .create_table()
                .map_err(|e| format!("Failed to create ui.component table: {}", e))?;

            let pid = plugin_id.clone();

            let list_fn = self
                .lua
                .create_function(move |lua, page_filter: Option<mlua::String>| {
                    let filter =
                        page_filter.map(|s| String::from_utf8_lossy(&s.as_bytes()).into_owned());
                    let entries = crate::plugins::api::component_mirror_list(filter.as_deref());
                    let result = lua.create_table()?;
                    for (i, entry) in entries.iter().enumerate() {
                        let item = lua.create_table()?;
                        item.set("id", entry.id.clone())?;
                        item.set("type", entry.component_type.clone())?;
                        result.set(i + 1, item)?;
                    }
                    Ok(result)
                })
                .map_err(|e| format!("Failed to create ui.component.list: {}", e))?;
            component_table
                .set("list", list_fn)
                .map_err(|e| format!("Failed to set ui.component.list: {}", e))?;

            let pid2 = pid.clone();
            let get_fn = self.lua.create_function(move |_, (cid, prop): (mlua::String, mlua::String)| {
                let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
                let prop = String::from_utf8_lossy(&prop.as_bytes()).into_owned();
                let payload = serde_json::json!({ "action": "get", "component_id": cid, "prop": prop, "plugin_id": pid2 });
                let _ = emit_component_event(&pid2, &payload.to_string());
                Ok(mlua::Value::Nil)
            }).map_err(|e| format!("Failed to create ui.component.get: {}", e))?;
            component_table
                .set("get", get_fn)
                .map_err(|e| format!("Failed to set ui.component.get: {}", e))?;
            let pid2 = pid.clone();
            let set_fn = self.lua.create_function(move |_, (cid, prop, val): (mlua::String, mlua::String, mlua::Value)| {
                let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
                let prop = String::from_utf8_lossy(&prop.as_bytes()).into_owned();
                let json_val = match val {
                    mlua::Value::Boolean(b) => serde_json::Value::Bool(b),
                    mlua::Value::Integer(i) => serde_json::json!(i),
                    mlua::Value::Number(n) => serde_json::json!(n),
                    mlua::Value::String(s) => serde_json::Value::String(String::from_utf8_lossy(&s.as_bytes()).into_owned()),
                    _ => serde_json::Value::Null,
                };
                let payload = serde_json::json!({ "action": "set", "component_id": cid, "prop": prop, "value": json_val, "plugin_id": pid2 });
                let _ = emit_component_event(&pid2, &payload.to_string());
                Ok(true)
            }).map_err(|e| format!("Failed to create ui.component.set: {}", e))?;
            component_table
                .set("set", set_fn)
                .map_err(|e| format!("Failed to set ui.component.set: {}", e))?;

            let pid2 = pid.clone();
            let call_fn = self.lua.create_function(move |_, (cid, method): (mlua::String, mlua::String)| {
                let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
                let method = String::from_utf8_lossy(&method.as_bytes()).into_owned();
                let payload = serde_json::json!({ "action": "call", "component_id": cid, "method": method, "plugin_id": pid2 });
                let _ = emit_component_event(&pid2, &payload.to_string());
                Ok(mlua::Value::Nil)
            }).map_err(|e| format!("Failed to create ui.component.call: {}", e))?;
            component_table
                .set("call", call_fn)
                .map_err(|e| format!("Failed to set ui.component.call: {}", e))?;

            let pid2 = pid.clone();
            let on_fn = self.lua.create_function(move |_, (cid, event): (mlua::String, mlua::String)| {
                let cid = String::from_utf8_lossy(&cid.as_bytes()).into_owned();
                let event = String::from_utf8_lossy(&event.as_bytes()).into_owned();
                let payload = serde_json::json!({ "action": "on", "component_id": cid, "prop": event, "plugin_id": pid2 });
                let _ = emit_component_event(&pid2, &payload.to_string());
                Ok(true)
            }).map_err(|e| format!("Failed to create ui.component.on: {}", e))?;
            component_table
                .set("on", on_fn)
                .map_err(|e| format!("Failed to set ui.component.on: {}", e))?;

            let pid2 = pid.clone();
            let create_fn = self
                .lua
                .create_function(
                    move |_,
                          (component_type, component_id, props): (
                        mlua::String,
                        mlua::String,
                        mlua::Table,
                    )| {
                        let component_type =
                            String::from_utf8_lossy(&component_type.as_bytes()).into_owned();
                        let component_id =
                            String::from_utf8_lossy(&component_id.as_bytes()).into_owned();

                        let mut props_map = serde_json::Map::new();
                        for (key, value) in props.pairs::<mlua::String, mlua::Value>().flatten() {
                            let key_str = String::from_utf8_lossy(&key.as_bytes()).into_owned();
                            let json_val = match value {
                                mlua::Value::Boolean(b) => serde_json::Value::Bool(b),
                                mlua::Value::Integer(i) => serde_json::json!(i),
                                mlua::Value::Number(n) => serde_json::json!(n),
                                mlua::Value::String(s) => serde_json::Value::String(
                                    String::from_utf8_lossy(&s.as_bytes()).into_owned(),
                                ),
                                _ => serde_json::Value::Null,
                            };
                            props_map.insert(key_str, json_val);
                        }

                        let payload = serde_json::json!({
                            "action": "create",
                            "component_type": component_type,
                            "component_id": component_id,
                            "props": props_map,
                            "plugin_id": pid2
                        });
                        let _ = emit_component_event(&pid2, &payload.to_string());
                        Ok(true)
                    },
                )
                .map_err(|e| format!("Failed to create ui.component.create: {}", e))?;
            component_table
                .set("create", create_fn)
                .map_err(|e| format!("Failed to set ui.component.create: {}", e))?;

            ui_table
                .set("component", component_table)
                .map_err(|e| format!("Failed to set ui.component: {}", e))?;
        }

        sl.set("ui", ui_table)
            .map_err(|e| format!("Failed to set sl.ui: {}", e))?;
        Ok(())
    }
}
