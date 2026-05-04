use super::shared::emit_process_log;
use crate::plugins::runtime::process::common::{
    is_process_owner, truncate_output, ProcessRegistry,
};
use mlua::{Function, Lua, Value};
use std::sync::Arc;

/// 注册 `sl.process.read_output`
///
/// # Parameters
///
/// - `lua`: 当前 Lua 实例
/// - `plugin_id`: 当前插件 ID
/// - `process_registry`: 共用进程注册表
///
/// # Returns
///
/// 返回一个读取后台进程输出的 Lua 函数
pub(super) fn read_output(
    lua: &Lua,
    plugin_id: &str,
    process_registry: &ProcessRegistry,
) -> Result<Function, String> {
    let pid = plugin_id.to_string();
    let registry = Arc::clone(process_registry);

    lua.create_function(move |lua, target_pid: u32| {
        emit_process_log(&pid, "sl.process.read_output", &format!("pid={}", target_pid));

        let mut procs = registry.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Process registry lock poisoned: {}", e);
            e.into_inner()
        });

        if let Some(entry) = procs.get_mut(&target_pid) {
            if !is_process_owner(entry, &pid) {
                return Ok(Value::Nil);
            }

            if let Some(ref mut stdout) = entry.child.stdout {
                use std::io::Read;

                let mut buf = [0u8; 8192];
                match stdout.read(&mut buf) {
                    Ok(0) => {
                        if entry.stdout_buf.is_empty() {
                            return Ok(Value::Nil);
                        }
                        let output = String::from_utf8_lossy(&entry.stdout_buf).to_string();
                        entry.stdout_buf.clear();
                        Ok(Value::String(lua.create_string(&output)?))
                    }
                    Ok(n) => {
                        entry.stdout_buf.extend_from_slice(&buf[..n]);
                        truncate_output(&mut entry.stdout_buf);
                        let output = String::from_utf8_lossy(&entry.stdout_buf).to_string();
                        entry.stdout_buf.clear();
                        Ok(Value::String(lua.create_string(&output)?))
                    }
                    Err(_) => Ok(Value::Nil),
                }
            } else {
                Ok(Value::Nil)
            }
        } else {
            Ok(Value::Nil)
        }
    })
    .map_err(|e| format!("Failed to create process.read_output: {}", e))
}
