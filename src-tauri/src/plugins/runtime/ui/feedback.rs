use super::super::PluginRuntime;
use super::common::{
    emit_ui_action, json_to_string, lua_str, map_create_err, map_set_err, UiLogSpec,
};

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
                    let json = json_to_string(
                        &serde_json::json!({
                            "type": toast_type,
                            "message": message,
                            "duration": dur
                        }),
                        "toast",
                    )?;

                    emit_ui_action(
                        lua,
                        &pid,
                        "toast",
                        "toast",
                        "toast",
                        &json,
                        Some(UiLogSpec {
                            api_name: "sl.ui.toast",
                            target: &toast_type,
                        }),
                    )
                },
            ),
            "ui.toast",
        )?;
    map_set_err(ui_table.set("toast", toast_fn), "ui.toast")?;

    Ok(())
}
