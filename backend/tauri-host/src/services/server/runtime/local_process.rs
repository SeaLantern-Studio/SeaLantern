use crate::models::server::LocalTerminalMode;
use portable_pty::{native_pty_system, CommandBuilder as PtyCommandBuilder, MasterPty, PtySize};
use std::io::{self, Read, Write};
use std::process::Command;

#[derive(Debug)]
pub enum LocalProcessSpawnError {
    PipeSpawn(String),
    PtyInit(String),
    PtySpawn(String),
}

impl LocalProcessSpawnError {
    pub fn is_pty_init_failure(&self) -> bool {
        matches!(self, Self::PtyInit(_))
    }
}

impl std::fmt::Display for LocalProcessSpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PipeSpawn(error) | Self::PtyInit(error) | Self::PtySpawn(error) => {
                f.write_str(error)
            }
        }
    }
}

impl std::error::Error for LocalProcessSpawnError {}

pub struct LocalProcessReaders {
    pub stdout: Option<Box<dyn Read + Send>>,
    pub stderr: Option<Box<dyn Read + Send>>,
}

pub struct LocalProcessLaunch {
    pub process: ManagedLocalProcess,
    pub readers: LocalProcessReaders,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalProcessExitStatus {
    code: Option<i32>,
}

impl std::fmt::Display for LocalProcessExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            #[cfg(windows)]
            Some(code) => write!(f, "exit code: {}", code),
            #[cfg(not(windows))]
            Some(code) => write!(f, "exit status: {}", code),
            None => write!(f, "terminated by signal"),
        }
    }
}

impl LocalProcessExitStatus {
    pub fn code(&self) -> Option<i32> {
        self.code
    }
}

enum ManagedLocalChild {
    Pipe(std::process::Child),
    Pty(Box<dyn portable_pty::Child + Send + Sync>),
}

pub struct ManagedLocalProcess {
    child: ManagedLocalChild,
    stdin: Option<Box<dyn Write + Send>>,
    pty_master: Option<Box<dyn MasterPty + Send>>,
    terminal_mode: LocalTerminalMode,
    terminal_size: Option<(u16, u16)>,
}

impl ManagedLocalProcess {
    pub fn from_pipe_child(mut child: std::process::Child) -> LocalProcessLaunch {
        let stdin = child.stdin.take().map(|value| Box::new(value) as Box<dyn Write + Send>);
        let stdout = child.stdout.take().map(|value| Box::new(value) as Box<dyn Read + Send>);
        let stderr = child.stderr.take().map(|value| Box::new(value) as Box<dyn Read + Send>);

        LocalProcessLaunch {
            process: ManagedLocalProcess {
                child: ManagedLocalChild::Pipe(child),
                stdin,
                pty_master: None,
                terminal_mode: LocalTerminalMode::PipeManaged,
                terminal_size: None,
            },
            readers: LocalProcessReaders { stdout, stderr },
        }
    }

    pub fn id(&self) -> Option<u32> {
        match &self.child {
            ManagedLocalChild::Pipe(child) => Some(child.id()),
            ManagedLocalChild::Pty(child) => child.process_id(),
        }
    }

    pub fn terminal_mode(&self) -> LocalTerminalMode {
        self.terminal_mode
    }

    pub fn terminal_size(&self) -> Option<(u16, u16)> {
        self.terminal_size
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), String> {
        let Some(master) = self.pty_master.as_ref() else {
            return Ok(());
        };

        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())
            .map(|_| {
                self.terminal_size = Some((cols, rows));
            })
    }

    pub fn try_wait(&mut self) -> io::Result<Option<LocalProcessExitStatus>> {
        match &mut self.child {
            ManagedLocalChild::Pipe(child) => child.try_wait().map(|status| {
                status.map(|status| LocalProcessExitStatus {
                    code: status.code(),
                })
            }),
            ManagedLocalChild::Pty(child) => child.try_wait().map(|status| {
                status.map(|status| LocalProcessExitStatus {
                    code: i32::try_from(status.exit_code()).ok(),
                })
            }),
        }
    }

    pub fn wait(&mut self) -> io::Result<LocalProcessExitStatus> {
        match &mut self.child {
            ManagedLocalChild::Pipe(child) => child.wait().map(|status| LocalProcessExitStatus {
                code: status.code(),
            }),
            ManagedLocalChild::Pty(child) => child.wait().map(|status| LocalProcessExitStatus {
                code: i32::try_from(status.exit_code()).ok(),
            }),
        }
    }

    pub fn kill(&mut self) -> io::Result<()> {
        match &mut self.child {
            ManagedLocalChild::Pipe(child) => child.kill(),
            ManagedLocalChild::Pty(child) => child.kill(),
        }
    }

    pub fn write_line(&mut self, command: &str) -> Result<(), String> {
        let stdin = self.stdin.as_mut().ok_or_else(|| "stdin unavailable".to_string())?;
        writeln!(stdin, "{}", command).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())
    }

    pub fn write_raw(&mut self, data: &str) -> Result<(), String> {
        let stdin = self.stdin.as_mut().ok_or_else(|| "stdin unavailable".to_string())?;
        stdin.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())
    }
}

pub fn spawn_local_process(
    cmd: Command,
    terminal_mode: LocalTerminalMode,
) -> Result<LocalProcessLaunch, LocalProcessSpawnError> {
    match terminal_mode {
        LocalTerminalMode::PipeManaged => spawn_pipe_process(cmd),
        LocalTerminalMode::PtyManaged => spawn_pty_process(cmd),
    }
}

fn spawn_pipe_process(mut cmd: Command) -> Result<LocalProcessLaunch, LocalProcessSpawnError> {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd
        .spawn()
        .map_err(|e| LocalProcessSpawnError::PipeSpawn(e.to_string()))?;

    Ok(ManagedLocalProcess::from_pipe_child(child))
}

fn spawn_pty_process(cmd: Command) -> Result<LocalProcessLaunch, LocalProcessSpawnError> {
    let pty_system = native_pty_system();
    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| LocalProcessSpawnError::PtyInit(e.to_string()))?;

    let builder = command_to_pty_builder(&cmd);
    let child = pty_pair
        .slave
        .spawn_command(builder)
        .map_err(|e| LocalProcessSpawnError::PtySpawn(e.to_string()))?;

    let stdout = Some(
        pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| LocalProcessSpawnError::PtyInit(e.to_string()))?,
    );
    let stdin = Some(
        pty_pair
            .master
            .take_writer()
            .map_err(|e| LocalProcessSpawnError::PtyInit(e.to_string()))?,
    );

    Ok(LocalProcessLaunch {
            process: ManagedLocalProcess {
                child: ManagedLocalChild::Pty(child),
                stdin,
                pty_master: Some(pty_pair.master),
                terminal_mode: LocalTerminalMode::PtyManaged,
                terminal_size: Some((80, 24)),
            },
        readers: LocalProcessReaders {
            stdout,
            stderr: None,
        },
    })
}

fn command_to_pty_builder(cmd: &Command) -> PtyCommandBuilder {
    let mut builder = PtyCommandBuilder::new(cmd.get_program());
    builder.args(cmd.get_args());

    if let Some(cwd) = cmd.get_current_dir() {
        builder.cwd(cwd);
    }

    for (key, value) in cmd.get_envs() {
        match value {
            Some(value) => builder.env(key, value),
            None => builder.env_remove(key),
        }
    }

    builder
}
