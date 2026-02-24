mod api_bridge;
mod console;
mod element;
mod filesystem;
pub(crate) mod helpers;
mod http;
mod i18n;
mod log;
mod plugins_api;
mod process;
mod server;
mod storage;
mod system;
mod ui;

use crate::plugins::api::ApiRegistry;
use helpers::{json_value_from_lua, lua_value_from_json};
use mlua::{Function, MultiValue, Table, Value};
pub use process::{kill_all_processes, new_process_registry, ProcessRegistry};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct PluginRuntime {
    pub(super) lua: mlua::Lua,
    pub(super) plugin_id: String,
    pub(super) plugin_dir: PathBuf,
    pub(super) data_dir: PathBuf,
    loaded: AtomicBool,

    pub(super) permissions: Vec<String>,

    pub(super) api_registry: ApiRegistry,

    pub(super) storage_lock: Arc<Mutex<()>>,

    pub(super) process_registry: ProcessRegistry,

    #[allow(dead_code)]
    element_callbacks: Arc<Mutex<std::collections::HashMap<u64, mlua::RegistryKey>>>,
}

impl PluginRuntime {
    pub fn new(
        plugin_id: &str,
        plugin_dir: &Path,
        data_dir: &Path,
        api_registry: ApiRegistry,
        permissions: Vec<String>,
    ) -> Result<Self, String> {
        let lua = mlua::Lua::new_with(
            mlua::StdLib::TABLE
                | mlua::StdLib::STRING
                | mlua::StdLib::MATH
                | mlua::StdLib::UTF8
                | mlua::StdLib::COROUTINE,
            mlua::LuaOptions::default(),
        )
        .map_err(|e| format!("Failed to create Lua instance: {}", e))?;

        fs::create_dir_all(data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

        let runtime = Self {
            lua,
            plugin_id: plugin_id.to_string(),
            plugin_dir: plugin_dir.to_path_buf(),
            data_dir: data_dir.to_path_buf(),
            loaded: AtomicBool::new(false),
            permissions,
            api_registry,
            storage_lock: Arc::new(Mutex::new(())),
            process_registry: new_process_registry(),
            element_callbacks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        };

        runtime.setup_sandbox(plugin_id, plugin_dir)?;
        runtime.setup_sl_namespace()?;

        Ok(runtime)
    }

    #[allow(dead_code)]
    pub fn is_loaded(&self) -> bool {
        self.loaded.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub(super) fn check_permission(&self, permission: &str) -> mlua::Result<()> {
        if self.permissions.iter().any(|p| p == permission) {
            Ok(())
        } else {
            Err(mlua::Error::runtime(format!(
                "Permission denied: '{}' permission is required for this operation",
                permission
            )))
        }
    }

    fn setup_sandbox(&self, plugin_id: &str, plugin_dir: &Path) -> Result<(), String> {
        let globals = self.lua.globals();

        globals
            .set("PLUGIN_ID", plugin_id)
            .map_err(|e| format!("Failed to set PLUGIN_ID: {}", e))?;

        globals
            .set("PLUGIN_DIR", plugin_dir.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to set PLUGIN_DIR: {}", e))?;

        self.sandbox_os_table()?;

        self.sandbox_io_table()?;

        self.remove_dangerous_globals()?;

        Ok(())
    }

    fn sandbox_os_table(&self) -> Result<(), String> {
        let globals = self.lua.globals();

        if globals.get::<Value>("os").is_ok() {
            globals
                .set("os", Value::Nil)
                .map_err(|e| format!("Failed to remove os table: {}", e))?;
        }

        Ok(())
    }

    fn sandbox_io_table(&self) -> Result<(), String> {
        let globals = self.lua.globals();

        if globals.get::<Value>("io").is_ok() {
            globals
                .set("io", Value::Nil)
                .map_err(|e| format!("Failed to remove io table: {}", e))?;
        }

        Ok(())
    }

    fn remove_dangerous_globals(&self) -> Result<(), String> {
        let globals = self.lua.globals();

        let dangerous_globals = ["loadfile", "dofile", "load", "require"];
        for func in dangerous_globals {
            globals
                .set(func, Value::Nil)
                .map_err(|e| format!("Failed to remove {}: {}", func, e))?;
        }

        if globals.get::<Value>("debug").is_ok() {
            globals
                .set("debug", Value::Nil)
                .map_err(|e| format!("Failed to remove debug table: {}", e))?;
        }
        if globals.get::<Value>("package").is_ok() {
            globals
                .set("package", Value::Nil)
                .map_err(|e| format!("Failed to remove package table: {}", e))?;
        }

        Ok(())
    }

    fn setup_sl_namespace(&self) -> Result<(), String> {
        let globals = self.lua.globals();

        let sl = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create sl table: {}", e))?;

        let has_log_permission = self.permissions.iter().any(|p| p == "log");
        self.setup_log_namespace(&sl, has_log_permission)?;

        if self.permissions.iter().any(|p| p == "storage") {
            self.setup_storage_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "storage")?;
        }

        if self.permissions.iter().any(|p| p == "fs") {
            self.setup_fs_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "fs")?;
        }

        if self.permissions.iter().any(|p| p == "api") {
            self.setup_api_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "api")?;
        }

        if self.permissions.iter().any(|p| p == "ui") {
            self.setup_ui_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "ui")?;
        }

        if self.permissions.iter().any(|p| p == "element") {
            self.setup_element_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "element")?;
        }

        if self.permissions.iter().any(|p| p == "server") {
            self.setup_server_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "server")?;
        }

        if self.permissions.iter().any(|p| p == "console") {
            self.setup_console_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "console")?;
        }

        if self.permissions.iter().any(|p| p == "system") {
            self.setup_system_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "system")?;
        }

        if self.permissions.iter().any(|p| p == "network") {
            self.setup_http_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "network")?;
        }

        if self.permissions.iter().any(|p| p == "execute_program") {
            self.setup_process_namespace(&sl, Arc::clone(&self.process_registry))?;
        } else {
            self.setup_permission_denied_module(&sl, "execute_program")?;
        }

        if self.permissions.iter().any(|p| p == "plugin_folder_access") {
            self.setup_plugins_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "plugin_folder_access")?;
        }

        self.setup_i18n_namespace(&sl)?;

        globals
            .set("sl", sl)
            .map_err(|e| format!("Failed to set sl global: {}", e))?;

        Ok(())
    }

    fn setup_permission_denied_module(&self, sl: &Table, module_name: &str) -> Result<(), String> {
        let module_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create {} table: {}", module_name, e))?;

        let module_name_owned = module_name.to_string();
        let error_fn = self
            .lua
            .create_function(move |_, ()| -> Result<(), mlua::Error> {
                Err(mlua::Error::runtime(format!(
                    "权限不足: 使用 'sl.{}' 模块需要在 manifest.json 中声明 '{}' 权限",
                    module_name_owned, module_name_owned
                )))
            })
            .map_err(|e| format!("Failed to create error function for {}: {}", module_name, e))?;

        module_table
            .set("_error", error_fn)
            .map_err(|e| format!("Failed to set error for {}: {}", module_name, e))?;

        let module_name_for_meta = module_name.to_string();
        let meta_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create metatable for {}: {}", module_name, e))?;

        let index_fn = self
            .lua
            .create_function(move |_, _key: mlua::Value| -> Result<mlua::Value, mlua::Error> {
                Err(mlua::Error::runtime(format!(
                    "权限不足: 使用 'sl.{}' 模块需要在 manifest.json 中声明 '{}' 权限",
                    module_name_for_meta, module_name_for_meta
                )))
            })
            .map_err(|e| format!("Failed to create __index for {}: {}", module_name, e))?;

        meta_table
            .set(mlua::MetaMethod::Index.name(), index_fn)
            .map_err(|e| format!("Failed to set __index for {}: {}", module_name, e))?;

        module_table.set_metatable(Some(meta_table));

        sl.set(module_name, module_table)
            .map_err(|e| format!("Failed to set sl.{}: {}", module_name, e))?;

        Ok(())
    }

    pub fn load_file(&self, path: &Path) -> Result<(), String> {
        let bytes = fs::read(path).map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;
        let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        let content = String::from_utf8_lossy(bytes).into_owned();

        let result: Value = self
            .lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .eval()
            .map_err(|e| format!("Failed to execute {:?}: {}", path, e))?;

        if let Value::Table(table) = result {
            let globals = self.lua.globals();
            globals
                .set("plugin", table)
                .map_err(|e| format!("Failed to set plugin global: {}", e))?;
        }

        self.loaded.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn call_lifecycle(&self, event: &str) -> Result<(), String> {
        let globals = self.lua.globals();

        if let Ok(plugin_table) = globals.get::<Table>("plugin") {
            if let Ok(func) = plugin_table.get::<Function>(event) {
                func.call::<()>(())
                    .map_err(|e| format!("Failed to call plugin.{}: {}", event, e))?;
                return Ok(());
            }
        }

        if let Ok(func) = globals.get::<Function>(event) {
            func.call::<()>(())
                .map_err(|e| format!("Failed to call {}: {}", event, e))?;
            return Ok(());
        }

        Ok(())
    }

    pub fn call_lifecycle_with_arg(&self, event: &str, arg: &str) -> Result<(), String> {
        let globals = self.lua.globals();

        if let Ok(plugin_table) = globals.get::<Table>("plugin") {
            if let Ok(func) = plugin_table.get::<Function>(event) {
                func.call::<()>(arg.to_string())
                    .map_err(|e| format!("Failed to call plugin.{}: {}", event, e))?;
                return Ok(());
            }
        }

        if let Ok(func) = globals.get::<Function>(event) {
            func.call::<()>(arg.to_string())
                .map_err(|e| format!("Failed to call {}: {}", event, e))?;
            return Ok(());
        }

        Ok(())
    }

    pub fn cleanup(&self) {
        use crate::plugins::api::emit_i18n_event;
        use crate::services::global::i18n_service;
        use crate::services::i18n::LocaleCallbackToken;

        let registry_key = format!("_locale_callback_token_{}", self.plugin_id);
        if let Ok(token_id) = self.lua.named_registry_value::<usize>(&registry_key) {
            let i18n = i18n_service();
            i18n.remove_locale_callback(&LocaleCallbackToken(token_id));
        }

        let _ = self.lua.set_named_registry_value(
            &format!("_locale_change_callbacks_{}", self.plugin_id),
            mlua::Value::Nil,
        );
        let _ = self.lua.set_named_registry_value(
            &format!("_locale_callback_token_{}", self.plugin_id),
            mlua::Value::Nil,
        );

        {
            let i18n = i18n_service();
            i18n.remove_plugin_translations(&self.plugin_id);
            let _ = emit_i18n_event(&self.plugin_id, "remove_translations", "", "");
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::api::new_api_registry;
    use mlua::Result as LuaResult;
    use std::env;

    #[test]
    fn test_runtime_creation() {
        let temp_dir = env::temp_dir().join("sl_test_plugin");
        let data_dir = temp_dir.join("data");
        let api_registry = new_api_registry();

        let runtime = PluginRuntime::new("test-plugin", &temp_dir, &data_dir, api_registry, vec![]);
        assert!(runtime.is_ok());

        let runtime = runtime.unwrap();
        assert!(!runtime.is_loaded());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sandbox_restrictions() {
        let temp_dir = env::temp_dir().join("sl_test_sandbox");
        let data_dir = temp_dir.join("data");
        let api_registry = new_api_registry();

        let runtime =
            PluginRuntime::new("test-sandbox", &temp_dir, &data_dir, api_registry, vec![]).unwrap();

        let result: LuaResult<Value> = runtime.lua.load("return os").eval();
        assert!(matches!(result, Ok(Value::Nil)));

        let result: LuaResult<Value> = runtime.lua.load("return io").eval();
        assert!(matches!(result, Ok(Value::Nil)));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
