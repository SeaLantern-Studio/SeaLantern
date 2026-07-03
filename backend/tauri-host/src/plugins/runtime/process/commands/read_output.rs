use super::shared::emit_process_log;
use crate::plugins::runtime::permissions::EXECUTE_PROGRAM_PERMISSION;
use crate::plugins::runtime::process::common::process_msg2;
use crate::plugins::runtime::process::common::{
    collect_finished_processes, is_output_drained, is_process_owner, take_output_bytes,
};
use crate::plugins::runtime::process::ProcessEntry;
use mlua::{Function, Lua, Table, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn require_any_permission(
    owned_permissions: &[String],
    required_permissions: &[&str],
) -> mlua::Result<()> {
    if required_permissions
        .iter()
        .any(|permission| owned_permissions.iter().any(|owned| owned == permission))
    {
        Ok(())
    } else {
        Err(mlua::Error::runtime(crate::services::global::i18n_service().t_with_options(
            "plugins.runtime.permissions.permission_required",
            &HashMap::from([(
                "0".to_string(),
                format!("{} | {}", required_permissions.join(" | "), EXECUTE_PROGRAM_PERMISSION),
            )]),
        )))
    }
}

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
/// - `chunk_seq: integer`，当前返回片段的递增序号，便于排序
/// - `updated_at_ms: integer`，最近一次采集到该批输出的 Unix 毫秒时间戳
/// - `truncated = true` 表示本次返回的 stdout 片段发生截断
/// - `stderr_truncated = true` 表示本次返回的 stderr 片段发生截断
///
/// 如果两个流当前都没有可读取内容，则返回 `nil`。
pub(super) fn read_output(
    lua: &Lua,
    plugin_id: &str,
    permissions: &[String],
    process_registry: &Arc<Mutex<HashMap<u32, ProcessEntry>>>,
    required_permissions: &'static [&'static str],
) -> Result<Function, String> {
    let pid = plugin_id.to_string();
    let permissions = permissions.to_vec();
    let registry = Arc::clone(process_registry);

    lua.create_function(move |lua, (target_pid, options): (u32, Option<Table>)| {
        require_any_permission(&permissions, required_permissions)?;
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
                let stdout_bytes = take_output_bytes(&mut output_state.stdout_buf);
                let stderr_bytes = take_output_bytes(&mut output_state.stderr_buf);
                let stdout_text = String::from_utf8_lossy(&stdout_bytes).to_string();
                let stderr_text = String::from_utf8_lossy(&stderr_bytes).to_string();
                output.set("stdout", lua.create_string(&stdout_text)?)?;
                output.set("stderr", lua.create_string(&stderr_text)?)?;
                output.set("chunk_seq", output_state.next_chunk_seq)?;
                if let Some(updated_at_ms) = output_state.last_update_unix_ms {
                    output.set("updated_at_ms", updated_at_ms)?;
                }

                if output_state.stdout_truncated {
                    output.set("truncated", true)?;
                }
                if output_state.stderr_truncated {
                    output.set("stderr_truncated", true)?;
                }

                output_state.next_chunk_seq = output_state.next_chunk_seq.saturating_add(1);
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

            let stdout_bytes = take_output_bytes(&mut output_state.stdout_buf);
            let output = String::from_utf8_lossy(&stdout_bytes).to_string();
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
    .map_err(|e| {
        process_msg2(
            "plugins.runtime.process.create_api_failed",
            "process.read_output",
            e.to_string(),
        )
    })
}
