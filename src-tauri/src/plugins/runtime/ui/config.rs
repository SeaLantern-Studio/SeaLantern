use super::super::PluginRuntime;
use super::common::{map_create_err, map_set_err, set_error_mode};

pub(super) fn register(runtime: &PluginRuntime, ui_table: &mlua::Table) -> Result<(), String> {
    // sl.ui.set_error_mode(mode) where mode in {'compat', 'strict'}
    let pid = runtime.plugin_id.clone();
    let set_error_mode_fn = map_create_err(
        runtime.lua.create_function(move |lua, mode: mlua::String| {
            let mode_str = String::from_utf8_lossy(&mode.as_bytes()).into_owned();
            set_error_mode(lua, &pid, &mode_str)?;
            Ok(true)
        }),
        "ui.set_error_mode",
    )?;
    map_set_err(ui_table.set("set_error_mode", set_error_mode_fn), "ui.set_error_mode")?;

    Ok(())
}
