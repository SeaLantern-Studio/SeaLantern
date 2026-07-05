use super::shared::{json_value_from_lua, lua_value_from_json};
use super::PluginRuntime;
use crate::plugins::api::ApiRegistryOps;
use crate::plugins::runtime::host_api::{host_call_api, host_t, host_t_with_options};
use mlua::{Function, MultiValue, Table, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

fn api_t(key: &str) -> String {
    host_t(key)
}

fn api_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    host_t_with_options(key, &m)
}

impl PluginRuntime {
    fn map_api_err(key: &str, e: mlua::Error) -> String {
        api_t1(key, e.to_string())
    }

    fn parse_call_args(args: MultiValue) -> Result<(String, String, Vec<JsonValue>), mlua::Error> {
        let mut args_iter = args.into_iter();

        let target_plugin: String = match args_iter.next() {
            Some(Value::String(s)) => s.to_str()?.to_string(),
            Some(_) => {
                return Err(mlua::Error::runtime(api_t(
                    "plugins.runtime.api_bridge.target_plugin_string_required",
                )))
            }
            None => {
                return Err(mlua::Error::runtime(api_t(
                    "plugins.runtime.api_bridge.target_plugin_missing",
                )))
            }
        };

        let api_name: String = match args_iter.next() {
            Some(Value::String(s)) => s.to_str()?.to_string(),
            Some(_) => {
                return Err(mlua::Error::runtime(api_t(
                    "plugins.runtime.api_bridge.api_name_string_required",
                )))
            }
            None => {
                return Err(mlua::Error::runtime(api_t(
                    "plugins.runtime.api_bridge.api_name_missing",
                )))
            }
        };

        let mut json_args: Vec<JsonValue> = Vec::new();
        for val in args_iter {
            match json_value_from_lua(&val, 0) {
                Ok(json) => json_args.push(json),
                Err(e) => {
                    return Err(mlua::Error::runtime(api_t1(
                        "plugins.runtime.api_bridge.args_convert_failed",
                        e.to_string(),
                    )))
                }
            }
        }

        Ok((target_plugin, api_name, json_args))
    }

    pub(super) fn setup_api_namespace(&self, sl: &Table) -> Result<(), String> {
        let api_table = self
            .lua
            .create_table()
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.create_table_failed", e))?;

        self.lua
            .load("_SL_APIS = {}")
            .exec()
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.init_registry_failed", e))?;

        let plugin_id = self.plugin_id.clone();
        let registry = self.api_registry.clone();

        // register
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
            .map_err(|e| {
                Self::map_api_err("plugins.runtime.api_bridge.create_register_failed", e)
            })?;
        api_table
            .set("register", register_fn)
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.set_register_failed", e))?;

        // has
        let reg = registry.clone();
        let has_fn = self
            .lua
            .create_function(move |_, (target_plugin, api_name): (String, String)| {
                Ok(reg.has_api(&target_plugin, &api_name))
            })
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.create_has_failed", e))?;
        api_table
            .set("has", has_fn)
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.set_has_failed", e))?;

        // list
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
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.create_list_failed", e))?;
        api_table
            .set("list", list_fn)
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.set_list_failed", e))?;

        // call
        let pid = plugin_id.clone();
        let call_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                let (target_plugin, api_name, json_args) = Self::parse_call_args(args)?;

                match host_call_api(&pid, &target_plugin, &api_name, json_args) {
                    Ok(result) => lua_value_from_json(lua, &result, 0).map_err(|e| {
                        mlua::Error::runtime(api_t1(
                            "plugins.runtime.api_bridge.result_convert_failed",
                            e.to_string(),
                        ))
                    }),
                    Err(e) => {
                        if e.contains("不存在")
                            || e.contains("未启用")
                            || e.contains("没有注册")
                            || e.contains("not found")
                            || e.contains("not enabled")
                            || e.contains("did not register")
                        {
                            Ok(Value::Nil)
                        } else {
                            Err(mlua::Error::runtime(e))
                        }
                    }
                }
            })
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.create_call_failed", e))?;
        api_table
            .set("call", call_fn)
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.set_call_failed", e))?;

        sl.set("api", api_table)
            .map_err(|e| Self::map_api_err("plugins.runtime.api_bridge.set_namespace_failed", e))?;

        Ok(())
    }
}
