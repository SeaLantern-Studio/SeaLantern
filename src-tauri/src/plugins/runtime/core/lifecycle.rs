use super::runtime::PluginRuntime;
use mlua::Value;
use std::path::Path;

impl PluginRuntime {
    pub fn load_file(&self, path: &Path) -> Result<(), String> {
        let bytes = self.load_file_bytes(path)?;
        let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        let content = String::from_utf8_lossy(bytes).into_owned();

        let result: Value = self.eval_lua_file(path, &content)?;

        if let Value::Table(table) = result {
            self.set_plugin_global(table)?;
        }

        self.mark_loaded();
        Ok(())
    }

    pub fn call_lifecycle(&self, event: &str) -> Result<(), String> {
        if let Ok(plugin_table) = self.plugin_table() {
            if self.call_table_function0(&plugin_table, event)? {
                return Ok(());
            }
        }

        let _ = self.call_global_function0(event)?;
        Ok(())
    }

    pub fn call_lifecycle_with_arg(&self, event: &str, arg: &str) -> Result<(), String> {
        if let Ok(plugin_table) = self.plugin_table() {
            if self.call_table_function1(&plugin_table, event, arg)? {
                return Ok(());
            }
        }

        let _ = self.call_global_function1(event, arg)?;
        Ok(())
    }

    pub fn cleanup(&self) {
        use crate::plugins::api::emit_i18n_event;
        use crate::plugins::runtime::process::kill_plugin_processes;
        use crate::services::global::i18n_service;
        use crate::services::i18n::LocaleCallbackToken;

        let callbacks_registry_key = format!("_locale_change_callbacks_{}", self.plugin_id);
        let token_registry_key = format!("_locale_callback_token_{}", self.plugin_id);

        if let Ok(token_id) = self.lua.named_registry_value::<usize>(&token_registry_key) {
            i18n_service().remove_locale_callback(&LocaleCallbackToken(token_id));
        }

        let _ = self
            .lua
            .set_named_registry_value(&callbacks_registry_key, mlua::Value::Nil);
        let _ = self
            .lua
            .set_named_registry_value(&token_registry_key, mlua::Value::Nil);

        kill_plugin_processes(&self.process_registry, &self.plugin_id);

        i18n_service().remove_plugin_translations(&self.plugin_id);
        let _ = emit_i18n_event(&self.plugin_id, "remove_translations", "", "");
    }
}
