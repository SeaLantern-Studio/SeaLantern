// mod common是通用函数
mod basic;
mod common;
mod component;
mod context_menu;
mod feedback;
mod sidebar;
mod style;

use super::PluginRuntime;
use mlua::Table;

impl PluginRuntime {
    pub(super) fn setup_ui_namespace(&self, sl: &Table) -> Result<(), String> {
        let ui_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create UI table(sl.ui): {}", e))?;

        basic::register(self, &ui_table)?;
        component::register(self, &ui_table)?;
        context_menu::register(self, &ui_table)?;
        feedback::register(self, &ui_table)?;
        sidebar::register(self, &ui_table)?;
        style::register(self, &ui_table)?;

        sl.set("ui", ui_table)
            .map_err(|e| format!("Failed to set UI table(sl.ui): {}", e))?;
        Ok(())
    }
}
