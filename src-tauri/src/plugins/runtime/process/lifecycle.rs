use super::common::{is_process_owner, ProcessRegistry};
use mlua::{Function, Lua, Table};
use std::sync::Arc;

fn emit_process_log(plugin_id: &str, action: &str, detail: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", action, detail);
}

pub(super) fn kill(
    lua: &Lua,
    plugin_id: &str,
    process_registry: &ProcessRegistry,
) -> Result<Function, String> {
    let pid = plugin_id.to_string();
    let registry = Arc::clone(process_registry);

    lua.create_function(move |_, target_pid: u32| {
        emit_process_log(&pid, "sl.process.kill", &format!("pid={}", target_pid));

        let mut procs = registry.lock().unwrap_or_else(|e| {
            eprintln!("[WARN] Process registry lock poisoned: {}", e);
            e.into_inner()
        });

        if let Some(mut entry) = procs.remove(&target_pid) {
            if !is_process_owner(&entry, &pid) {
                procs.insert(target_pid, entry);
                return Ok(false);
            }

            match entry.child.kill() {
                Ok(_) => {
                    let _ = entry.child.wait();
                    Ok(true)
                }
                Err(e) => Err(mlua::Error::runtime(format!(
                    "Failed to kill process {}: {}",
                    target_pid, e
                ))),
            }
        } else {
            Ok(false)
        }
    })
    .map_err(|e| format!("Failed to create process.kill: {}", e))
}

pub(super) fn register(
    lua: &Lua,
    process_table: &Table,
    plugin_id: &str,
    process_registry: &ProcessRegistry,
) -> Result<(), String> {
    process_table
        .set("kill", kill(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.kill: {}", e))?;
    Ok(())
}
