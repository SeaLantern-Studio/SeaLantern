use mlua::Lua;

use super::common::{emit_system_log, map_system_err, SystemContext};
use crate::utils::app_version;

pub(super) fn get_os(lua: &Lua, ctx: &SystemContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, ()| {
        emit_system_log(&ctx.plugin_id, "sl.system.get_os");
        Ok(std::env::consts::OS.to_string())
    })
    .map_err(|e| map_system_err("system.create_get_os_failed", e))
}

pub(super) fn get_arch(lua: &Lua, ctx: &SystemContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, ()| {
        emit_system_log(&ctx.plugin_id, "sl.system.get_arch");
        Ok(std::env::consts::ARCH.to_string())
    })
    .map_err(|e| map_system_err("system.create_get_arch_failed", e))
}

pub(super) fn get_app_version(lua: &Lua, ctx: &SystemContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, ()| {
        emit_system_log(&ctx.plugin_id, "sl.system.get_app_version");
        Ok(app_version::display_version())
    })
    .map_err(|e| map_system_err("system.create_get_app_version_failed", e))
}
