use super::runtime::PluginRuntime;
use mlua::Value;
use std::path::Path;

impl PluginRuntime {
    pub(super) fn setup_sandbox(&self, plugin_id: &str, plugin_dir: &Path) -> Result<(), String> {
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
        if self.globals_table().get::<Value>("os").is_ok() {
            self.globals_table()
                .set("os", Value::Nil)
                .map_err(|e| format!("Failed to remove os table: {}", e))?;
        }

        Ok(())
    }

    fn sandbox_io_table(&self) -> Result<(), String> {
        if self.globals_table().get::<Value>("io").is_ok() {
            self.globals_table()
                .set("io", Value::Nil)
                .map_err(|e| format!("Failed to remove io table: {}", e))?;
        }

        Ok(())
    }

    fn remove_dangerous_globals(&self) -> Result<(), String> {
        let globals = self.globals_table();

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
}
