use std::process::{Child, Command};

#[cfg(windows)]
use sysinfo::{Pid, ProcessesToUpdate, System};

#[cfg(unix)]
use std::collections::HashSet;

#[cfg(not(any(unix, windows)))]
use super::i18n::manager_t;
#[cfg(any(windows, not(any(unix, windows))))]
use super::i18n::manager_t1;

#[cfg(unix)]
fn list_child_pids_unix(ppid: u32) -> Vec<u32> {
    let output = Command::new("pgrep")
        .arg("-P")
        .arg(ppid.to_string())
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() || output.stdout.is_empty() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect()
}

#[cfg(unix)]
fn collect_descendant_pids_unix(root_pid: u32) -> Vec<u32> {
    let mut stack = vec![root_pid];
    let mut seen = HashSet::new();
    let mut descendants = Vec::new();

    while let Some(parent) = stack.pop() {
        for child in list_child_pids_unix(parent) {
            if seen.insert(child) {
                descendants.push(child);
                stack.push(child);
            }
        }
    }

    descendants
}

#[cfg(unix)]
fn is_process_alive_unix(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(unix)]
fn force_kill_process_tree_by_pid_unix(root_pid: u32) -> Result<(), String> {
    let mut pids = collect_descendant_pids_unix(root_pid);
    pids.push(root_pid);
    pids.sort_unstable();
    pids.dedup();

    for pid in pids.iter().rev() {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    for pid in pids.iter().rev() {
        if is_process_alive_unix(*pid) {
            let _ = Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status();
        }
    }

    Ok(())
}

#[cfg(unix)]
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    let root_pid = child.id();
    force_kill_process_tree_by_pid_unix(root_pid)?;
    let _ = child.wait();
    Ok(())
}

#[cfg(windows)]
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    let pid_str = child.id().to_string();
    let status = Command::new("taskkill")
        .args(["/PID", &pid_str, "/T", "/F"])
        .status()
        .map_err(|e| manager_t1("server.manager.process_taskkill_failed", e.to_string()))?;

    if !status.success() {
        let _ = child.kill();
    }
    let _ = child.wait();
    Ok(())
}

#[cfg(windows)]
fn is_process_alive_windows(pid: u32) -> bool {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
    system.process(Pid::from_u32(pid)).is_some()
}

pub(crate) fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        is_process_alive_unix(pid)
    }

    #[cfg(windows)]
    {
        is_process_alive_windows(pid)
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        false
    }
}

pub(crate) fn force_kill_process_tree_by_pid(pid: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        force_kill_process_tree_by_pid_unix(pid)
    }

    #[cfg(windows)]
    {
        let pid_str = pid.to_string();
        let status = Command::new("taskkill")
            .args(["/PID", &pid_str, "/T", "/F"])
            .status()
            .map_err(|e| manager_t1("server.manager.process_taskkill_failed", e.to_string()))?;
        if status.success() || !is_process_alive_windows(pid) {
            return Ok(());
        }

        Err(manager_t1(
            "server.manager.process_taskkill_pid_not_terminated",
            pid.to_string(),
        ))
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        Err(manager_t("server.manager.process_force_kill_platform_unsupported"))
    }
}

#[cfg(not(any(unix, windows)))]
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    child
        .kill()
        .map_err(|e| manager_t1("server.manager.process_kill_failed", e.to_string()))?;
    child
        .wait()
        .map(|_| ())
        .map_err(|e| manager_t1("server.manager.process_wait_failed", e.to_string()))
}
