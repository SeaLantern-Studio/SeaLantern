use super::shared::{
    collect_env_vars, emit_process_denied_log, emit_process_log, mask_args_for_log,
    validate_args, validate_program_path,
};
use crate::plugins::runtime::process::common::{
    collect_finished_processes, new_process_output, plugin_process_count, process_err,
    process_err1, process_err2, process_msg1, process_msg2, spawn_background_pipe_reader,
    truncate_output, ProcessEntry, ProcessStream, MAX_ARGS_COUNT, MAX_ARG_LENGTH,
    MAX_BACKGROUND_PROCESSES_PER_PLUGIN, MAX_ENV_KEY_LENGTH, MAX_ENV_VALUE_LENGTH, MAX_ENV_VARS,
    MAX_FOREGROUND_EXEC_DURATION,
};
use crate::plugins::runtime::shared::validate_path_static;
use mlua::{Function, Lua, Table};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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
/// 返回一个可在 Lua 里执行程序的函数。
///
/// 前台执行成功创建结果表后，始终会返回：
/// - `pid = 0`
/// - `success: boolean`
/// - `exit_code: integer`
/// - `stdout: string`
/// - `stderr: string`
///
/// 当某个输出流被截断到 `MAX_STDOUT_BUFFER_BYTES` 上限时，会额外返回：
/// - `truncated = true` 表示 `stdout` 被截断
/// - `stderr_truncated = true` 表示 `stderr` 被截断
pub(super) fn exec(
    lua: &Lua,
    plugin_dir: &std::path::Path,
    plugin_id: &str,
    permissions: &[String],
    allowed_programs: &HashSet<PathBuf>,
    process_registry: &Arc<Mutex<HashMap<u32, ProcessEntry>>>,
) -> Result<Function, String> {
    let dir = plugin_dir.to_path_buf();
    let pid = plugin_id.to_string();
    let perms = permissions.to_vec();
    let declared_programs = allowed_programs.clone();
    let registry = Arc::clone(process_registry);

    lua.create_function(
        move |lua, (program, args, options): (String, Option<Vec<String>>, Option<Table>)| {
            if !perms.iter().any(|p| p == "execute_program") {
                return Err(process_err(
                    "plugins.runtime.process.execute_permission_required",
                ));
            }

            let program_path = validate_path_static(&dir, &program)?;
            validate_program_path(&program_path, &program)?;

            if declared_programs.is_empty() {
                emit_process_denied_log(
                    &pid,
                    "sl.process.exec",
                    &format!("program={} reason=manifest_missing_programs", program),
                );
                return Err(process_err(
                    "plugins.runtime.process.manifest_programs_required",
                ));
            }

            if !declared_programs.contains(&program_path) {
                emit_process_denied_log(
                    &pid,
                    "sl.process.exec",
                    &format!("program={} reason=program_not_declared", program),
                );
                return Err(process_err1(
                    "plugins.runtime.process.program_not_declared",
                    program,
                ));
            }

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
            let mut timeout = MAX_FOREGROUND_EXEC_DURATION;

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
                if let Ok(timeout_ms) = opts.get::<u64>("timeout_ms") {
                    if timeout_ms == 0 {
                        return Err(process_err(
                            "plugins.runtime.process.timeout_must_be_positive",
                        ));
                    }

                    let requested_timeout = Duration::from_millis(timeout_ms);
                    timeout = requested_timeout.min(MAX_FOREGROUND_EXEC_DURATION);
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

                if plugin_process_count(&mut procs, &pid) >= MAX_BACKGROUND_PROCESSES_PER_PLUGIN {
                    return Err(process_err1(
                        "plugins.runtime.process.too_many_background_processes",
                        MAX_BACKGROUND_PROCESSES_PER_PLUGIN.to_string(),
                    ));
                }

                match cmd.spawn() {
                    Ok(mut child) => {
                        let child_pid = child.id();
                        result_table.set("pid", child_pid)?;
                        result_table.set("success", true)?;

                        let output = new_process_output();
                        if let Some(stdout) = child.stdout.take() {
                            spawn_background_pipe_reader(
                                stdout,
                                Arc::clone(&output),
                                ProcessStream::Stdout,
                            );
                        }
                        if let Some(stderr) = child.stderr.take() {
                            spawn_background_pipe_reader(
                                stderr,
                                Arc::clone(&output),
                                ProcessStream::Stderr,
                            );
                        }

                        let entry = ProcessEntry {
                            owner_plugin_id: pid.clone(),
                            program: program.clone(),
                            child,
                            output,
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
                    Ok(mut child) => {
                        let stdout_reader = child.stdout.take().map(spawn_pipe_reader);
                        let stderr_reader = child.stderr.take().map(spawn_pipe_reader);

                        let mut timed_out = false;
                        let exit_status = loop {
                            match child.try_wait() {
                                Ok(Some(status)) => break status,
                                Ok(None) => {
                                    if started_at.elapsed() >= timeout {
                                        timed_out = true;

                                        if let Err(kill_error) = child.kill() {
                                            match child.try_wait() {
                                                Ok(Some(status)) => break status,
                                                Ok(None) => {
                                                    return Err(process_err1(
                                                        "plugins.runtime.process.terminate_timed_out_failed",
                                                        kill_error.to_string(),
                                                    ));
                                                }
                                                Err(wait_error) => {
                                                    return Err(process_err1(
                                                        "plugins.runtime.process.inspect_timed_out_failed",
                                                        wait_error.to_string(),
                                                    ));
                                                }
                                            }
                                        }

                                        break child.wait().map_err(|e| {
                                            process_err1(
                                                "plugins.runtime.process.wait_timed_out_failed",
                                                e.to_string(),
                                            )
                                        })?;
                                    }

                                    thread::sleep(Duration::from_millis(25));
                                }
                                Err(e) => {
                                    return Err(process_err1(
                                        "plugins.runtime.process.wait_failed",
                                        e.to_string(),
                                    ));
                                }
                            }
                        };

                        let mut stdout = join_pipe_reader(stdout_reader, "stdout")?;
                        let mut stderr = join_pipe_reader(stderr_reader, "stderr")?;

                        if timed_out {
                            return Err(mlua::Error::runtime(timeout_error_message(timeout)));
                        }

                        let stdout_truncated = truncate_output(&mut stdout);
                        let stderr_truncated = truncate_output(&mut stderr);
                        result_table.set("pid", 0)?;
                        result_table.set("success", exit_status.success())?;
                        result_table.set("exit_code", exit_status.code().unwrap_or(-1))?;
                        let stdout_text = String::from_utf8_lossy(&stdout).into_owned();
                        let stderr_text = String::from_utf8_lossy(&stderr).into_owned();
                        result_table.set("stdout", lua.create_string(stdout_text.as_bytes())?)?;
                        result_table.set("stderr", lua.create_string(stderr_text.as_bytes())?)?;
                        if stdout_truncated {
                            result_table.set("truncated", true)?;
                        }
                        if stderr_truncated {
                            result_table.set("stderr_truncated", true)?;
                        }
                    }
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
    .map_err(|e| process_msg2("plugins.runtime.process.create_api_failed", "process.exec", e.to_string()))
}

fn spawn_pipe_reader<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    })
}

fn join_pipe_reader(
    handle: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
    stream_name: &str,
) -> mlua::Result<Vec<u8>> {
    let Some(handle) = handle else {
        return Ok(Vec::new());
    };

    match handle.join() {
        Ok(Ok(buffer)) => Ok(buffer),
        Ok(Err(error)) => Err(process_err2(
            "plugins.runtime.process.read_stream_failed",
            stream_name,
            error.to_string(),
        )),
        Err(_) => Err(process_err1("plugins.runtime.process.reader_thread_panicked", stream_name)),
    }
}

fn timeout_error_message(timeout: Duration) -> String {
    if timeout.as_millis().is_multiple_of(1000) {
        process_msg1(
            "plugins.runtime.process.execution_timeout_seconds",
            timeout.as_secs().to_string(),
        )
    } else {
        process_msg1(
            "plugins.runtime.process.execution_timeout_millis",
            timeout.as_millis().to_string(),
        )
    }
}
