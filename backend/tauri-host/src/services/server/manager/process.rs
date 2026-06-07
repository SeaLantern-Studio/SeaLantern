use std::process::{Child, Command};

#[cfg(unix)]
use std::collections::HashSet;

#[cfg(any(windows, not(any(unix, windows))))]
use super::i18n::manager_t1;
#[cfg(not(any(unix, windows)))]
use super::i18n::manager_t;

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

    tasklist_csv_has_pid(&String::from_utf8_lossy(&output.stdout), pid)
}

#[cfg(windows)]
fn tasklist_csv_has_pid(stdout: &str, pid: u32) -> bool {
    let expected_pid = pid.to_string();

    stdout.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('"') {
            return false;
        }

        let mut fields = trimmed.split("\",\"");
        let _image_name = fields.next();
        let pid_field = fields.next().map(|field| field.trim_matches('"'));

        pid_field.is_some_and(|value| value == expected_pid)
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

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::tasklist_csv_has_pid;

    #[cfg(windows)]
    #[test]
    fn tasklist_csv_has_pid_matches_actual_csv_rows() {
        let stdout = "\"java.exe\",\"25212\",\"Console\",\"1\",\"512,000 K\"\r\n";

        assert!(tasklist_csv_has_pid(stdout, 25212));
        assert!(!tasklist_csv_has_pid(stdout, 99999));
    }

    #[cfg(windows)]
    #[test]
    fn tasklist_csv_has_pid_rejects_localized_no_match_text() {
        let stdout = "INFO: 没有运行的任务匹配指定标准。\r\n";

        assert!(!tasklist_csv_has_pid(stdout, 25212));
    }
}
