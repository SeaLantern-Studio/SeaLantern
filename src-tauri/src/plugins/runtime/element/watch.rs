use super::common::{
    convert_lua_string, element_watch_registry_key, map_element_error, set_table_value,
};
use super::PluginRuntime;
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, element_table: &Table) -> Result<(), String> {
    use crate::plugins::api::{emit_permission_log, emit_ui_event};

    let plugin_id = runtime.plugin_id.clone();
    let lua_weak = runtime.lua.clone();
    let pid = plugin_id.clone();
    let on_change_fn = runtime
        .lua
        .create_function(move |_, (selector, callback): (mlua::String, mlua::Function)| {
            let selector_str = convert_lua_string(&selector);
            let _ = emit_permission_log(&pid, "api_call", "sl.element.on_change", &selector_str);

            let registry_key = element_watch_registry_key(&pid, &selector_str);
            lua_weak
                .set_named_registry_value(&registry_key, callback)
                .map_err(|e| {
                    mlua::Error::runtime(map_element_error("element.store_callback_failed", &e))
                })?;

            let cleanup_key = registry_key.clone();
            let cleanup_pid = pid.clone();
            let cleanup_selector = selector_str.clone();
            let cleanup_lua = lua_weak.clone();
            let cleanup_fn = lua_weak.create_function(move |lua, ()| {
                lua.unset_named_registry_value(&cleanup_key).map_err(|e| {
                    mlua::Error::runtime(map_element_error("element.cleanup_callback_failed", &e))
                })?;

                let payload = serde_json::json!({ "selector": cleanup_selector }).to_string();
                if let Err(e) =
                    emit_ui_event(&cleanup_pid, "element_off_change", &cleanup_selector, &payload)
                {
                    super::common::log_element_action_error("element.on_change_error", &e);
                }

                cleanup_lua.expire_registry_values();
                Ok(())
            })?;

            let data = serde_json::json!({ "selector": selector_str }).to_string();
            match emit_ui_event(&pid, "element_on_change", &selector_str, &data) {
                Ok(()) => Ok(cleanup_fn),
                Err(e) => {
                    let _ = lua_weak.unset_named_registry_value(&registry_key);
                    super::common::log_element_action_error("element.on_change_error", &e);
                    let noop_fn = lua_weak.create_function(|_, ()| Ok(()))?;
                    Ok(noop_fn)
                }
            }
        })
        .map_err(|e| map_element_error("element.create_on_change_failed", &e))?;
    set_table_value(element_table, "on_change", on_change_fn, "element.set_on_change_failed")?;

    Ok(())
}
