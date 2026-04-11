use super::*;
use crate::plugins::api::new_api_registry;
use mlua::Value;
use std::collections::HashMap;
use std::env;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn create_test_runtime(permissions: Vec<&str>) -> PluginRuntime {
    let temp_dir = env::temp_dir().join("sl_test_runtime_security");
    let data_dir = temp_dir.join("data");
    let server_dir = temp_dir.join("servers");
    let global_dir = temp_dir.join("global");
    let api_registry = new_api_registry();

    PluginRuntime::new(
        "test-runtime-security",
        &temp_dir,
        &data_dir,
        &server_dir,
        &global_dir,
        api_registry,
        permissions.into_iter().map(|p| p.to_string()).collect(),
    )
    .unwrap()
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
                owner_plugin_id: "plugin-a".to_string(),
                program: "quick-exit".to_string(),
                child,
                stdout_buf: Vec::new(),
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
                owner_plugin_id: "plugin-owned".to_string(),
                program: "sleep-owned".to_string(),
                child: owned,
                stdout_buf: Vec::new(),
                started_at: Instant::now(),
            },
        );
        procs.insert(
            foreign_pid,
            process::ProcessEntry {
                owner_plugin_id: "plugin-foreign".to_string(),
                program: "sleep-foreign".to_string(),
                child: foreign,
                stdout_buf: Vec::new(),
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
