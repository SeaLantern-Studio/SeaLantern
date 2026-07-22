use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

use sealantern_core::process::{
    Daemon, DaemonTerminationError, Terminal, TerminalOutput, TerminalStream, TerminalWriteError,
};

/// 一个拥有服务器子进程及其终端流的运行时服务。
///
/// 该服务不依赖 Tauri、HTTP 或事件传输。宿主负责构建命令、持久化配置和消费输出，
/// 服务只保证进程和终端流以 core 定义的所有权模型协作。
pub struct ServerRuntime {
    daemon: Daemon,
    terminal: Terminal,
}

impl ServerRuntime {
    /// 启动服务器进程，并为控制台和日志消费者保留标准流。
    pub fn spawn(command: &mut Command, accepts_console_input: bool) -> io::Result<Self> {
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut daemon = Daemon::spawn(command)?;
        let terminal = Terminal::from_daemon_with_input(&mut daemon, accepts_console_input);
        Ok(Self { daemon, terminal })
    }

    /// 返回操作系统进程标识符。
    pub fn id(&self) -> u32 {
        self.daemon.id()
    }

    /// 非阻塞地查询服务器是否已退出。
    pub fn poll(&mut self) -> io::Result<Option<ExitStatus>> {
        self.daemon.poll()
    }

    /// 等待服务器进程退出。
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.daemon.wait()
    }

    /// 将一行命令写入服务器控制台。
    pub fn send_console_command(&mut self, command: &str) -> Result<(), TerminalWriteError> {
        self.terminal.write_line(command)
    }

    /// 将一个输出流交给宿主的日志读取器。
    pub fn take_output(&mut self, stream: TerminalStream) -> Option<TerminalOutput> {
        self.terminal.take_output(stream)
    }

    /// 使用默认的有界宽限期终止服务器进程树。
    pub fn terminate_tree(&mut self) -> Result<(), DaemonTerminationError> {
        self.daemon.terminate_tree()
    }

    /// 立即终止服务器进程树，供确认后的强制停止操作使用。
    pub fn force_terminate_tree(&mut self) -> Result<(), DaemonTerminationError> {
        self.daemon.terminate_tree_with_timeout(Duration::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[cfg(unix)]
    fn command_reader() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "read line; printf '%s' \"$line\""]);
        command
    }

    #[cfg(windows)]
    fn command_reader() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/V:ON", "/C", "set /p line=& echo !line!"]);
        command
    }

    #[test]
    fn takes_server_output_for_a_host_reader() {
        let mut command = command_reader();
        let mut runtime = ServerRuntime::spawn(&mut command, true).expect("spawn server runtime");

        runtime
            .send_console_command("say hello")
            .expect("write console command");
        let mut output = runtime
            .take_output(TerminalStream::Stdout)
            .expect("stdout should be available");
        let mut text = String::new();
        output.read_to_string(&mut text).expect("read stdout");

        assert!(runtime.wait().expect("wait for server process").success());
        assert!(text.contains("say hello"));
    }
}
