mod exec;
mod query;
mod read_output;
mod shared;

use mlua::{Lua, Table};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::common::process_msg2;

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
    allowed_programs: &HashSet<PathBuf>,
    process_registry: &Arc<Mutex<HashMap<u32, super::common::ProcessEntry>>>,
) -> Result<(), String> {
    process_table
        .set(
            "exec",
            exec::exec(
                lua,
                plugin_dir,
                plugin_id,
                permissions,
                allowed_programs,
                process_registry,
            )?,
        )
        .map_err(|e| {
            process_msg2("plugins.runtime.process.set_api_failed", "process.exec", e.to_string())
        })?;
    process_table
        .set("get", query::get(lua, plugin_id, process_registry)?)
        .map_err(|e| {
            process_msg2("plugins.runtime.process.set_api_failed", "process.get", e.to_string())
        })?;
    process_table
        .set("list", query::list(lua, plugin_id, process_registry)?)
        .map_err(|e| {
            process_msg2("plugins.runtime.process.set_api_failed", "process.list", e.to_string())
        })?;
    process_table
        .set("read_output", read_output::read_output(lua, plugin_id, process_registry)?)
        .map_err(|e| {
            process_msg2(
                "plugins.runtime.process.set_api_failed",
                "process.read_output",
                e.to_string(),
            )
        })?;

    Ok(())
}
