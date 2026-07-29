//! sl.ui 命名空间装配入口
//!
//! 职责：
//! - 创建 Lua 表 sl.ui，并将各子模块的 API 注册到其中
//! - 保持装配顺序与约定：config -> basic -> style -> feedback -> sidebar -> context_menu -> component
//! - 仅承担“路由/挂载”，不在此处放置具体闭包实现
//!
//! 目录约定：
//! - 子模块位于同级目录下，每个仅暴露一个 pub(super) fn register(runtime, ui_table)
//! - 公共工具位于 common.rs，仅在出现跨 >=2 子模块复用后上提
//! - 行为开关位于 config.rs，例如 sl.ui.set_error_mode('compat'|'strict')

mod basic;
mod common;
mod component;
mod config;
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
            .map_err(|e| format!("创建 UI 表 (sl.ui) 失败: {}", e))?;

        // Register in a fixed, documented order to keep deterministic behavior
        config::register(self, &ui_table)?; // config first (e.g., set_error_mode)
        basic::register(self, &ui_table)?;
        style::register(self, &ui_table)?;
        feedback::register(self, &ui_table)?;
        sidebar::register(self, &ui_table)?;
        context_menu::register(self, &ui_table)?;
        component::register(self, &ui_table)?;

        sl.set("ui", ui_table)
            .map_err(|e| format!("设置 UI 表 (sl.ui) 失败: {}", e))?;
        Ok(())
    }
}
