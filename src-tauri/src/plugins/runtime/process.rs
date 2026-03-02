use super::helpers::validate_path_static;
use super::PluginRuntime;
use mlua::Table;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub type ProcessRegistry = Arc<Mutex<HashMap<u32, ProcessEntry>>>;

pub struct ProcessEntry {
    pub program: String,
    pub child: Child,

    pub stdout_buf: Vec<u8>,
}

pub fn new_process_registry() -> ProcessRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn kill_all_processes(registry: &ProcessRegistry) {
    let mut procs = registry.lock().unwrap_or_else(|e| {
        eprintln!("[WARN] Process registry lock poisoned, recovering: {}", e);
        e.into_inner()
    });
    for (pid, entry) in procs.iter_mut() {
        if let Err(e) = entry.child.kill() {
            eprintln!("[WARN] Failed to kill process {} (pid {}): {}", entry.program, pid, e);
        }
    }
    procs.clear();
}

impl PluginRuntime {
    pub(super) fn setup_process_namespace(
        &self,
        sl: &Table,
        process_registry: ProcessRegistry,
    ) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;

        let process_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create process table: {}", e))?;

        let plugin_dir = self.plugin_dir.clone();
        let plugin_id = self.plugin_id.clone();
        let permissions = self.permissions.clone();

        let dir = plugin_dir.clone();
        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let registry = Arc::clone(&process_registry);
        let exec_fn =
            self.lua
                .create_function(
                    move |lua,
                          (program, args, options): (
                        String,
                        Option<Vec<String>>,
                        Option<Table>,
                    )| {
                        if !perms.iter().any(|p| p == "execute_program") {
                            return Err(mlua::Error::runtime(
                                "Permission denied: 'execute_program' permission required",
                            ));
                        }

                        let program_path = validate_path_static(&dir, &program)?;

                        if !program_path.exists() {
                            return Err(mlua::Error::runtime(format!(
                                "Program not found: {}",
                                program
                            )));
                        }

                        let _ = emit_permission_log(
                            &pid,
                            "api_call",
                            "sl.process.exec",
                            &format!("{} {:?}", program, args.as_deref().unwrap_or(&[])),
                        );

                        let args = args.unwrap_or_default();

                        let mut cwd = dir.clone();
                        let mut env_vars: Vec<(String, String)> = Vec::new();
                        let mut background = false;

                        if let Some(ref opts) = options {
                            if let Ok(cwd_str) = opts.get::<String>("cwd") {
                                cwd = validate_path_static(&dir, &cwd_str)?;
                            }
                            if let Ok(env_table) = opts.get::<Table>("env") {
                                for (k, v) in env_table.pairs::<String, String>().flatten() {
                                    env_vars.push((k, v));
                                }
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

                        match cmd.spawn() {
                            Ok(child) => {
                                let child_pid = child.id();
                                result_table.set("pid", child_pid)?;
                                result_table.set("success", true)?;

                                let entry = ProcessEntry {
                                    program: program.clone(),
                                    child,
                                    stdout_buf: Vec::new(),
                                };

                                let mut procs = registry.lock().unwrap_or_else(|e| {
                                    eprintln!("[WARN] Process registry lock poisoned: {}", e);
                                    e.into_inner()
                                });
                                procs.insert(child_pid, entry);

                                if !background {
                                    if let Some(entry) = procs.get_mut(&child_pid) {
                                        match entry.child.wait() {
                                            Ok(status) => {
                                                result_table.set(
                                                    "exit_code",
                                                    status.code().unwrap_or(-1),
                                                )?;
                                            }
                                            Err(e) => {
                                                result_table.set("error", format!("{}", e))?;
                                            }
                                        }

                                        if let Some(ref mut stdout) = entry.child.stdout {
                                            use std::io::Read;
                                            let mut buf = Vec::new();
                                            let _ = stdout.read_to_end(&mut buf);
                                            entry.stdout_buf = buf;
                                        }
                                    }

                                    procs.remove(&child_pid);
                                }
                            }
                            Err(e) => {
                                result_table.set("pid", 0)?;
                                result_table.set("success", false)?;
                                result_table.set("error", format!("{}", e))?;
                            }
                        }

                        Ok(result_table)
                    },
                )
                .map_err(|e| format!("Failed to create process.exec: {}", e))?;
        process_table
            .set("exec", exec_fn)
            .map_err(|e| format!("Failed to set process.exec: {}", e))?;

        let pid = plugin_id.clone();
        let registry = Arc::clone(&process_registry);
        let kill_fn = self
            .lua
            .create_function(move |_, target_pid: u32| {
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.process.kill",
                    &format!("pid={}", target_pid),
                );

                let mut procs = registry.lock().unwrap_or_else(|e| {
                    eprintln!("[WARN] Process registry lock poisoned: {}", e);
                    e.into_inner()
                });

                if let Some(entry) = procs.get_mut(&target_pid) {
                    match entry.child.kill() {
                        Ok(_) => {
                            procs.remove(&target_pid);
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
            .map_err(|e| format!("Failed to create process.kill: {}", e))?;
        process_table
            .set("kill", kill_fn)
            .map_err(|e| format!("Failed to set process.kill: {}", e))?;

        let pid = plugin_id.clone();
        let registry = Arc::clone(&process_registry);
        let get_fn = self
            .lua
            .create_function(move |lua, target_pid: u32| {
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.process.get",
                    &format!("pid={}", target_pid),
                );

                let mut procs = registry.lock().unwrap_or_else(|e| {
                    eprintln!("[WARN] Process registry lock poisoned: {}", e);
                    e.into_inner()
                });

                if let Some(entry) = procs.get_mut(&target_pid) {
                    let info = lua.create_table()?;
                    info.set("pid", target_pid)?;

                    match entry.child.try_wait() {
                        Ok(Some(status)) => {
                            info.set("running", false)?;
                            info.set("exit_code", status.code().unwrap_or(-1))?;
                        }
                        Ok(None) => {
                            info.set("running", true)?;
                        }
                        Err(e) => {
                            return Err(mlua::Error::runtime(format!(
                                "Failed to check process status: {}",
                                e
                            )));
                        }
                    }

                    Ok(mlua::Value::Table(info))
                } else {
                    Ok(mlua::Value::Nil)
                }
            })
            .map_err(|e| format!("Failed to create process.get: {}", e))?;
        process_table
            .set("get", get_fn)
            .map_err(|e| format!("Failed to set process.get: {}", e))?;

        let pid = plugin_id.clone();
        let registry = Arc::clone(&process_registry);
        let list_fn = self
            .lua
            .create_function(move |lua, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.process.list", "");

                let mut procs = registry.lock().unwrap_or_else(|e| {
                    eprintln!("[WARN] Process registry lock poisoned: {}", e);
                    e.into_inner()
                });

                let result = lua.create_table()?;
                let mut i = 1;

                let pids: Vec<u32> = procs.keys().cloned().collect();

                for proc_pid in pids {
                    if let Some(entry) = procs.get_mut(&proc_pid) {
                        let item = lua.create_table()?;
                        item.set("pid", proc_pid)?;
                        item.set("program", entry.program.clone())?;

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
            .map_err(|e| format!("Failed to create process.list: {}", e))?;
        process_table
            .set("list", list_fn)
            .map_err(|e| format!("Failed to set process.list: {}", e))?;

        let pid = plugin_id.clone();
        let registry = Arc::clone(&process_registry);
        let read_output_fn = self
            .lua
            .create_function(move |lua, target_pid: u32| {
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.process.read_output",
                    &format!("pid={}", target_pid),
                );

                let mut procs = registry.lock().unwrap_or_else(|e| {
                    eprintln!("[WARN] Process registry lock poisoned: {}", e);
                    e.into_inner()
                });

                if let Some(entry) = procs.get_mut(&target_pid) {
                    if let Some(ref mut stdout) = entry.child.stdout {
                        use std::io::Read;
                        let mut buf = [0u8; 8192];
                        match stdout.read(&mut buf) {
                            Ok(0) => {
                                if entry.stdout_buf.is_empty() {
                                    return Ok(mlua::Value::Nil);
                                }
                                let output = String::from_utf8_lossy(&entry.stdout_buf).to_string();
                                entry.stdout_buf.clear();
                                Ok(mlua::Value::String(lua.create_string(&output)?))
                            }
                            Ok(n) => {
                                entry.stdout_buf.extend_from_slice(&buf[..n]);
                                let output = String::from_utf8_lossy(&entry.stdout_buf).to_string();
                                entry.stdout_buf.clear();
                                Ok(mlua::Value::String(lua.create_string(&output)?))
                            }
                            Err(_) => Ok(mlua::Value::Nil),
                        }
                    } else {
                        Ok(mlua::Value::Nil)
                    }
                } else {
                    Ok(mlua::Value::Nil)
                }
            })
            .map_err(|e| format!("Failed to create process.read_output: {}", e))?;
        process_table
            .set("read_output", read_output_fn)
            .map_err(|e| format!("Failed to set process.read_output: {}", e))?;

        sl.set("process", process_table)
            .map_err(|e| format!("Failed to set sl.process: {}", e))?;

        Ok(())
    }
}
