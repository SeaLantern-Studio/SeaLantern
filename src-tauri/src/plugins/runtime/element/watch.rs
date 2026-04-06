use super::common::{convert_lua_string, element_create_error, element_set_error};
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

            let registry_key = format!("_element_change_callback_{}_{}", pid, selector_str);
            lua_weak
                .set_named_registry_value(&registry_key, callback)
                .map_err(|e| {
                    mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                        "element.store_callback_failed",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })?;

            let cleanup_key = registry_key.clone();
            let cleanup_fn = lua_weak.create_function(move |lua, ()| {
                lua.unset_named_registry_value(&cleanup_key).map_err(|e| {
                    mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
                        "element.cleanup_callback_failed",
                        &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                    ))
                })
            })?;

            let data = serde_json::json!({ "selector": selector_str }).to_string();
            match emit_ui_event(&pid, "element_on_change", &selector_str, &data) {
                Ok(()) => Ok(cleanup_fn),
                Err(e) => {
                    let _ = lua_weak.unset_named_registry_value(&registry_key);
                    eprintln!(
                        "[Element] {}",
                        crate::services::global::i18n_service().t_with_options(
                            "element.on_change_error",
                            &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
                        )
                    );
                    let noop_fn = lua_weak.create_function(|_, ()| Ok(()))?;
                    Ok(noop_fn)
                }
            }
        })
        .map_err(|e| element_create_error("element.create_on_change_failed", &e))?;
    element_table
        .set("on_change", on_change_fn)
        .map_err(|e| element_set_error("element.set_on_change_failed", &e))?;

    Ok(())
}
