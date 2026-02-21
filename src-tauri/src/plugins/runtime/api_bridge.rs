use super::helpers::{json_value_from_lua, lua_value_from_json};
use super::PluginRuntime;
use crate::plugins::api::{call_api, ApiRegistryOps};
use mlua::{Function, MultiValue, Table, Value};
use serde_json::Value as JsonValue;

impl PluginRuntime {
    pub(super) fn setup_api_namespace(&self, sl: &Table) -> Result<(), String> {
        let api_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create api table: {}", e))?;

        self.lua
            .load("_SL_APIS = {}")
            .exec()
            .map_err(|e| format!("Failed to init _SL_APIS: {}", e))?;

        let plugin_id = self.plugin_id.clone();
        let registry = self.api_registry.clone();

        let pid = plugin_id.clone();
        let reg = registry.clone();
        let register_fn = self
            .lua
            .create_function(move |lua, (name, func): (String, Function)| {
                let globals = lua.globals();
                let apis: Table = globals.get("_SL_APIS")?;
                apis.set(name.clone(), func)?;

                reg.register_api(&pid, &name, &name);

                Ok(())
            })
            .map_err(|e| format!("Failed to create api.register: {}", e))?;
        api_table
            .set("register", register_fn)
            .map_err(|e| format!("Failed to set api.register: {}", e))?;

        let reg = registry.clone();
        let has_fn = self
            .lua
            .create_function(move |_, (target_plugin, api_name): (String, String)| {
                Ok(reg.has_api(&target_plugin, &api_name))
            })
            .map_err(|e| format!("Failed to create api.has: {}", e))?;
        api_table
            .set("has", has_fn)
            .map_err(|e| format!("Failed to set api.has: {}", e))?;

        let reg = registry.clone();
        let list_fn = self
            .lua
            .create_function(move |lua, target_plugin: String| {
                let apis = reg.list_apis(&target_plugin);
                let table = lua.create_table()?;
                for (i, api) in apis.iter().enumerate() {
                    table.set(i + 1, api.clone())?;
                }
                Ok(table)
            })
            .map_err(|e| format!("Failed to create api.list: {}", e))?;
        api_table
            .set("list", list_fn)
            .map_err(|e| format!("Failed to set api.list: {}", e))?;

        let pid = plugin_id.clone();
        let call_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                let mut args_iter = args.into_iter();

                let target_plugin: String = match args_iter.next() {
                    Some(Value::String(s)) => s.to_str()?.to_string(),
                    Some(_) => {
                        return Err(mlua::Error::runtime("第一个参数必须是字符串（目标插件 ID）"))
                    }
                    None => return Err(mlua::Error::runtime("缺少目标插件 ID 参数")),
                };

                let api_name: String = match args_iter.next() {
                    Some(Value::String(s)) => s.to_str()?.to_string(),
                    Some(_) => {
                        return Err(mlua::Error::runtime("第二个参数必须是字符串（API 名称）"))
                    }
                    None => return Err(mlua::Error::runtime("缺少 API 名称参数")),
                };

                let mut json_args: Vec<JsonValue> = Vec::new();
                for val in args_iter {
                    match json_value_from_lua(&val, 0) {
                        Ok(json) => json_args.push(json),
                        Err(e) => return Err(mlua::Error::runtime(format!("参数转换失败: {}", e))),
                    }
                }

                match call_api(&pid, &target_plugin, &api_name, json_args) {
                    Ok(result) => lua_value_from_json(lua, &result, 0)
                        .map_err(|e| mlua::Error::runtime(format!("结果转换失败: {}", e))),
                    Err(e) => {
                        if e.contains("不存在") || e.contains("未启用") || e.contains("没有注册")
                        {
                            Ok(Value::Nil)
                        } else {
                            Err(mlua::Error::runtime(e))
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to create api.call: {}", e))?;
        api_table
            .set("call", call_fn)
            .map_err(|e| format!("Failed to set api.call: {}", e))?;

        sl.set("api", api_table)
            .map_err(|e| format!("Failed to set sl.api: {}", e))?;

        Ok(())
    }
}
