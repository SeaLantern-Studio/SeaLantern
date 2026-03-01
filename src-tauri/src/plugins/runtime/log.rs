use super::PluginRuntime;
use crate::services::global::i18n_service;
use mlua::{Function, Table};

impl PluginRuntime {
    fn map_log_err(key: &str, e: mlua::Error) -> String {
        format!("{}: {}", i18n_service().t(key), e)
    }

    fn convert_lua_string(s: &mlua::String) -> String {
        String::from_utf8_lossy(&s.as_bytes()).into_owned()
    }

    fn create_log_function(&self, plugin_id: &str, level: &str) -> Result<Function, mlua::Error> {
        use crate::plugins::api::emit_log_event;

        let pid = plugin_id.to_string();
        let level = level.to_string();
        self.lua.create_function(move |_, msg: mlua::String| {
            let msg_str = Self::convert_lua_string(&msg);
            println!("[{}] [{}] {}", level.to_uppercase(), pid, msg_str);
            let _ = emit_log_event(&pid, &level, &msg_str);
            Ok(())
        })
    }

    fn create_noop_log_function(&self) -> Result<Function, mlua::Error> {
        self.lua
            .create_function(move |_, _msg: mlua::String| Ok(()))
    }

    pub(super) fn setup_log_namespace(
        &self,
        sl: &Table,
        has_log_permission: bool,
    ) -> Result<(), String> {
        let log = self
            .lua
            .create_table()
            .map_err(|e| Self::map_log_err("log.create_table_failed", e))?;

        let plugin_id = self.plugin_id.clone();

        // debug 需要权限检查
        let debug_fn = if has_log_permission {
            self.create_log_function(&plugin_id, "debug")
                .map_err(|e| Self::map_log_err("log.create_debug_failed", e))?
        } else {
            self.create_noop_log_function()
                .map_err(|e| Self::map_log_err("log.create_debug_noop_failed", e))?
        };
        log.set("debug", debug_fn)
            .map_err(|e| Self::map_log_err("log.set_debug_failed", e))?;

        // info, warn, error 不需要权限检查
        let info_fn = self
            .create_log_function(&plugin_id, "info")
            .map_err(|e| Self::map_log_err("log.create_info_failed", e))?;
        log.set("info", info_fn)
            .map_err(|e| Self::map_log_err("log.set_info_failed", e))?;

        let warn_fn = self
            .create_log_function(&plugin_id, "warn")
            .map_err(|e| Self::map_log_err("log.create_warn_failed", e))?;
        log.set("warn", warn_fn)
            .map_err(|e| Self::map_log_err("log.set_warn_failed", e))?;

        let error_fn = self
            .create_log_function(&plugin_id, "error")
            .map_err(|e| Self::map_log_err("log.create_error_failed", e))?;
        log.set("error", error_fn)
            .map_err(|e| Self::map_log_err("log.set_error_failed", e))?;

        sl.set("log", log)
            .map_err(|e| Self::map_log_err("log.set_log_failed", e))?;

        Ok(())
    }
}
