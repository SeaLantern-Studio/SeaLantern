use std::io;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus};

/// Owns a server process and provides its lifecycle operations.
pub struct Daemon {
    child: Child,
}

/// Identifies an abnormal daemon-termination outcome for callers and logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonTerminationSign {
    ProcessAlreadyExited,
    ProcessStateUnknown,
    GracefulTerminationFailed,
    ForcedTerminationFailed,
}

impl DaemonTerminationSign {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProcessAlreadyExited => "process_already_exited",
            Self::ProcessStateUnknown => "process_state_unknown",
            Self::GracefulTerminationFailed => "graceful_termination_failed",
            Self::ForcedTerminationFailed => "forced_termination_failed",
        }
    }
}

/// Describes a failed or anomalous daemon-termination attempt.
#[derive(Debug)]
pub struct DaemonTerminationError {
    sign: DaemonTerminationSign,
    process_id: u32,
    message: String,
    source: Option<io::Error>,
}

impl DaemonTerminationError {
    pub fn sign(&self) -> DaemonTerminationSign {
        self.sign
    }
}

impl std::fmt::Display for DaemonTerminationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "daemon process {} termination sign {}: {}",
            self.process_id,
            self.sign.as_str(),
            self.message
        )?;

        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }

        Ok(())
    }
}

impl std::error::Error for DaemonTerminationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl Daemon {
    /// Spawns a daemon from a fully configured command.
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            command.process_group(0);
        }

        command.spawn().map(|child| Self { child })
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

    /// Terminates the daemon and every process in its process tree.
    ///
    /// If the child has already exited or cannot be inspected, a force-termination attempt is still
    /// made and the method returns an error carrying an abnormal termination sign.
    pub fn terminate_tree(&mut self) -> Result<(), DaemonTerminationError> {
        let process_id = self.id();
        match self.poll() {
            Ok(None) => terminate_running_process_tree(&mut self.child).map_err(|source| {
                termination_error(
                    DaemonTerminationSign::GracefulTerminationFailed,
                    process_id,
                    "the running process tree could not be terminated",
                    Some(source),
                )
            }),
            Ok(Some(status)) => self.force_after_abnormal_state(
                DaemonTerminationSign::ProcessAlreadyExited,
                format!("the process had already exited with status {status}"),
            ),
            Err(source) => self.force_after_abnormal_state(
                DaemonTerminationSign::ProcessStateUnknown,
                format!("the process state could not be inspected: {source}"),
            ),
        }
    }

    fn force_after_abnormal_state(
        &mut self,
        sign: DaemonTerminationSign,
        state_message: String,
    ) -> Result<(), DaemonTerminationError> {
        let process_id = self.id();
        match force_terminate_process_tree(&mut self.child) {
            Ok(()) => Err(termination_error(
                sign,
                process_id,
                format!("{state_message}; force termination completed"),
                None,
            )),
            Err(source) => Err(termination_error(
                DaemonTerminationSign::ForcedTerminationFailed,
                process_id,
                format!("{state_message}; force termination failed"),
                Some(source),
            )),
        }
    }
}

fn termination_error(
    sign: DaemonTerminationSign,
    process_id: u32,
    message: impl Into<String>,
    source: Option<io::Error>,
) -> DaemonTerminationError {
    let error = DaemonTerminationError {
        sign,
        process_id,
        message: message.into(),
        source,
    };
    eprintln!("[sealantern-core][daemon] {error}");
    error
}

#[cfg(unix)]
fn terminate_running_process_tree(child: &mut Child) -> io::Result<()> {
    let process_group_id = child.id();
    signal_process_group(process_group_id, "TERM")?;
    std::thread::sleep(std::time::Duration::from_millis(300));

    if child.try_wait()?.is_none() {
        signal_process_group(process_group_id, "KILL")?;
    }

    child.wait().map(|_| ())
}

#[cfg(unix)]
fn force_terminate_process_tree(child: &mut Child) -> io::Result<()> {
    signal_process_group(child.id(), "KILL")?;
    child.wait().map(|_| ())
}

#[cfg(unix)]
fn signal_process_group(process_group_id: u32, signal: &str) -> io::Result<()> {
    let status = Command::new("kill")
        .args([format!("-{signal}"), format!("-{process_group_id}")])
        .status()
        .map_err(|source| {
            io::Error::new(
                source.kind(),
                format!("could not send {signal} to process group {process_group_id}: {source}"),
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "sending {signal} to process group {process_group_id} exited with {status}"
        )))
    }
}

#[cfg(windows)]
fn terminate_running_process_tree(child: &mut Child) -> io::Result<()> {
    terminate_windows_process_tree(child)
}

#[cfg(windows)]
fn force_terminate_process_tree(child: &mut Child) -> io::Result<()> {
    terminate_windows_process_tree(child)
}

#[cfg(windows)]
fn terminate_windows_process_tree(child: &mut Child) -> io::Result<()> {
    let status = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "taskkill failed for process tree {} with {status}",
            child.id()
        )));
    }

    child.wait().map(|_| ())
}

#[cfg(not(any(unix, windows)))]
fn terminate_running_process_tree(child: &mut Child) -> io::Result<()> {
    child.kill()?;
    child.wait().map(|_| ())
}

#[cfg(not(any(unix, windows)))]
fn force_terminate_process_tree(child: &mut Child) -> io::Result<()> {
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

    #[cfg(unix)]
    fn long_running_tree_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30 & wait"]);
        command
    }

    #[cfg(windows)]
    fn long_running_tree_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "ping -n 30 127.0.0.1 > NUL"]);
        command
    }

    #[test]
    fn reports_the_exit_status_of_a_finished_daemon() {
        let mut command = exit_successfully_command();
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");

        assert!(daemon.wait().expect("wait for test process").success());
        assert!(daemon.poll().expect("poll test process").is_some());
    }

    #[test]
    fn reports_an_abnormal_sign_for_an_exited_daemon() {
        let mut command = exit_successfully_command();
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let _ = daemon.wait().expect("wait for test process");

        let error = daemon
            .terminate_tree()
            .expect_err("an exited daemon must report an abnormal sign");
        assert!(matches!(
            error.sign(),
            DaemonTerminationSign::ProcessAlreadyExited
                | DaemonTerminationSign::ForcedTerminationFailed
        ));
    }

    #[test]
    fn terminates_a_running_process_tree() {
        let mut command = long_running_tree_command();
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process tree");

        daemon
            .terminate_tree()
            .expect("terminate running process tree");
        assert!(daemon.poll().expect("poll terminated process").is_some());
    }
}
