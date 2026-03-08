use super::super::PluginRuntime;
use crate::plugins::api::{emit_permission_log, emit_ui_event};

pub(super) fn register(runtime: &PluginRuntime, ui_table: &mlua::Table) -> Result<(), String> {
    // sl.ui.toast(type, message, duration?)
    let pid = runtime.plugin_id.clone();
    let toast_fn = runtime
        .lua
        .create_function(
            move |_, (toast_type, message, duration): (mlua::String, mlua::String, Option<u32>)| {
                let toast_type = String::from_utf8_lossy(&toast_type.as_bytes()).into_owned();
                let message = String::from_utf8_lossy(&message.as_bytes()).into_owned();
                let dur = duration.unwrap_or(3000);
                let _ = emit_permission_log(&pid, "api_call", "sl.ui.toast", &toast_type);
                let json = serde_json::json!({
                    "type": toast_type,
                    "message": message,
                    "duration": dur
                })
                .to_string();
                match emit_ui_event(&pid, "toast", "toast", &json) {
                    Ok(()) => Ok(true),
                    Err(e) => {
                        eprintln!("[UI] toast error: {}", e);
                        Ok(false)
                    }
                }
            },
        )
        .map_err(|e| format!("Failed to create ui.toast: {}", e))?;
    ui_table
        .set("toast", toast_fn)
        .map_err(|e| format!("Failed to set ui.toast: {}", e))?;

    Ok(())
}
