use super::*;
use crate::plugins::api::new_api_registry;
use mlua::Value;
use mlua::{Function, Table};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PROCESS_OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;

fn create_test_runtime(permissions: Vec<&str>) -> PluginRuntime {
    let (runtime, _temp_dir) = create_test_runtime_with_root(permissions);
    runtime
}

fn create_test_runtime_with_root(permissions: Vec<&str>) -> (PluginRuntime, PathBuf) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let temp_dir =
        env::temp_dir().join(format!("sl_test_runtime_security_{}_{}", std::process::id(), now));
    let data_dir = temp_dir.join("data");
    let server_dir = temp_dir.join("servers");
    let global_dir = temp_dir.join("global");
    let api_registry = new_api_registry();

    fs::create_dir_all(&temp_dir).unwrap();

    let runtime = PluginRuntime::new(
        "test-runtime-security",
        &temp_dir,
        &data_dir,
        &server_dir,
        &global_dir,
        api_registry,
        permissions.into_iter().map(|p| p.to_string()).collect(),
    )
    .unwrap();

    (runtime, temp_dir)
}

fn cleanup_test_root(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_http_ssrf_rejects_localhost_variants() {
    assert!(http::validate_ssrf_url("http://localhost").is_err());
    assert!(http::validate_ssrf_url("http://localhost.").is_err());
    assert!(http::validate_ssrf_url("http://127.0.0.1").is_err());
    assert!(http::validate_ssrf_url("http://[::1]").is_err());
}

#[test]
fn test_http_ssrf_rejects_non_http_schemes() {
    assert!(http::validate_ssrf_url("file:///etc/passwd").is_err());
    assert!(http::validate_ssrf_url("ftp://example.com/file").is_err());
}

#[test]
fn test_json_conversion_rejects_non_finite_numbers() {
    let runtime = create_test_runtime(vec![]);
    let value = Value::Number(f64::NAN);
    let result = runtime.lua_eval::<Value>("return 1").ok();
    assert!(result.is_some());
    assert!(shared::json_value_from_lua(&value, 0).is_err());
}

#[test]
fn test_json_conversion_rejects_unsupported_types() {
    let runtime = create_test_runtime(vec![]);
    let function: mlua::Function = runtime.lua().create_function(|_, ()| Ok(())).unwrap();
    let value = Value::Function(function);
    assert!(shared::json_value_from_lua(&value, 0).is_err());
}

#[test]
fn test_collect_finished_processes_removes_exited_children() {
    let registry = process::new_process_registry();
    let child = spawn_quick_exit_child();
    let pid = child.id();

    {
        let mut procs = registry.lock().unwrap();
        procs.insert(
            pid,
            process::ProcessEntry {
                output: {
                    let output = process::new_process_output();
                    {
                        let mut state = output.lock().unwrap();
                        state.stdout_closed = true;
                        state.stderr_closed = true;
                    }
                    output
                },
                owner_plugin_id: "plugin-a".to_string(),
                program: "quick-exit".to_string(),
                child,
                started_at: Instant::now(),
            },
        );
    }

    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut procs = registry.lock().unwrap();
    process::collect_finished_processes(&mut procs);
    assert!(!procs.contains_key(&pid));
}

#[test]
fn test_kill_plugin_processes_only_kills_owned_processes() {
    let registry: process::ProcessRegistry = Arc::new(Mutex::new(HashMap::new()));
    let owned = spawn_sleep_child();
    let owned_pid = owned.id();
    let foreign = spawn_sleep_child();
    let foreign_pid = foreign.id();

    {
        let mut procs = registry.lock().unwrap();
        procs.insert(
            owned_pid,
            process::ProcessEntry {
                output: {
                    let output = process::new_process_output();
                    {
                        let mut state = output.lock().unwrap();
                        state.stdout_closed = true;
                        state.stderr_closed = true;
                    }
                    output
                },
                owner_plugin_id: "plugin-owned".to_string(),
                program: "sleep-owned".to_string(),
                child: owned,
                started_at: Instant::now(),
            },
        );
        procs.insert(
            foreign_pid,
            process::ProcessEntry {
                output: {
                    let output = process::new_process_output();
                    {
                        let mut state = output.lock().unwrap();
                        state.stdout_closed = true;
                        state.stderr_closed = true;
                    }
                    output
                },
                owner_plugin_id: "plugin-foreign".to_string(),
                program: "sleep-foreign".to_string(),
                child: foreign,
                started_at: Instant::now(),
            },
        );
    }

    process::kill_plugin_processes(&registry, "plugin-owned");

    let mut procs = registry.lock().unwrap();
    assert!(!procs.contains_key(&owned_pid));
    assert!(procs.contains_key(&foreign_pid));

    if let Some(mut entry) = procs.remove(&foreign_pid) {
        let _ = entry.child.kill();
        let _ = entry.child.wait();
    }
}

#[test]
fn test_plugins_legacy_permission_alias_mounts_plugins_namespace() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["plugins"]);

    let namespace_type: String = runtime
        .lua()
        .load(r#"return type(sl.plugins.list)"#)
        .eval()
        .unwrap();
    assert_eq!(namespace_type, "function");

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_plugins_canonical_permission_mounts_plugins_namespace() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["plugin_folder_access"]);

    let namespace_type: String = runtime
        .lua()
        .load(r#"return type(sl.plugins.list)"#)
        .eval()
        .unwrap();
    assert_eq!(namespace_type, "function");

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_exec_timeout_kills_and_reaps_foreground_process() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let pid_file = temp_dir.join("foreground-timeout.pid");
    let (program, args) = prepare_timeout_process_fixture(&temp_dir, &pid_file);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("timeout_ms", 400).unwrap();

    let started_at = Instant::now();
    let result: mlua::Result<Table> = exec.call((program, args, Some(options)));
    let elapsed = started_at.elapsed();

    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("exceeded maximum duration") || error_message.contains("timeout")
    );
    assert!(elapsed >= Duration::from_millis(250));
    assert!(elapsed < Duration::from_secs(5));

    let child_pid = wait_for_pid_file(&pid_file);
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        !process_exists(child_pid),
        "timed out child process {} should have been terminated",
        child_pid
    );

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_exec_returns_stderr_for_failed_foreground_process() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();

    let result: Table = exec.call((program, args, Option::<Table>::None)).unwrap();

    let success: bool = result.get("success").unwrap();
    let exit_code: i32 = result.get("exit_code").unwrap();
    let stdout: String = result.get("stdout").unwrap();
    let stderr: String = result.get("stderr").unwrap();

    assert!(!success);
    assert_eq!(exit_code, 7);
    assert_eq!(stdout, "");
    assert!(stderr.contains("boom from stderr"));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_exec_marks_stderr_truncated_when_limit_is_exceeded() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_large_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();

    let result: Table = exec.call((program, args, Option::<Table>::None)).unwrap();

    let success: bool = result.get("success").unwrap();
    let stderr: String = result.get("stderr").unwrap();
    let stderr_truncated: bool = result.get("stderr_truncated").unwrap();
    let has_stdout_truncated_flag: bool = runtime
        .lua()
        .load("return rawget(..., 'truncated') ~= nil")
        .call(result.clone())
        .unwrap();

    assert!(!success);
    assert!(stderr_truncated);
    assert_eq!(stderr.len(), PROCESS_OUTPUT_LIMIT_BYTES);
    assert!(!has_stdout_truncated_flag);
    assert!(stderr.ends_with("tail-marker"));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_read_output_returns_background_stdout_and_stderr() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_background_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let read_output: Function = process.get("read_output").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("background", true).unwrap();

    let launch_result: Table = exec.call((program, args, Some(options))).unwrap();
    let pid: u32 = launch_result.get("pid").unwrap();

    let output = wait_for_background_output(runtime.lua(), &read_output, pid).unwrap();
    let stdout: String = output.get("stdout").unwrap();
    let stderr: String = output.get("stderr").unwrap();

    assert!(stdout.contains("bg-out"));
    assert!(stderr.contains("bg-err"));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_read_output_preserves_stdout_only_contract_by_default() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_background_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let read_output: Function = process.get("read_output").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("background", true).unwrap();

    let launch_result: Table = exec.call((program, args, Some(options))).unwrap();
    let pid: u32 = launch_result.get("pid").unwrap();

    let stdout = wait_for_background_stdout(&read_output, pid).unwrap();
    assert!(stdout.contains("bg-out"));

    let output = wait_for_background_output(runtime.lua(), &read_output, pid).unwrap();
    let stdout_after_default: String = output.get("stdout").unwrap();
    let stderr: String = output.get("stderr").unwrap();

    assert_eq!(stdout_after_default, "");
    assert!(stderr.contains("bg-err"));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_read_output_marks_background_stderr_truncated() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_large_background_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let read_output: Function = process.get("read_output").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("background", true).unwrap();

    let launch_result: Table = exec.call((program, args, Some(options))).unwrap();
    let pid: u32 = launch_result.get("pid").unwrap();

    std::thread::sleep(Duration::from_millis(500));

    let output = wait_for_background_output(runtime.lua(), &read_output, pid).unwrap();
    let stderr: String = output.get("stderr").unwrap();
    let stderr_truncated: bool = output.get("stderr_truncated").unwrap();
    let has_stdout_truncated_flag: bool = runtime
        .lua()
        .load("return rawget(..., 'truncated') ~= nil")
        .call(output.clone())
        .unwrap();

    assert!(stderr_truncated);
    assert_eq!(stderr.len(), PROCESS_OUTPUT_LIMIT_BYTES);
    assert!(!has_stdout_truncated_flag);
    assert!(stderr.ends_with("bg-tail-marker"));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_read_output_keeps_exited_stderr_after_status_queries() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_background_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let read_output: Function = process.get("read_output").unwrap();
    let get: Function = process.get("get").unwrap();
    let list: Function = process.get("list").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("background", true).unwrap();

    let launch_result: Table = exec.call((program, args, Some(options))).unwrap();
    let pid: u32 = launch_result.get("pid").unwrap();

    let info = wait_for_background_process_exit(&get, pid).unwrap();
    let running: bool = info.get("running").unwrap();
    assert!(!running);

    let listed: Table = list.call(()).unwrap();
    let listed_pid: u32 = listed.get::<Table>(1).unwrap().get("pid").unwrap();
    assert_eq!(listed_pid, pid);

    let output = wait_for_background_output(runtime.lua(), &read_output, pid).unwrap();
    let stdout: String = output.get("stdout").unwrap();
    let stderr: String = output.get("stderr").unwrap();

    assert!(stdout.contains("bg-out"));
    assert!(stderr.contains("bg-err"));

    let final_read: mlua::Value = read_output
        .call((pid, background_output_options(runtime.lua())))
        .unwrap();
    assert!(matches!(final_read, mlua::Value::Nil));

    let post_drain: mlua::Value = get.call(pid).unwrap();
    assert!(matches!(post_drain, mlua::Value::Nil));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_read_output_include_stderr_false_keeps_stdout_only_behavior() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_background_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let read_output: Function = process.get("read_output").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("background", true).unwrap();

    let launch_result: Table = exec.call((program, args, Some(options))).unwrap();
    let pid: u32 = launch_result.get("pid").unwrap();

    let stdout_value = wait_for_background_value_with_options(
        runtime.lua(),
        &read_output,
        pid,
        stdout_only_output_options(runtime.lua()).as_ref(),
        |value| matches!(value, mlua::Value::String(_)),
    )
    .unwrap();

    let stdout = match stdout_value {
        mlua::Value::String(text) => text.to_str().unwrap().to_string(),
        other => panic!("expected stdout string, got {:?}", other.type_name()),
    };
    assert!(stdout.contains("bg-out"));

    let nil_value = wait_for_background_value_with_options(
        runtime.lua(),
        &read_output,
        pid,
        stdout_only_output_options(runtime.lua()).as_ref(),
        |value| matches!(value, mlua::Value::Nil),
    )
    .unwrap();
    assert!(matches!(nil_value, mlua::Value::Nil));

    let structured_value = wait_for_background_value_with_options(
        runtime.lua(),
        &read_output,
        pid,
        background_output_options(runtime.lua()).as_ref(),
        |value| matches!(value, mlua::Value::Table(_)),
    )
    .unwrap();

    let structured = match structured_value {
        mlua::Value::Table(table) => table,
        other => panic!("expected structured table, got {:?}", other.type_name()),
    };
    let stdout_after_default: String = structured.get("stdout").unwrap();
    let stderr: String = structured.get("stderr").unwrap();

    assert_eq!(stdout_after_default, "");
    assert!(stderr.contains("bg-err"));

    cleanup_test_root(&temp_dir);
}

#[test]
fn test_process_read_output_default_call_returns_nil_until_stderr_is_opted_in() {
    let (runtime, temp_dir) = create_test_runtime_with_root(vec!["execute_program"]);
    let (program, args) = prepare_background_stderr_process_fixture(&temp_dir);

    let sl: Table = runtime.lua().globals().get("sl").unwrap();
    let process: Table = sl.get("process").unwrap();
    let exec: Function = process.get("exec").unwrap();
    let read_output: Function = process.get("read_output").unwrap();
    let options = runtime.lua().create_table().unwrap();
    options.set("background", true).unwrap();

    let launch_result: Table = exec.call((program, args, Some(options))).unwrap();
    let pid: u32 = launch_result.get("pid").unwrap();

    let stdout = wait_for_background_stdout(&read_output, pid).unwrap();
    assert!(stdout.contains("bg-out"));

    let nil_value = wait_for_background_value_with_options(
        runtime.lua(),
        &read_output,
        pid,
        None,
        |value| matches!(value, mlua::Value::Nil),
    )
    .unwrap();
    assert!(matches!(nil_value, mlua::Value::Nil));

    let structured_value = wait_for_background_value_with_options(
        runtime.lua(),
        &read_output,
        pid,
        background_output_options(runtime.lua()).as_ref(),
        |value| matches!(value, mlua::Value::Table(_)),
    )
    .unwrap();

    let structured = match structured_value {
        mlua::Value::Table(table) => table,
        other => panic!("expected structured table, got {:?}", other.type_name()),
    };
    let stdout_after_default: String = structured.get("stdout").unwrap();
    let stderr: String = structured.get("stderr").unwrap();

    assert_eq!(stdout_after_default, "");
    assert!(stderr.contains("bg-err"));

    cleanup_test_root(&temp_dir);
}

#[cfg(windows)]
fn spawn_quick_exit_child() -> Child {
    Command::new("cmd")
        .args(["/C", "exit 0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[cfg(not(windows))]
fn spawn_quick_exit_child() -> Child {
    Command::new("sh")
        .args(["-c", "exit 0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[cfg(windows)]
fn spawn_sleep_child() -> Child {
    Command::new("cmd")
        .args(["/C", "ping 127.0.0.1 -n 6 >NUL"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[cfg(not(windows))]
fn spawn_sleep_child() -> Child {
    Command::new("sh")
        .args(["-c", "sleep 5"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[cfg(windows)]
fn prepare_timeout_process_fixture(temp_dir: &Path, pid_file: &Path) -> (String, Vec<String>) {
    let shell_src = PathBuf::from(env::var("SystemRoot").unwrap())
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let shell_dest = temp_dir.join("powershell.exe");
    fs::copy(&shell_src, &shell_dest).unwrap();

    let script_path = temp_dir.join("timeout-script.ps1");
    let script = format!(
        "$PID | Set-Content -LiteralPath '{}'\nStart-Sleep -Seconds 5\nWrite-Output 'finished'\n",
        pid_file.display()
    );
    fs::write(&script_path, script).unwrap();

    (
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            "timeout-script.ps1".to_string(),
        ],
    )
}

#[cfg(windows)]
fn prepare_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_src = PathBuf::from(env::var("SystemRoot").unwrap())
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let shell_dest = temp_dir.join("powershell.exe");
    fs::copy(&shell_src, &shell_dest).unwrap();

    let script_path = temp_dir.join("stderr-script.ps1");
    let script = "[Console]::Error.WriteLine('boom from stderr')\nexit 7\n";
    fs::write(&script_path, script).unwrap();

    (
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            "stderr-script.ps1".to_string(),
        ],
    )
}

#[cfg(windows)]
fn prepare_large_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_src = PathBuf::from(env::var("SystemRoot").unwrap())
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let shell_dest = temp_dir.join("powershell.exe");
    fs::copy(&shell_src, &shell_dest).unwrap();

    let script_path = temp_dir.join("stderr-large-script.ps1");
    let script = r#"
$payload = "x" * 1100000
[Console]::Error.Write($payload)
[Console]::Error.Write("tail-marker")
exit 9
"#;
    fs::write(&script_path, script).unwrap();

    (
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            "stderr-large-script.ps1".to_string(),
        ],
    )
}

#[cfg(windows)]
fn prepare_background_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_src = PathBuf::from(env::var("SystemRoot").unwrap())
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let shell_dest = temp_dir.join("powershell.exe");
    fs::copy(&shell_src, &shell_dest).unwrap();

    let script_path = temp_dir.join("background-stderr-script.ps1");
    let script = "Write-Output 'bg-out'\n[Console]::Error.WriteLine('bg-err')\nStart-Sleep -Milliseconds 200\n";
    fs::write(&script_path, script).unwrap();

    (
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            "background-stderr-script.ps1".to_string(),
        ],
    )
}

#[cfg(windows)]
fn prepare_large_background_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_src = PathBuf::from(env::var("SystemRoot").unwrap())
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    let shell_dest = temp_dir.join("powershell.exe");
    fs::copy(&shell_src, &shell_dest).unwrap();

    let script_path = temp_dir.join("background-stderr-large-script.ps1");
    let script = r#"
$payload = "x" * 1100000
[Console]::Error.Write($payload)
[Console]::Error.Write("bg-tail-marker")
Start-Sleep -Milliseconds 200
"#;
    fs::write(&script_path, script).unwrap();

    (
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            "background-stderr-large-script.ps1".to_string(),
        ],
    )
}

#[cfg(not(windows))]
fn prepare_timeout_process_fixture(temp_dir: &Path, pid_file: &Path) -> (String, Vec<String>) {
    let shell_dest = temp_dir.join("sh");
    fs::copy("/bin/sh", &shell_dest).unwrap();

    let script_path = temp_dir.join("timeout-script.sh");
    let script =
        format!("printf '%s' \"$$\" > '{}'\nsleep 5\nprintf 'finished\\n'\n", pid_file.display());
    fs::write(&script_path, script).unwrap();

    ("sh".to_string(), vec!["timeout-script.sh".to_string()])
}

#[cfg(not(windows))]
fn prepare_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_dest = temp_dir.join("sh");
    fs::copy("/bin/sh", &shell_dest).unwrap();

    let script_path = temp_dir.join("stderr-script.sh");
    let script = "printf 'boom from stderr\\n' >&2\nexit 7\n";
    fs::write(&script_path, script).unwrap();

    ("sh".to_string(), vec!["stderr-script.sh".to_string()])
}

#[cfg(not(windows))]
fn prepare_large_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_dest = temp_dir.join("sh");
    fs::copy("/bin/sh", &shell_dest).unwrap();

    let script_path = temp_dir.join("stderr-large-script.sh");
    let script = "python3 - <<'PY'\nimport sys\nsys.stderr.write('x' * 1100000)\nsys.stderr.write('tail-marker')\nsys.exit(9)\nPY\n";
    fs::write(&script_path, script).unwrap();

    ("sh".to_string(), vec!["stderr-large-script.sh".to_string()])
}

#[cfg(not(windows))]
fn prepare_background_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_dest = temp_dir.join("sh");
    fs::copy("/bin/sh", &shell_dest).unwrap();

    let script_path = temp_dir.join("background-stderr-script.sh");
    let script = "printf 'bg-out\\n'\nprintf 'bg-err\\n' >&2\nsleep 0.2\n";
    fs::write(&script_path, script).unwrap();

    ("sh".to_string(), vec!["background-stderr-script.sh".to_string()])
}

#[cfg(not(windows))]
fn prepare_large_background_stderr_process_fixture(temp_dir: &Path) -> (String, Vec<String>) {
    let shell_dest = temp_dir.join("sh");
    fs::copy("/bin/sh", &shell_dest).unwrap();

    let script_path = temp_dir.join("background-stderr-large-script.sh");
    let script = "python3 - <<'PY'\nimport sys, time\nsys.stderr.write('x' * 1100000)\nsys.stderr.write('bg-tail-marker')\ntime.sleep(0.2)\nPY\n";
    fs::write(&script_path, script).unwrap();

    ("sh".to_string(), vec!["background-stderr-large-script.sh".to_string()])
}

fn wait_for_background_output(lua: &mlua::Lua, read_output: &Function, pid: u32) -> Option<Table> {
    let deadline = Instant::now() + Duration::from_secs(3);

    let options = background_output_options(lua)?;

    while Instant::now() < deadline {
        let value: mlua::Value = read_output.call((pid, Some(options.clone()))).unwrap();
        if let mlua::Value::Table(table) = value {
            let stdout: String = table.get("stdout").unwrap();
            let stderr: String = table.get("stderr").unwrap();
            if !stdout.is_empty() || !stderr.is_empty() {
                return Some(table);
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    None
}

fn background_output_options(lua: &mlua::Lua) -> Option<Table> {
    let options = lua.create_table().ok()?;
    options.set("include_stderr", true).ok()?;
    Some(options)
}

fn stdout_only_output_options(lua: &mlua::Lua) -> Option<Table> {
    let options = lua.create_table().ok()?;
    options.set("include_stderr", false).ok()?;
    Some(options)
}

fn wait_for_background_value_with_options<P>(
    _lua: &mlua::Lua,
    read_output: &Function,
    pid: u32,
    options: Option<&Table>,
    predicate: P,
) -> Option<mlua::Value>
where
    P: Fn(&mlua::Value) -> bool,
{
    let deadline = Instant::now() + Duration::from_secs(3);
    let cloned_options = options.and_then(|table| Some(table.clone()));

    while Instant::now() < deadline {
        let value: mlua::Value = read_output.call((pid, cloned_options.clone())).unwrap();
        if predicate(&value) {
            return Some(value);
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    None
}

fn wait_for_background_stdout(read_output: &Function, pid: u32) -> Option<String> {
    let deadline = Instant::now() + Duration::from_secs(3);

    while Instant::now() < deadline {
        let value: mlua::Value = read_output.call((pid, Option::<Table>::None)).unwrap();
        if let mlua::Value::String(text) = value {
            let stdout = text.to_str().unwrap().to_string();
            if !stdout.is_empty() {
                return Some(stdout);
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    None
}

fn wait_for_background_process_exit(get: &Function, pid: u32) -> Option<Table> {
    let deadline = Instant::now() + Duration::from_secs(3);

    while Instant::now() < deadline {
        let value: mlua::Value = get.call(pid).unwrap();
        if let mlua::Value::Table(table) = value {
            let running: bool = table.get("running").unwrap();
            if !running {
                return Some(table);
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    None
}

fn wait_for_pid_file(pid_file: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(2);

    while Instant::now() < deadline {
        if let Ok(content) = fs::read_to_string(pid_file) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                return pid;
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    panic!("timed out waiting for pid file {}", pid_file.display());
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    !stdout.contains("No tasks are running") && stdout.contains(&format!("\"{}\"", pid))
}

#[cfg(not(windows))]
fn process_exists(pid: u32) -> bool {
    Command::new("sh")
        .args(["-c", &format!("kill -0 {} 2>/dev/null", pid)])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
