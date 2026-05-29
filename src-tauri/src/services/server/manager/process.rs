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
        .map_err(|e| format!("执行 taskkill 失败: {}", e))?;

    if !status.success() {
        let _ = child.kill();
    }
    let _ = child.wait();
    Ok(())
}

#[cfg(windows)]
fn is_process_alive_windows(pid: u32) -> bool {
    let filter = format!("PID eq {}", pid);
    let output = Command::new("tasklist")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output();

    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.contains("No tasks are running")
    })
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
        return force_kill_process_tree_by_pid_unix(pid);
    }

    #[cfg(windows)]
    {
        let pid_str = pid.to_string();
        let status = Command::new("taskkill")
            .args(["/PID", &pid_str, "/T", "/F"])
            .status()
            .map_err(|e| format!("执行 taskkill 失败: {}", e))?;
        if status.success() || !is_process_alive_windows(pid) {
            return Ok(());
        }

        Err(format!("taskkill 未能终止 PID {}", pid))
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        Err("当前平台暂不支持按 PID 强制终止进程树".to_string())
    }
}

#[cfg(not(any(unix, windows)))]
pub(super) fn force_kill_process_tree(child: &mut Child) -> Result<(), String> {
    child.kill().map_err(|e| format!("终止进程失败: {}", e))?;
    child
        .wait()
        .map(|_| ())
        .map_err(|e| format!("等待进程退出失败: {}", e))
}
