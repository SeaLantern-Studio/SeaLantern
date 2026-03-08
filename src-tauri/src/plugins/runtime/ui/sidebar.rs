use super::super::PluginRuntime;
use crate::plugins::api::emit_sidebar_event;
use mlua::Table;

pub(super) fn register(runtime: &PluginRuntime, ui_table: &Table) -> Result<(), String> {
    // sl.ui.register_sidebar({ label, icon? })
    let pid = runtime.plugin_id.clone();
    let register_sidebar_fn = runtime
        .lua
        .create_function(move |_, config: Table| {
            let label: String = config
                .get("label")
                .map_err(|_| mlua::Error::runtime("侧边栏配置缺少必需的 'label' 字段"))?;

            let icon: String = config.get("icon").unwrap_or_default();

            match emit_sidebar_event(&pid, "register", &label, &icon) {
                Ok(()) => Ok(true),
                Err(e) => {
                    eprintln!("[UI] register_sidebar error: {}", e);
                    Ok(false)
                }
            }
        })
        .map_err(|e| format!("Failed to create ui.register_sidebar: {}", e))?;
    ui_table
        .set("register_sidebar", register_sidebar_fn)
        .map_err(|e| format!("Failed to set ui.register_sidebar: {}", e))?;

    // sl.ui.unregister_sidebar()
    let pid = runtime.plugin_id.clone();
    let unregister_sidebar_fn = runtime
        .lua
        .create_function(move |_, ()| match emit_sidebar_event(&pid, "unregister", "", "") {
            Ok(()) => Ok(true),
            Err(e) => {
                eprintln!("[UI] unregister_sidebar error: {}", e);
                Ok(false)
            }
        })
        .map_err(|e| format!("Failed to create ui.unregister_sidebar: {}", e))?;
    ui_table
        .set("unregister_sidebar", unregister_sidebar_fn)
        .map_err(|e| format!("Failed to set ui.unregister_sidebar: {}", e))?;

    Ok(())
}
