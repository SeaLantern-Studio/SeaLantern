use std::io;
use std::process::ExitStatus;

use crate::process::Daemon;

/// A point-in-time observation of a server daemon.
#[derive(Debug)]
pub struct ServerStatus {
    pub process_id: u32,
    pub state: ServerProcessState,
}

/// The process state reported by a server daemon.
#[derive(Debug)]
pub enum ServerProcessState {
    Running,
    Exited(ExitStatus),
}

impl ServerStatus {
    /// Wraps the current state collected by a daemon.
    pub fn from_daemon(daemon: &mut Daemon) -> io::Result<Self> {
        let process_id = daemon.id();
        let state = match daemon.poll().map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("could not observe daemon process {process_id}: {error}"),
            )
        })? {
            Some(exit_status) => ServerProcessState::Exited(exit_status),
            None => ServerProcessState::Running,
        };

        Ok(Self { state, process_id })
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::{ServerProcessState, ServerStatus};
    use crate::process::Daemon;

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
    fn long_running_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        command
    }

    #[cfg(windows)]
    fn long_running_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "ping -n 30 127.0.0.1 > NUL"]);
        command
    }

    #[test]
    fn wraps_a_running_daemon() {
        let mut command = long_running_command();
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let process_id = daemon.id();

        let status = ServerStatus::from_daemon(&mut daemon).expect("observe running daemon");

        assert_eq!(status.process_id, process_id);
        assert!(matches!(status.state, ServerProcessState::Running));
        daemon
            .terminate_tree()
            .expect("terminate test process tree");
    }

    #[test]
    fn wraps_a_finished_daemon_exit_status() {
        let mut command = exit_successfully_command();
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let _ = daemon.wait().expect("wait for test process");

        let status = ServerStatus::from_daemon(&mut daemon).expect("observe finished daemon");

        assert!(matches!(
            status.state,
            ServerProcessState::Exited(exit_status) if exit_status.success()
        ));
    }
}
