use super::runtime::PluginRuntime;
use crate::services::global::i18n_service;
use mlua::Value;
use std::collections::HashMap;
use std::path::Path;

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
    pub(super) fn setup_sandbox(&self, plugin_id: &str, plugin_dir: &Path) -> Result<(), String> {
        let globals = self.lua.globals();

        globals
            .set("PLUGIN_ID", plugin_id)
            .map_err(|e| core_t1("plugins.runtime.core.set_plugin_id_failed", e.to_string()))?;

        globals
            .set("PLUGIN_DIR", plugin_dir.to_string_lossy().to_string())
            .map_err(|e| core_t1("plugins.runtime.core.set_plugin_dir_failed", e.to_string()))?;

        self.sandbox_os_table()?;
        self.sandbox_io_table()?;
        self.remove_dangerous_globals()?;

        Ok(())
    }

    fn sandbox_os_table(&self) -> Result<(), String> {
        if self.globals_table().get::<Value>("os").is_ok() {
            self.globals_table().set("os", Value::Nil).map_err(|e| {
                core_t1("plugins.runtime.core.remove_os_table_failed", e.to_string())
            })?;
        }

        Ok(())
    }

    fn sandbox_io_table(&self) -> Result<(), String> {
        if self.globals_table().get::<Value>("io").is_ok() {
            self.globals_table().set("io", Value::Nil).map_err(|e| {
                core_t1("plugins.runtime.core.remove_io_table_failed", e.to_string())
            })?;
        }

        Ok(())
    }

    fn remove_dangerous_globals(&self) -> Result<(), String> {
        let globals = self.globals_table();

        let dangerous_globals = ["loadfile", "dofile", "load", "require"];
        for func in dangerous_globals {
            globals.set(func, Value::Nil).map_err(|e| {
                core_t2("plugins.runtime.core.remove_global_failed", func, e.to_string())
            })?;
        }

        if globals.get::<Value>("debug").is_ok() {
            globals.set("debug", Value::Nil).map_err(|e| {
                core_t1("plugins.runtime.core.remove_debug_table_failed", e.to_string())
            })?;
        }
        if globals.get::<Value>("package").is_ok() {
            globals.set("package", Value::Nil).map_err(|e| {
                core_t1("plugins.runtime.core.remove_package_table_failed", e.to_string())
            })?;
        }

        Ok(())
    }
}
