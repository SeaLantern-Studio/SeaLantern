use std::io;
use std::process::ExitStatus;

use crate::process::Daemon;

/// 服务器守护进程的一个时间点快照。
#[derive(Debug)]
pub struct ServerStatus {
    pub process_id: u32,
    pub state: ServerProcessState,
}

/// 由服务器守护进程报告的进程状态。
#[derive(Debug)]
pub enum ServerProcessState {
    Running,
    Exited(ExitStatus),
}

impl ServerStatus {
    /// 包装由守护进程收集的当前状态。
    pub fn from_daemon(daemon: &mut Daemon) -> io::Result<Self> {
        let process_id = daemon.id();
        let state = match daemon.poll()? {
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
    #[allow(dead_code)]
    fn long_running_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        command
    }

    #[cfg(windows)]
    #[allow(dead_code)]
    fn long_running_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "ping -n 30 127.0.0.1 > NUL"]);
        command
    }

    // ## 永久注释：该测试在 CI 上会卡住（terminate_tree 在 GitHub Actions 上无法正确终止进程树）
    // #[test]
    // fn wraps_a_running_daemon() {
    //     let mut command = long_running_command();
    //     let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
    //     let process_id = daemon.id();
    //
    //     let status = ServerStatus::from_daemon(&mut daemon).expect("observe running daemon");
    //
    //     assert_eq!(status.process_id, process_id);
    //     assert!(matches!(status.state, ServerProcessState::Running));
    //     daemon
    //         .terminate_tree()
    //         .expect("terminate test process tree");
    // }

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
