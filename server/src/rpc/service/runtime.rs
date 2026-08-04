//! 直接使用 core 进程能力的受管服务器运行时。

use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

use sealantern_core::process::{
    CommandBuildRequest, Daemon, DaemonTerminationError, Terminal, TerminalOutput, TerminalStream,
    TerminalWriteError,
};

/// 一个拥有服务器子进程及其终端流的运行时服务。
///
/// 该服务直接使用 core 的进程与终端模型。控制台输入是否可用只能由
/// [`CommandBuildRequest`] 的启动模式推导，避免将命令写入 shell 或自定义启动包装进程的
/// stdin。
pub struct ServerRuntime {
    daemon: Daemon,
    terminal: Terminal,
}

impl ServerRuntime {
    /// 使用调用方配置的标准流启动服务器进程。
    pub fn spawn(command: &mut Command, request: &CommandBuildRequest<'_>) -> io::Result<Self> {
        let daemon = Daemon::spawn(command)?;
        Ok(Self::from_daemon(daemon, request))
    }

    /// 使用 piped 标准流启动服务器进程。
    ///
    /// shell 和自定义启动模式即使拥有 piped stdin，仍会由 core 的输入策略移除控制台写入
    /// 能力。
    pub fn spawn_with_piped_stdio(
        command: &mut Command,
        request: &CommandBuildRequest<'_>,
    ) -> io::Result<Self> {
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        Self::spawn(command, request)
    }

    /// 从已启动的 core 守护进程接管服务器运行时。
    pub fn from_daemon(mut daemon: Daemon, request: &CommandBuildRequest<'_>) -> Self {
        let terminal = Terminal::from_daemon(&mut daemon, request);
        Self { daemon, terminal }
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

    /// 将一行命令写入已验证的服务器控制台。
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
    use std::path::Path;

    use super::*;
    use sealantern_core::process::{CommandBuildMode, CommandBuildRequest};

    fn request(mode: CommandBuildMode) -> CommandBuildRequest<'static> {
        CommandBuildRequest::new(mode, Path::new("server"))
    }

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
    fn direct_server_mode_accepts_console_input() {
        let mut command = command_reader();
        let request = request(CommandBuildMode::DirectJar);
        let mut runtime = ServerRuntime::spawn_with_piped_stdio(&mut command, &request)
            .expect("spawn server runtime");

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

    #[test]
    fn shell_mode_discards_piped_console_input() {
        let mut command = exit_successfully_command();
        let request = request(CommandBuildMode::Shell);
        let mut runtime = ServerRuntime::spawn_with_piped_stdio(&mut command, &request)
            .expect("spawn shell runtime");

        assert!(matches!(
            runtime.send_console_command("stop"),
            Err(TerminalWriteError::InputUnavailable)
        ));
        let _ = runtime.wait().expect("wait for shell process");
    }

    #[test]
    fn preserves_caller_stdio_configuration() {
        let mut command = exit_successfully_command();
        let request = request(CommandBuildMode::DirectJar);
        let mut runtime =
            ServerRuntime::spawn(&mut command, &request).expect("spawn server runtime");

        assert!(runtime.take_output(TerminalStream::Stdout).is_none());
        assert!(matches!(
            runtime.send_console_command("stop"),
            Err(TerminalWriteError::InputUnavailable)
        ));
        assert!(runtime.wait().expect("wait for server process").success());
    }
}
