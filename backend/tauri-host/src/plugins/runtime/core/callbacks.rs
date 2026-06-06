use super::runtime::PluginRuntime;
use crate::plugins::runtime::shared::{json_value_from_lua, lua_value_from_json};
use crate::services::global::i18n_service;
use mlua::{Function, MultiValue, Table, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

fn core_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

fn core_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

impl PluginRuntime {
    pub fn call_registered_api(
        &self,
        api_name: &str,
        args: Vec<JsonValue>,
    ) -> Result<JsonValue, String> {
        let globals = self.lua.globals();

        let apis: Table = globals
            .get("_SL_APIS")
            .map_err(|e| core_t1("plugins.runtime.core.get_apis_failed", e.to_string()))?;

        let func: Function = apis
            .get(api_name.to_string())
            .map_err(|e| core_t2("plugins.runtime.core.api_not_found", api_name, e.to_string()))?;

        let mut lua_args = Vec::new();
        for arg in args {
            let lua_val = lua_value_from_json(&self.lua, &arg, 0)
                .map_err(|e| core_t1("plugins.runtime.core.args_convert_failed", e.to_string()))?;
            lua_args.push(lua_val);
        }

        let result: Value = func.call(MultiValue::from_vec(lua_args)).map_err(|e| {
            core_t2("plugins.runtime.core.call_api_failed", api_name, e.to_string())
        })?;

        json_value_from_lua(&result, 0)
            .map_err(|e| core_t1("plugins.runtime.core.result_convert_failed", e.to_string()))
    }

    pub fn call_context_menu_hide_callback(&self) -> Result<(), String> {
        let registry_key = format!("_context_menu_hide_callback_{}", self.plugin_id);
        let callback: Function = self.lua.named_registry_value(&registry_key).map_err(|e| {
            core_t1("plugins.runtime.core.get_context_menu_hide_callback_failed", e.to_string())
        })?;
        callback.call::<()>(()).map_err(|e| {
            core_t1("plugins.runtime.core.call_context_menu_hide_callback_failed", e.to_string())
        })?;
        Ok(())
    }

    pub fn call_context_menu_show_callback(
        &self,
        context: &str,
        target_data: JsonValue,
        x: f64,
        y: f64,
    ) -> Result<Vec<JsonValue>, String> {
        let registry_key = format!("_context_menu_show_callback_{}", self.plugin_id);

        let callback: Function = self.lua.named_registry_value(&registry_key).map_err(|e| {
            core_t1("plugins.runtime.core.get_context_menu_show_callback_failed", e.to_string())
        })?;

        let target_lua = lua_value_from_json(&self.lua, &target_data, 0).map_err(|e| {
            core_t1("plugins.runtime.core.convert_target_data_failed", e.to_string())
        })?;

        let result: Value = callback
            .call((context.to_string(), target_lua, x, y))
            .map_err(|e| {
                core_t1(
                    "plugins.runtime.core.call_context_menu_show_callback_failed",
                    e.to_string(),
                )
            })?;

        let mut dynamic_items = Vec::new();
        if let Value::Table(tbl) = result {
            for item_val in tbl.sequence_values::<Value>().flatten() {
                if let Ok(JsonValue::Object(mut obj)) = json_value_from_lua(&item_val, 0) {
                    obj.insert("pluginId".to_string(), JsonValue::String(self.plugin_id.clone()));
                    dynamic_items.push(JsonValue::Object(obj));
                }
            }
        }

        Ok(dynamic_items)
    }

    pub fn call_context_menu_callback(
        &self,
        context: &str,
        item_id: &str,
        target_data: JsonValue,
    ) -> Result<(), String> {
        let registry_key = format!("_context_menu_callback_{}", self.plugin_id);

        let callback: Function = self.lua.named_registry_value(&registry_key).map_err(|e| {
            core_t1("plugins.runtime.core.get_context_menu_callback_failed", e.to_string())
        })?;

        let target_lua = lua_value_from_json(&self.lua, &target_data, 0).map_err(|e| {
            core_t1("plugins.runtime.core.convert_target_data_failed", e.to_string())
        })?;

        callback
            .call::<()>((context.to_string(), item_id.to_string(), target_lua))
            .map_err(|e| {
                core_t1("plugins.runtime.core.call_context_menu_callback_failed", e.to_string())
            })?;

        Ok(())
    }
}
