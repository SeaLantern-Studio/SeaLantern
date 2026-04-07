use super::common::{
    collect_finished_processes, is_process_owner, plugin_process_count, truncate_output,
    ProcessEntry, ProcessRegistry, MAX_ARGS_COUNT, MAX_ARG_LENGTH,
    MAX_BACKGROUND_PROCESSES_PER_PLUGIN, MAX_ENV_KEY_LENGTH, MAX_ENV_VALUE_LENGTH, MAX_ENV_VARS,
    MAX_FOREGROUND_EXEC_DURATION, MAX_STDOUT_BUFFER_BYTES,
};
use crate::plugins::runtime::shared::validate_path_static;
use mlua::{Function, Lua, Table, Value};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

fn emit_process_log(plugin_id: &str, action: &str, detail: &str) {
    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", action, detail);
}

fn process_error(prefix: &str, error: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::runtime(format!("{}: {}", prefix, error))
}

fn mask_args_for_log(args: &[String]) -> String {
    if args.is_empty() {
        "[]".to_string()
    } else {
        format!("[{} args]", args.len())
    }
}

fn validate_program_path(program_path: &std::path::Path, program: &str) -> Result<(), mlua::Error> {
    if !program_path.exists() {
        return Err(mlua::Error::runtime(format!("Program not found: {}", program)));
    }

    if !program_path.is_file() {
        return Err(mlua::Error::runtime(format!("Program path is not a file: {}", program)));
    }

    Ok(())
}

fn validate_args(args: &[String]) -> Result<(), mlua::Error> {
    if args.len() > MAX_ARGS_COUNT {
        return Err(mlua::Error::runtime(format!(
            "Too many arguments: maximum {} allowed",
            MAX_ARGS_COUNT
        )));
    }

    for arg in args {
        if arg.len() > MAX_ARG_LENGTH {
            return Err(mlua::Error::runtime(format!(
                "Argument too long: maximum {} characters allowed",
                MAX_ARG_LENGTH
            )));
        }
    }

    Ok(())
}

fn is_allowed_env_key(key: &str) -> bool {
    !matches!(
        key.to_ascii_uppercase().as_str(),
        "PATH"
            | "PATHEXT"
            | "LD_PRELOAD"
            | "LD_LIBRARY_PATH"
            | "DYLD_INSERT_LIBRARIES"
            | "DYLD_LIBRARY_PATH"
            | "SYSTEMROOT"
            | "COMSPEC"
            | "PROMPT"
            | "PSMODULEPATH"
    )
}

fn collect_env_vars(env_table: Table) -> Result<Vec<(String, String)>, mlua::Error> {
    let mut env_vars = Vec::new();

    for pair in env_table.pairs::<String, String>() {
        let (k, v) = pair?;

        if env_vars.len() >= MAX_ENV_VARS {
            return Err(mlua::Error::runtime(format!(
                "Too many environment variables: maximum {} allowed",
                MAX_ENV_VARS
            )));
        }

        if k.is_empty() || k.len() > MAX_ENV_KEY_LENGTH {
            return Err(mlua::Error::runtime(format!(
                "Invalid environment key length: maximum {} characters allowed",
                MAX_ENV_KEY_LENGTH
            )));
        }

        if v.len() > MAX_ENV_VALUE_LENGTH {
            return Err(mlua::Error::runtime(format!(
                "Environment value too long: maximum {} characters allowed",
                MAX_ENV_VALUE_LENGTH
            )));
        }

        if !is_allowed_env_key(&k) {
            return Err(mlua::Error::runtime(format!(
                "Environment variable is not allowed: {}",
                k
            )));
        }

        env_vars.push((k, v));
    }

    Ok(env_vars)
}

pub(super) fn exec(
    lua: &Lua,
    plugin_dir: &std::path::Path,
    plugin_id: &str,
    permissions: &[String],
    process_registry: &ProcessRegistry,
) -> Result<Function, String> {
    let dir = plugin_dir.to_path_buf();
    let pid = plugin_id.to_string();
    let perms = permissions.to_vec();
    let registry = Arc::clone(process_registry);

    lua.create_function(
        move |lua, (program, args, options): (String, Option<Vec<String>>, Option<Table>)| {
            if !perms.iter().any(|p| p == "execute_program") {
                return Err(mlua::Error::runtime(
                    "Permission denied: 'execute_program' permission required",
                ));
            }

            let program_path = validate_path_static(&dir, &program)?;
            validate_program_path(&program_path, &program)?;

            let args = args.unwrap_or_default();
            validate_args(&args)?;

            emit_process_log(
                &pid,
                "sl.process.exec",
                &format!("{} {}", program, mask_args_for_log(&args)),
            );

            let mut cwd = dir.clone();
            let mut env_vars: Vec<(String, String)> = Vec::new();
            let mut background = false;

            if let Some(ref opts) = options {
                if let Ok(cwd_str) = opts.get::<String>("cwd") {
                    cwd = validate_path_static(&dir, &cwd_str)?;
                }
                if let Ok(env_table) = opts.get::<Table>("env") {
                    env_vars = collect_env_vars(env_table)?;
                }
                if let Ok(bg) = opts.get::<bool>("background") {
                    background = bg;
                }
            }

            let mut cmd = Command::new(&program_path);
            cmd.args(&args)
                .current_dir(&cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            for (k, v) in &env_vars {
                cmd.env(k, v);
            }

            let result_table = lua.create_table()?;

            if background {
                let mut procs = registry.lock().unwrap_or_else(|e| {
                    eprintln!("[WARN] Process registry lock poisoned: {}", e);
                    e.into_inner()
                });
                collect_finished_processes(&mut procs);

                if plugin_process_count(&procs, &pid) >= MAX_BACKGROUND_PROCESSES_PER_PLUGIN {
                    return Err(mlua::Error::runtime(format!(
                        "Too many background processes: maximum {} allowed per plugin",
                        MAX_BACKGROUND_PROCESSES_PER_PLUGIN
                    )));
                }

                match cmd.spawn() {
                    Ok(child) => {
                        let child_pid = child.id();
                        result_table.set("pid", child_pid)?;
                        result_table.set("success", true)?;

                        let entry = ProcessEntry {
                            owner_plugin_id: pid.clone(),
                            program: program.clone(),
                            child,
                            stdout_buf: Vec::new(),
                            started_at: Instant::now(),
                        };

                        procs.insert(child_pid, entry);
                    }
                    Err(e) => {
                        result_table.set("pid", 0)?;
                        result_table.set("success", false)?;
                        result_table.set("error", format!("{}", e))?;
                    }
                }
            } else {
                let started_at = Instant::now();
                match cmd.spawn() {
                    Ok(child) => match child.wait_with_output() {
                        Ok(output) => {
                            let elapsed = started_at.elapsed();
                            if elapsed > MAX_FOREGROUND_EXEC_DURATION {
                                return Err(mlua::Error::runtime(format!(
                                    "Process execution exceeded maximum duration of {} seconds",
                                    MAX_FOREGROUND_EXEC_DURATION.as_secs()
                                )));
                            }

                            let mut stdout = output.stdout;
                            truncate_output(&mut stdout);
                            result_table.set("pid", 0)?;
                            result_table.set("success", output.status.success())?;
                            result_table.set("exit_code", output.status.code().unwrap_or(-1))?;
                            let stdout_text = String::from_utf8_lossy(&stdout).into_owned();
                            result_table
                                .set("stdout", lua.create_string(stdout_text.as_bytes())?)?;
                            if stdout.len() >= MAX_STDOUT_BUFFER_BYTES {
                                result_table.set("truncated", true)?;
                            }
                        }
                        Err(e) => {
                            result_table.set("pid", 0)?;
                            result_table.set("success", false)?;
                            result_table.set("error", format!("{}", e))?;
                        }
                    },
                    Err(e) => {
                        result_table.set("pid", 0)?;
                        result_table.set("success", false)?;
                        result_table.set("error", format!("{}", e))?;
                    }
                }
            }

            Ok(result_table)
        },
    )
    .map_err(|e| format!("Failed to create process.exec: {}", e))
}

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

pub(super) fn register(
    lua: &Lua,
    process_table: &Table,
    plugin_dir: &std::path::Path,
    plugin_id: &str,
    permissions: &[String],
    process_registry: &ProcessRegistry,
) -> Result<(), String> {
    process_table
        .set("exec", exec(lua, plugin_dir, plugin_id, permissions, process_registry)?)
        .map_err(|e| format!("Failed to set process.exec: {}", e))?;
    process_table
        .set("get", get(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.get: {}", e))?;
    process_table
        .set("list", list(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.list: {}", e))?;
    process_table
        .set("read_output", read_output(lua, plugin_id, process_registry)?)
        .map_err(|e| format!("Failed to set process.read_output: {}", e))?;

    Ok(())
}
