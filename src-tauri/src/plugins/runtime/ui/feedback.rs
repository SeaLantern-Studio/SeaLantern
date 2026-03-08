use super::super::PluginRuntime;
use super::common::{emit_result, lua_str, map_create_err, map_set_err};
use crate::plugins::api::{emit_permission_log, emit_ui_event};

pub(super) fn register(runtime: &PluginRuntime, ui_table: &mlua::Table) -> Result<(), String> {
    // sl.ui.toast(type, message, duration?)
    let pid = runtime.plugin_id.clone();
    let toast_fn =
        map_create_err(
            runtime.lua.create_function(
                move |lua,
                      (toast_type, message, duration): (
                    mlua::String,
                    mlua::String,
                    Option<u32>,
                )| {
                    let toast_type = lua_str(toast_type);
                    let message = lua_str(message);
                    let dur = duration.unwrap_or(3000);
                    let _ = emit_permission_log(&pid, "api_call", "sl.ui.toast", &toast_type);
                    let json = serde_json::json!({
                        "type": toast_type,
                        "message": message,
                        "duration": dur
                    })
                    .to_string();
                    emit_result(lua, &pid, "toast", emit_ui_event(&pid, "toast", "toast", &json))
                },
            ),
            "ui.toast",
        )?;
    map_set_err(ui_table.set("toast", toast_fn), "ui.toast")?;

    Ok(())
}
