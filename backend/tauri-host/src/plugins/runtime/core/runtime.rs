use crate::plugins::runtime::process::ProcessEntry;
use crate::services::global::i18n_service;
use mlua::{Function, Table, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

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

pub struct PluginRuntime {
    pub(crate) lua: mlua::Lua,
    pub(crate) plugin_id: String,
    pub(crate) plugin_dir: PathBuf,
    pub(crate) data_dir: PathBuf,
    pub(crate) server_dir: PathBuf,
    pub(crate) global_dir: PathBuf,
    pub(crate) loaded: AtomicBool,

    pub(crate) permissions: Vec<String>,

    pub(crate) allowed_programs: HashSet<PathBuf>,

    pub(crate) api_registry: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,

    pub(crate) storage_lock: Arc<Mutex<()>>,

    pub(crate) process_registry: Arc<Mutex<HashMap<u32, ProcessEntry>>>,

    #[allow(dead_code)] // 未来回调用
    pub(crate) element_callbacks: Arc<Mutex<std::collections::HashMap<u64, mlua::RegistryKey>>>,
}

impl PluginRuntime {
    pub fn lua(&self) -> &mlua::Lua {
        &self.lua
    }

    #[allow(dead_code)] // 测试调用
    pub fn is_loaded(&self) -> bool {
        self.loaded.load(Ordering::SeqCst)
    }

    // test要用, 别删
    pub fn lua_eval<T>(&self, code: &str) -> mlua::Result<T>
    where
        T: mlua::FromLuaMulti,
    {
        self.lua.load(code).eval()
    }

    pub(super) fn mark_loaded(&self) {
        self.loaded.store(true, Ordering::SeqCst);
    }

    pub(super) fn set_plugin_global(&self, table: Table) -> Result<(), String> {
        let globals = self.lua.globals();
        globals
            .set("plugin", table)
            .map_err(|e| core_t1("plugins.runtime.core.set_plugin_global_failed", e.to_string()))
    }

    pub(super) fn load_file_bytes(&self, path: &Path) -> Result<Vec<u8>, String> {
        fs::read(path).map_err(|e| {
            core_t2(
                "plugins.runtime.core.read_file_failed",
                path.display().to_string(),
                e.to_string(),
            )
        })
    }

    pub(super) fn eval_lua_file(&self, path: &Path, content: &str) -> Result<Value, String> {
        self.lua
            .load(content)
            .set_name(path.to_string_lossy())
            .eval()
            .map_err(|e| {
                core_t2(
                    "plugins.runtime.core.execute_file_failed",
                    path.display().to_string(),
                    e.to_string(),
                )
            })
    }

    pub(super) fn call_table_function0(&self, table: &Table, event: &str) -> Result<bool, String> {
        if let Ok(func) = table.get::<Function>(event) {
            func.call::<()>(()).map_err(|e| {
                core_t2("plugins.runtime.core.call_plugin_event_failed", event, e.to_string())
            })?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn call_table_function1(
        &self,
        table: &Table,
        event: &str,
        arg: &str,
    ) -> Result<bool, String> {
        if let Ok(func) = table.get::<Function>(event) {
            func.call::<()>(arg.to_string()).map_err(|e| {
                core_t2("plugins.runtime.core.call_plugin_event_failed", event, e.to_string())
            })?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn call_global_function0(&self, event: &str) -> Result<bool, String> {
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<Function>(event) {
            func.call::<()>(()).map_err(|e| {
                core_t2("plugins.runtime.core.call_global_event_failed", event, e.to_string())
            })?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn call_global_function1(&self, event: &str, arg: &str) -> Result<bool, String> {
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<Function>(event) {
            func.call::<()>(arg.to_string()).map_err(|e| {
                core_t2("plugins.runtime.core.call_global_event_failed", event, e.to_string())
            })?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn plugin_table(&self) -> mlua::Result<Table> {
        self.lua.globals().get::<Table>("plugin")
    }
}
