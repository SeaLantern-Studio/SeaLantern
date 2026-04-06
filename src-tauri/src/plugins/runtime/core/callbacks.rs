use super::runtime::PluginRuntime;
use crate::plugins::runtime::shared::{json_value_from_lua, lua_value_from_json};
use mlua::{Function, MultiValue, Table, Value};
use serde_json::Value as JsonValue;

impl PluginRuntime {
    pub fn call_registered_api(
        &self,
        api_name: &str,
        args: Vec<JsonValue>,
    ) -> Result<JsonValue, String> {
        let globals = self.lua.globals();

        let apis: Table = globals
            .get("_SL_APIS")
            .map_err(|e| format!("获取 _SL_APIS 失败: {}", e))?;

        let func: Function = apis
            .get(api_name.to_string())
            .map_err(|e| format!("API '{}' 不存在: {}", api_name, e))?;

        let mut lua_args = Vec::new();
        for arg in args {
            let lua_val = lua_value_from_json(&self.lua, &arg, 0)
                .map_err(|e| format!("参数转换失败: {}", e))?;
            lua_args.push(lua_val);
        }

        let result: Value = func
            .call(MultiValue::from_vec(lua_args))
            .map_err(|e| format!("调用 API '{}' 失败: {}", api_name, e))?;

        json_value_from_lua(&result, 0).map_err(|e| format!("结果转换失败: {}", e))
    }

    pub fn call_context_menu_hide_callback(&self) -> Result<(), String> {
        let registry_key = format!("_context_menu_hide_callback_{}", self.plugin_id);
        let callback: Function = self
            .lua
            .named_registry_value(&registry_key)
            .map_err(|e| format!("获取右键菜单隐藏回调函数失败: {}", e))?;
        callback
            .call::<()>(())
            .map_err(|e| format!("调用右键菜单隐藏回调失败: {}", e))?;
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

        let callback: Function = self
            .lua
            .named_registry_value(&registry_key)
            .map_err(|e| format!("获取右键菜单显示回调函数失败: {}", e))?;

        let target_lua = lua_value_from_json(&self.lua, &target_data, 0)
            .map_err(|e| format!("转换 target_data 失败: {}", e))?;

        let result: Value = callback
            .call((context.to_string(), target_lua, x, y))
            .map_err(|e| format!("调用右键菜单显示回调失败: {}", e))?;

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

        let callback: Function = self
            .lua
            .named_registry_value(&registry_key)
            .map_err(|e| format!("获取右键菜单回调函数失败: {}", e))?;

        let target_lua = lua_value_from_json(&self.lua, &target_data, 0)
            .map_err(|e| format!("转换 target_data 失败: {}", e))?;

        callback
            .call::<()>((context.to_string(), item_id.to_string(), target_lua))
            .map_err(|e| format!("调用右键菜单回调失败: {}", e))?;

        Ok(())
    }
}
