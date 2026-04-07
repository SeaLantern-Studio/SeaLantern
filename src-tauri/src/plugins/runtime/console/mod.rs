use super::PluginRuntime;
use mlua::Table;

pub use common::i18n_arg;

mod common;
mod logs;
mod send;
mod status;

use common::{create_console_table, set_console_function, set_console_table, ConsoleContext};

impl PluginRuntime {
    pub(super) fn setup_console_namespace(&self, sl: &Table) -> Result<(), String> {
        let console_table = create_console_table(&self.lua)?;
        let ctx = ConsoleContext::new(self.plugin_id.clone());

        set_console_function(
            &console_table,
            "send",
            send::send(&self.lua, &ctx)?,
            "console.set_send_failed",
        )?;
        set_console_function(
            &console_table,
            "get_logs",
            logs::get_logs(&self.lua, &ctx)?,
            "console.set_get_logs_failed",
        )?;
        set_console_function(
            &console_table,
            "get_status",
            status::get_status(&self.lua, &ctx)?,
            "console.set_get_status_failed",
        )?;

        set_console_table(sl, console_table)
    }
}
