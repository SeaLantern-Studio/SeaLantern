use super::PluginRuntime;
use mlua::Table;

impl PluginRuntime {
    pub(super) fn setup_log_namespace(
        &self,
        sl: &Table,
        has_log_permission: bool,
    ) -> Result<(), String> {
        use crate::plugins::api::emit_log_event;

        let log = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create log table: {}", e))?;

        let plugin_id = self.plugin_id.clone();

        if has_log_permission {
            let pid = plugin_id.clone();
            let debug_fn = self
                .lua
                .create_function(move |_, msg: mlua::String| {
                    let msg_str = String::from_utf8_lossy(&msg.as_bytes()).to_string();
                    println!("[DEBUG] [{}] {}", pid, msg_str);

                    let _ = emit_log_event(&pid, "debug", &msg_str);
                    Ok(())
                })
                .map_err(|e| format!("Failed to create log.debug: {}", e))?;
            log.set("debug", debug_fn)
                .map_err(|e| format!("Failed to set log.debug: {}", e))?;
        } else {
            let debug_fn = self
                .lua
                .create_function(move |_, _msg: mlua::String| Ok(()))
                .map_err(|e| format!("Failed to create log.debug (noop): {}", e))?;
            log.set("debug", debug_fn)
                .map_err(|e| format!("Failed to set log.debug (noop): {}", e))?;
        }

        let pid = plugin_id.clone();
        let info_fn = self
            .lua
            .create_function(move |_, msg: mlua::String| {
                let msg_str = String::from_utf8_lossy(&msg.as_bytes()).to_string();
                println!("[INFO] [{}] {}", pid, msg_str);

                let _ = emit_log_event(&pid, "info", &msg_str);
                Ok(())
            })
            .map_err(|e| format!("Failed to create log.info: {}", e))?;
        log.set("info", info_fn)
            .map_err(|e| format!("Failed to set log.info: {}", e))?;

        let pid = plugin_id.clone();
        let warn_fn = self
            .lua
            .create_function(move |_, msg: mlua::String| {
                let msg_str = String::from_utf8_lossy(&msg.as_bytes()).to_string();
                println!("[WARN] [{}] {}", pid, msg_str);

                let _ = emit_log_event(&pid, "warn", &msg_str);
                Ok(())
            })
            .map_err(|e| format!("Failed to create log.warn: {}", e))?;
        log.set("warn", warn_fn)
            .map_err(|e| format!("Failed to set log.warn: {}", e))?;

        let pid = plugin_id.clone();
        let error_fn = self
            .lua
            .create_function(move |_, msg: mlua::String| {
                let msg_str = String::from_utf8_lossy(&msg.as_bytes()).to_string();
                println!("[ERROR] [{}] {}", pid, msg_str);

                let _ = emit_log_event(&pid, "error", &msg_str);
                Ok(())
            })
            .map_err(|e| format!("Failed to create log.error: {}", e))?;
        log.set("error", error_fn)
            .map_err(|e| format!("Failed to set log.error: {}", e))?;

        sl.set("log", log)
            .map_err(|e| format!("Failed to set sl.log: {}", e))?;

        Ok(())
    }
}
