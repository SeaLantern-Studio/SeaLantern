mod exec;
mod query;
mod read_output;
mod shared;

use mlua::{Lua, Table};

use super::common::ProcessRegistry;

/// 注册 `sl.process` 下的命令接口
///
/// # Parameters
///
/// - `lua`: 当前 Lua 实例
/// - `process_table`: `sl.process` 对应的 Lua 表
/// - `plugin_dir`: 当前插件目录
/// - `plugin_id`: 当前插件 ID
/// - `permissions`: 当前插件权限列表
/// - `process_registry`: 共用进程注册表
///
/// # Returns
///
/// 注册成功时返回 `Ok(())`
pub(super) fn register(
    lua: &Lua,
    process_table: &Table,
    plugin_dir: &std::path::Path,
    plugin_id: &str,
    permissions: &[String],
    process_registry: &ProcessRegistry,
) -> Result<(), String> {
    process_table
        .set("exec", exec::exec(lua, plugin_dir, plugin_id, permissions, process_registry)?)
        .map_err(|e| format!("Failed to set process.exec: {}", e))?;
    process_table
        .set("get", query::get(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.get: {}", e))?;
    process_table
        .set("list", query::list(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.list: {}", e))?;
    process_table
        .set("read_output", read_output::read_output(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.read_output: {}", e))?;

    Ok(())
}
