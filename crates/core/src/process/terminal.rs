use std::fmt;
use std::io::{self, Read, Write};
use std::process::{ChildStderr, ChildStdin, ChildStdout};

use super::command_build::ConsoleInputPolicy;
use super::{CommandBuildRequest, Daemon};

/// 标识守护进程的一个输出流。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalStream {
    Stdout,
    Stderr,
}

/// 一个持有的守护进程输出流，保留其源身份标识。
pub enum TerminalOutput {
    Stdout(ChildStdout),
    Stderr(ChildStderr),
}

impl TerminalOutput {
    pub fn stream(&self) -> TerminalStream {
        match self {
            Self::Stdout(_) => TerminalStream::Stdout,
            Self::Stderr(_) => TerminalStream::Stderr,
        }
    }
}

impl Read for TerminalOutput {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(stdout) => stdout.read(buffer),
            Self::Stderr(stderr) => stderr.read(buffer),
        }
    }
}

/// 守护进程的标准流，在其所有权已移出进程句柄之后。
pub struct Terminal {
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

impl Terminal {
    /// 根据用于启动守护进程的请求，从守护进程转移已配置的流。
    ///
    /// 此方法仅从请求推导输入策略，其余流转移统一委托给 [`Self::from_daemon_with_input`]。
    pub fn from_daemon(daemon: &mut Daemon, request: &CommandBuildRequest<'_>) -> Self {
        Self::from_daemon_with_input(
            daemon,
            matches!(request.console_input_policy(), ConsoleInputPolicy::Enabled),
        )
    }

    /// 从守护进程转移标准流，并由宿主显式声明是否保留控制台输入。
    ///
    /// 适用于已经在 core 外部构建进程命令、但仍希望复用守护进程和终端所有权模型的宿主。
    pub fn from_daemon_with_input(daemon: &mut Daemon, accepts_input: bool) -> Self {
        let stdin = daemon.take_stdin();
        Self {
            stdin: accepts_input.then_some(stdin).flatten(),
            stdout: daemon.take_stdout(),
            stderr: daemon.take_stderr(),
        }
    }

    /// 返回守护进程是否配置了可写的标准输入流。
    pub fn accepts_input(&self) -> bool {
        self.stdin.is_some()
    }

    /// 将字节写入标准输入并刷新到守护进程。
    pub fn write(&mut self, input: &[u8]) -> Result<(), TerminalWriteError> {
        let stdin = self.stdin()?;
        stdin.write_all(input).map_err(TerminalWriteError::Write)?;
        stdin.flush().map_err(TerminalWriteError::Flush)
    }

    /// 将一个命令行写入标准输入并刷新到守护进程。
    pub fn write_line(&mut self, line: &str) -> Result<(), TerminalWriteError> {
        let stdin = self.stdin()?;
        stdin
            .write_all(line.as_bytes())
            .map_err(TerminalWriteError::Write)?;
        stdin.write_all(b"\n").map_err(TerminalWriteError::Write)?;
        stdin.flush().map_err(TerminalWriteError::Flush)
    }

    /// 将输出流的所有权转移到主机读取器。
    pub fn take_output(&mut self, stream: TerminalStream) -> Option<TerminalOutput> {
        match stream {
            TerminalStream::Stdout => self.stdout.take().map(TerminalOutput::Stdout),
            TerminalStream::Stderr => self.stderr.take().map(TerminalOutput::Stderr),
        }
    }

    fn stdin(&mut self) -> Result<&mut ChildStdin, TerminalWriteError> {
        self.stdin
            .as_mut()
            .ok_or(TerminalWriteError::InputUnavailable)
    }
}

/// 描述写入守护进程终端失败的原因。
#[derive(Debug)]
pub enum TerminalWriteError {
    InputUnavailable,
    Write(io::Error),
    Flush(io::Error),
}

impl TerminalWriteError {
    /// 返回底层的操作系统错误（当尝试写入时）。
    pub fn io_error(&self) -> Option<&io::Error> {
        match self {
            Self::InputUnavailable => None,
            Self::Write(error) | Self::Flush(error) => Some(error),
        }
    }
}

impl fmt::Display for TerminalWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputUnavailable => write!(formatter, "daemon standard input is not configured"),
            Self::Write(error) => {
                write!(formatter, "could not write to daemon standard input: {error}")
            }
            Self::Flush(error) => {
                write!(formatter, "could not flush daemon standard input: {error}")
            }
        }
    }
}

impl std::error::Error for TerminalWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.io_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

// ## 仅限于 Windows：该模块的测试涉及子进程 pipe 通信，在 Linux CI 上会因进程管理问题卡住超时。
// ## Windows 上 taskkill 工作正常，故保留。若需在 Linux 上运行，需先修复 terminate_tree 的信号发送逻辑。
#[cfg(all(test, not(unix)))]
mod tests {
    use std::io::Read;
    use std::process::{Command, Stdio};

    use super::{Terminal, TerminalOutput, TerminalStream, TerminalWriteError};
    use crate::process::{CommandBuildMode, CommandBuildRequest, Daemon};

    fn request(mode: CommandBuildMode) -> CommandBuildRequest<'static> {
        CommandBuildRequest::new(mode, std::path::Path::new("server"))
    }

    #[cfg(unix)]
    fn output_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "printf stdout; printf stderr >&2"]);
        command
    }

    #[cfg(windows)]
    fn output_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "echo stdout & echo stderr 1>&2"]);
        command
    }

    #[cfg(unix)]
    fn command_reader_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "read line; printf '%s' \"$line\""]);
        command
    }

    #[cfg(windows)]
    fn command_reader_command() -> Command {
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
    fn transfers_output_streams_without_losing_their_identity() {
        let mut command = output_command();
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let request = request(CommandBuildMode::Shell);
        let mut terminal = Terminal::from_daemon(&mut daemon, &request);

        let mut stdout = terminal
            .take_output(TerminalStream::Stdout)
            .expect("stdout should be available");
        let mut stderr = terminal
            .take_output(TerminalStream::Stderr)
            .expect("stderr should be available");
        let mut stdout_text = String::new();
        let mut stderr_text = String::new();

        stdout
            .read_to_string(&mut stdout_text)
            .expect("read stdout");
        stderr
            .read_to_string(&mut stderr_text)
            .expect("read stderr");

        assert_eq!(stdout.stream(), TerminalStream::Stdout);
        assert_eq!(stderr.stream(), TerminalStream::Stderr);
        assert_eq!(stdout_text.trim(), "stdout");
        assert_eq!(stderr_text.trim(), "stderr");
        assert!(daemon.wait().expect("wait for test process").success());
        assert!(terminal.take_output(TerminalStream::Stdout).is_none());
    }

    #[test]
    fn host_selected_console_input_writes_and_flushes_a_command_line() {
        let mut command = command_reader_command();
        command.stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let mut terminal = Terminal::from_daemon_with_input(&mut daemon, true);

        terminal
            .write_line("say hello")
            .expect("write command line");
        let mut stdout = terminal
            .take_output(TerminalStream::Stdout)
            .expect("stdout should be available");
        let mut output = String::new();
        stdout
            .read_to_string(&mut output)
            .expect("read command output");

        assert!(daemon.wait().expect("wait for test process").success());
        assert!(output.contains("say hello"));
    }

    #[test]
    fn reports_unavailable_standard_input_without_discarding_state() {
        let mut command = exit_successfully_command();
        command.stdin(Stdio::null());
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let request = request(CommandBuildMode::DirectJar);
        let mut terminal = Terminal::from_daemon(&mut daemon, &request);

        let error = terminal
            .write_line("stop")
            .expect_err("stdin was not configured as piped");

        assert!(matches!(error, TerminalWriteError::InputUnavailable));
        assert!(!terminal.accepts_input());
        let _ = daemon.wait().expect("wait for test process");
    }

    #[test]
    fn terminal_output_implements_read_for_host_owned_readers() {
        let mut command = output_command();
        command.stdout(Stdio::piped()).stderr(Stdio::null());
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let request = request(CommandBuildMode::Shell);
        let mut terminal = Terminal::from_daemon(&mut daemon, &request);
        let mut output = terminal
            .take_output(TerminalStream::Stdout)
            .expect("stdout should be available");

        assert!(matches!(output, TerminalOutput::Stdout(_)));

        let mut text = String::new();
        output
            .read_to_string(&mut text)
            .expect("read terminal output");
        assert!(text.contains("stdout"));
        let _ = daemon.wait().expect("wait for test process");
    }

    #[test]
    fn shell_mode_discards_piped_input() {
        let mut command = exit_successfully_command();
        command.stdin(Stdio::piped());
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let request = request(CommandBuildMode::Shell);
        let mut terminal = Terminal::from_daemon(&mut daemon, &request);

        let error = terminal
            .write_line("stop")
            .expect_err("shell mode must not receive console input");

        assert!(matches!(error, TerminalWriteError::InputUnavailable));
        assert!(!terminal.accepts_input());
        let _ = daemon.wait().expect("wait for test process");
    }

    #[test]
    fn legacy_custom_command_discards_piped_input() {
        let mut command = exit_successfully_command();
        command.stdin(Stdio::piped());
        let mut daemon = Daemon::spawn(&mut command).expect("spawn test process");
        let mut request = request(CommandBuildMode::Custom);
        request.custom_command = Some("java -jar server.jar");
        let mut terminal = Terminal::from_daemon(&mut daemon, &request);

        let error = terminal
            .write_line("stop")
            .expect_err("legacy custom shell must not receive console input");

        assert!(matches!(error, TerminalWriteError::InputUnavailable));
        assert!(!terminal.accepts_input());
        let _ = daemon.wait().expect("wait for test process");
    }
}
