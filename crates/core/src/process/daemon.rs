use std::io;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus};

/// Owns a server process and provides its lifecycle operations.
pub struct Daemon {
    child: Child,
}

impl Daemon {
    /// Spawns a daemon from a fully configured command.
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        command.spawn().map(Self::from_child)
    }

    /// Wraps an already spawned child process.
    pub fn from_child(child: Child) -> Self {
        Self { child }
    }

    /// Returns the operating-system process identifier.
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Polls the child without blocking.
    ///
    /// `Ok(None)` means the daemon is still running. `Ok(Some(_))` contains its exit status.
    pub fn poll(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    /// Waits until the daemon exits.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }

    /// Transfers standard input ownership to the terminal module.
    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    /// Transfers standard output ownership to the terminal module.
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    /// Transfers standard error ownership to the terminal module.
    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    /// Terminates the daemon and every process it started.
    pub fn terminate_tree(&mut self) -> io::Result<()> {
        if self.poll()?.is_some() {
            return Ok(());
        }

        terminate_process_tree(&mut self.child)
    }
}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) -> io::Result<()> {
    use std::collections::HashSet;

    fn direct_children(parent_pid: u32) -> Vec<u32> {
        let output = Command::new("pgrep")
            .arg("-P")
            .arg(parent_pid.to_string())
            .output();
        let Ok(output) = output else {
            return Vec::new();
        };

        if !output.status.success() {
            return Vec::new();
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect()
    }

    fn descendants(root_pid: u32) -> Vec<u32> {
        let mut pending = vec![root_pid];
        let mut seen = HashSet::new();
        let mut descendants = Vec::new();

        while let Some(parent_pid) = pending.pop() {
            for child_pid in direct_children(parent_pid) {
                if seen.insert(child_pid) {
                    descendants.push(child_pid);
                    pending.push(child_pid);
                }
            }
        }

        descendants
    }

    fn is_alive(pid: u32) -> bool {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    let root_pid = child.id();
    let mut pids = descendants(root_pid);
    pids.push(root_pid);
    pids.sort_unstable();
    pids.dedup();

    for pid in pids.iter().rev() {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }

    std::thread::sleep(std::time::Duration::from_millis(300));

    for pid in pids.iter().rev().filter(|pid| is_alive(**pid)) {
        let _ = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status();
    }

    child.wait().map(|_| ())
}

#[cfg(windows)]
fn terminate_process_tree(child: &mut Child) -> io::Result<()> {
    let status = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .status()?;

    if !status.success() {
        child.kill()?;
    }

    child.wait().map(|_| ())
}

#[cfg(not(any(unix, windows)))]
fn terminate_process_tree(child: &mut Child) -> io::Result<()> {
    child.kill()?;
    child.wait().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn exit_successfully_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "exit 0"]);
        command
    }

    #[cfg(windows)]
    fn exit_successfully_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "exit 0"]);
        command
    }

    #[test]
    fn reports_the_exit_status_of_a_finished_daemon() {
        let mut command = exit_successfully_command();
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");

        assert!(daemon.wait().expect("wait for test process").success());
        assert!(daemon.poll().expect("poll test process").is_some());
    }
}
