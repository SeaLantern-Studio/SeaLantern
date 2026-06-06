use super::shared::{
    collect_env_vars, emit_process_log, mask_args_for_log, validate_args, validate_program_path,
};
use crate::plugins::runtime::process::common::{
    collect_finished_processes, plugin_process_count, truncate_output, ProcessEntry,
    ProcessRegistry, MAX_ARGS_COUNT, MAX_ARG_LENGTH, MAX_BACKGROUND_PROCESSES_PER_PLUGIN,
    MAX_ENV_KEY_LENGTH, MAX_ENV_VALUE_LENGTH, MAX_ENV_VARS, MAX_FOREGROUND_EXEC_DURATION,
    MAX_STDOUT_BUFFER_BYTES,
};
use crate::plugins::runtime::shared::validate_path_static;
use mlua::{Function, Lua, Table};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

/// 注册 `sl.process.exec`
///
/// # Parameters
///
/// - `lua`: 当前 Lua 实例
/// - `plugin_dir`: 当前插件目录
/// - `plugin_id`: 当前插件 ID
/// - `permissions`: 当前插件权限列表
/// - `process_registry`: 共用进程注册表
///
/// # Returns
///
/// 返回一个可在 Lua 里执行程序的函数
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
            validate_args(&args, MAX_ARGS_COUNT, MAX_ARG_LENGTH)?;

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
                    env_vars = collect_env_vars(
                        env_table,
                        MAX_ENV_VARS,
                        MAX_ENV_KEY_LENGTH,
                        MAX_ENV_VALUE_LENGTH,
                    )?;
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
