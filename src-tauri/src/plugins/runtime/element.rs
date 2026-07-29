//! sl.element 命名空间装配入口
//!
//! 职责：
//! - 创建 Lua 表 sl.element，并按职责挂载子模块能力
//! - 保持入口轻量，不在此处堆积具体闭包实现

mod common;
mod mutate;
mod query;
mod watch;

use super::PluginRuntime;
use crate::services::global::i18n_service;
use mlua::Table;

impl PluginRuntime {
    pub(super) fn setup_element_namespace(&self, sl: &Table) -> Result<(), String> {
        let element_table = self.lua.create_table().map_err(|e| {
            i18n_service().t_with_options(
                "element.create_table_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        query::register(self, &element_table)?;
        mutate::register(self, &element_table)?;
        watch::register(self, &element_table)?;

        sl.set("element", element_table).map_err(|e| {
            i18n_service().t_with_options(
                "element.set_element_failed",
                &crate::plugins::runtime::console::i18n_arg("0", &e.to_string()),
            )
        })?;

        Ok(())
    }
}
