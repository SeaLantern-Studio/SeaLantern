use mlua::Lua;
use sysinfo::System;

use super::common::{emit_system_log, map_system_err, SystemContext};

pub(super) fn get_memory(lua: &Lua, ctx: &SystemContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, ()| {
        emit_system_log(&ctx.plugin_id, "sl.system.get_memory");

        let mut sys = System::new();
        sys.refresh_memory();

        let total = sys.total_memory();
        let used = sys.used_memory();
        let free = total.saturating_sub(used);

        let mem_table = lua.create_table()?;
        mem_table.set("total", total)?;
        mem_table.set("used", used)?;
        mem_table.set("free", free)?;
        Ok(mem_table)
    })
    .map_err(|e| map_system_err("system.create_get_memory_failed", e))
}

pub(super) fn get_cpu(lua: &Lua, ctx: &SystemContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, ()| {
        emit_system_log(&ctx.plugin_id, "sl.system.get_cpu");

        let mut sys = System::new();
        sys.refresh_cpu_all();

        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_cpu_all();

        let cpus = sys.cpus();
        let cpu_name = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();
        let cores = cpus.len() as u64;
        let usage: f64 = if cores > 0 {
            cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cores as f64
        } else {
            0.0
        };

        let usage = (usage * 100.0).round() / 100.0;

        let cpu_table = lua.create_table()?;
        cpu_table.set("name", cpu_name)?;
        cpu_table.set("cores", cores)?;
        cpu_table.set("usage", usage)?;
        Ok(cpu_table)
    })
    .map_err(|e| map_system_err("system.create_get_cpu_failed", e))
}
