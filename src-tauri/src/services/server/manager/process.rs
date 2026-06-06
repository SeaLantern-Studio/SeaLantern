use std::process::{Child, Command};

#[cfg(unix)]
use std::collections::HashSet;

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
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    let root_pid = child.id();
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

    let _ = child.wait();
    Ok(())
}

#[cfg(windows)]
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    let pid_str = child.id().to_string();
    let status = Command::new("taskkill")
        .args(["/PID", &pid_str, "/T", "/F"])
        .status()
        .map_err(|e| format!("执行 taskkill 失败: {}", e))?;

    if !status.success() {
        let _ = child.kill();
    }
    let _ = child.wait();
    Ok(())
}

#[cfg(not(any(unix, windows)))]
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    child.kill().map_err(|e| format!("终止进程失败: {}", e))?;
    child
        .wait()
        .map(|_| ())
        .map_err(|e| format!("等待进程退出失败: {}", e))
}
