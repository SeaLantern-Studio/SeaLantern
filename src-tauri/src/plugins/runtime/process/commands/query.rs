use super::shared::{emit_process_log, process_error};
use crate::plugins::runtime::process::common::{
    collect_finished_processes, is_process_owner, ProcessRegistry,
};
use mlua::{Function, Lua, Value};
use std::sync::Arc;

/// 注册 `sl.process.get`
///
/// # Parameters
///
/// - `lua`: 当前 Lua 实例
/// - `plugin_id`: 当前插件 ID
/// - `process_registry`: 共用进程注册表
///
/// # Returns
///
/// 返回一个读取单个进程状态的 Lua 函数
pub(super) fn get(
    lua: &Lua,
    plugin_id: &str,
    process_registry: &ProcessRegistry,
) -> Result<Function, String> {
    let pid = plugin_id.to_string();
    let registry = Arc::clone(process_registry);

    lua.create_function(move |lua, target_pid: u32| {
        emit_process_log(&pid, "sl.process.get", &format!("pid={}", target_pid));

        let mut procs = registry.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Process registry lock poisoned: {}", e);
            e.into_inner()
        });
        collect_finished_processes(&mut procs);

        if let Some(entry) = procs.get_mut(&target_pid) {
            if !is_process_owner(entry, &pid) {
                return Ok(Value::Nil);
            }

            let info = lua.create_table()?;
            info.set("pid", target_pid)?;
            info.set("program", entry.program.clone())?;
            info.set("uptime_ms", entry.started_at.elapsed().as_millis() as u64)?;

            match entry.child.try_wait() {
                Ok(Some(status)) => {
                    info.set("running", false)?;
                    info.set("exit_code", status.code().unwrap_or(-1))?;
                }
                Ok(None) => {
                    info.set("running", true)?;
                }
                Err(e) => {
                    return Err(process_error("Failed to check process status", e));
                }
            }

            Ok(Value::Table(info))
        } else {
            Ok(Value::Nil)
        }
    })
    .map_err(|e| format!("Failed to create process.get: {}", e))
}

/// 注册 `sl.process.list`
///
/// # Parameters
///
/// - `lua`: 当前 Lua 实例
/// - `plugin_id`: 当前插件 ID
/// - `process_registry`: 共用进程注册表
///
/// # Returns
///
/// 返回一个列出当前插件进程的 Lua 函数
pub(super) fn list(
    lua: &Lua,
    plugin_id: &str,
    process_registry: &ProcessRegistry,
) -> Result<Function, String> {
    let pid = plugin_id.to_string();
    let registry = Arc::clone(process_registry);

    lua.create_function(move |lua, ()| {
        emit_process_log(&pid, "sl.process.list", "");

        let mut procs = registry.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Process registry lock poisoned: {}", e);
            e.into_inner()
        });
        collect_finished_processes(&mut procs);

        let result = lua.create_table()?;
        let mut i = 1;
        let pids: Vec<u32> = procs.keys().cloned().collect();

        for proc_pid in pids {
            if let Some(entry) = procs.get_mut(&proc_pid) {
                if !is_process_owner(entry, &pid) {
                    continue;
                }

                let item = lua.create_table()?;
                item.set("pid", proc_pid)?;
                item.set("program", entry.program.clone())?;
                item.set("uptime_ms", entry.started_at.elapsed().as_millis() as u64)?;

                let running = match entry.child.try_wait() {
                    Ok(Some(_)) => false,
                    Ok(None) => true,
                    Err(_) => false,
                };
                item.set("running", running)?;

                result.set(i, item)?;
                i += 1;
            }
        }

        Ok(result)
    })
    .map_err(|e| format!("Failed to create process.list: {}", e))
}
