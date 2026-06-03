use super::shared::emit_process_log;
use crate::plugins::runtime::process::common::{
    collect_finished_processes, is_output_drained, is_process_owner, ProcessRegistry,
};
use mlua::{Function, Lua, Table, Value};
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
/// 返回一个读取后台进程输出的 Lua 函数。
///
/// 默认兼容模式下，仅返回 stdout 字符串或 `nil`。
///
/// 当第二个参数传入 `{ include_stderr = true }` 时，有输出时返回表：
/// - `stdout: string`
/// - `stderr: string`
/// - `truncated = true` 表示本次返回的 stdout 片段发生截断
/// - `stderr_truncated = true` 表示本次返回的 stderr 片段发生截断
///
/// 如果两个流当前都没有可读取内容，则返回 `nil`。
pub(super) fn read_output(
    lua: &Lua,
    plugin_id: &str,
    process_registry: &ProcessRegistry,
) -> Result<Function, String> {
    let pid = plugin_id.to_string();
    let registry = Arc::clone(process_registry);

    lua.create_function(move |lua, (target_pid, options): (u32, Option<Table>)| {
        emit_process_log(&pid, "sl.process.read_output", &format!("pid={}", target_pid));

        let include_stderr = options
            .as_ref()
            .and_then(|table| table.get::<bool>("include_stderr").ok())
            .unwrap_or(false);

        let mut procs = registry.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Process registry lock poisoned: {}", e);
            e.into_inner()
        });

        if let Some(entry) = procs.get_mut(&target_pid) {
            if !is_process_owner(entry, &pid) {
                return Ok(Value::Nil);
            }

            let mut output_state = entry.output.lock().unwrap_or_else(|e| {
                eprintln!("[WARN] Process output lock poisoned: {}", e);
                e.into_inner()
            });

            if include_stderr {
                if output_state.stdout_buf.is_empty() && output_state.stderr_buf.is_empty() {
                    let should_cleanup = is_output_drained(&output_state);
                    drop(output_state);
                    if should_cleanup {
                        procs.remove(&target_pid);
                    }
                    return Ok(Value::Nil);
                }

                let output = lua.create_table()?;
                let stdout_text = String::from_utf8_lossy(&output_state.stdout_buf).to_string();
                let stderr_text = String::from_utf8_lossy(&output_state.stderr_buf).to_string();
                output.set("stdout", lua.create_string(&stdout_text)?)?;
                output.set("stderr", lua.create_string(&stderr_text)?)?;

                if output_state.stdout_truncated {
                    output.set("truncated", true)?;
                }
                if output_state.stderr_truncated {
                    output.set("stderr_truncated", true)?;
                }

                output_state.stdout_buf.clear();
                output_state.stderr_buf.clear();
                output_state.stdout_truncated = false;
                output_state.stderr_truncated = false;

                let should_cleanup = is_output_drained(&output_state);
                drop(output_state);
                if should_cleanup {
                    procs.remove(&target_pid);
                }

                return Ok(Value::Table(output));
            }

            if output_state.stdout_buf.is_empty() {
                let should_cleanup = is_output_drained(&output_state);
                drop(output_state);
                if should_cleanup {
                    procs.remove(&target_pid);
                }
                return Ok(Value::Nil);
            }

            let output = String::from_utf8_lossy(&output_state.stdout_buf).to_string();
            output_state.stdout_buf.clear();
            output_state.stdout_truncated = false;

            let should_cleanup = is_output_drained(&output_state);
            drop(output_state);
            if should_cleanup {
                procs.remove(&target_pid);
            }

            Ok(Value::String(lua.create_string(&output)?))
        } else {
            collect_finished_processes(&mut procs);
            Ok(Value::Nil)
        }
    })
    .map_err(|e| format!("Failed to create process.read_output: {}", e))
}
