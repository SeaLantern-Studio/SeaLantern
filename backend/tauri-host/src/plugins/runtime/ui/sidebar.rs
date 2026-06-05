use super::super::PluginRuntime;
use super::common::{emit_result, map_create_err, map_set_err};
use crate::plugins::api::emit_sidebar_event;
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.register_sidebar({ label, icon? })
    let pid = runtime.plugin_id.clone();
    let register_sidebar_fn = map_create_err(
        runtime.lua.create_function(move |lua, config: Table| {
            let label: String = config
                .get("label")
                .map_err(|_| mlua::Error::runtime("侧边栏配置缺少必需的 'label' 字段"))?;

            let icon: String = config.get("icon").unwrap_or_default();

            emit_result(
                lua,
                &pid,
                "register_sidebar",
                emit_sidebar_event(&pid, "register", &label, &icon),
            )
        }),
        "ui.register_sidebar",
    )?;
    map_set_err(ui_table.set("register_sidebar", register_sidebar_fn), "ui.register_sidebar")?;

    // sl.ui.unregister_sidebar()
    let pid = runtime.plugin_id.clone();
    let unregister_sidebar_fn = map_create_err(
        runtime.lua.create_function(move |lua, ()| {
            emit_result(
                lua,
                &pid,
                "unregister_sidebar",
                emit_sidebar_event(&pid, "unregister", "", ""),
            )
        }),
        "ui.unregister_sidebar",
    )?;
    map_set_err(
        ui_table.set("unregister_sidebar", unregister_sidebar_fn),
        "ui.unregister_sidebar",
    )?;

    Ok(())
}
