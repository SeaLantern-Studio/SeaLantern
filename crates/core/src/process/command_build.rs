use std::env;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// 构建服务器进程命令的受支持方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBuildMode {
    DirectJar,
    Custom,
    Batch,
    Shell,
    PowerShell,
}

impl CommandBuildMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectJar => "direct_jar",
            Self::Custom => "custom",
            Self::Batch => "batch",
            Self::Shell => "shell",
            Self::PowerShell => "powershell",
        }
    }
}

/// 控制终端是否可以保留子进程标准输入句柄。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConsoleInputPolicy {
    Enabled,
    Disabled,
}

/// 调用批处理脚本时使用的 Windows 控制台编码。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsConsoleEncoding {
    Utf8,
    Gbk,
}

#[cfg(target_os = "windows")]
impl WindowsConsoleEncoding {
    fn code_page(self) -> &'static str {
        match self {
            Self::Utf8 => "65001",
            Self::Gbk => "936",
        }
    }
}

/// 注入到脚本和自定义可执行环境中的 Java 目录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaEnvironment {
    pub home: PathBuf,
    pub bin: PathBuf,
}

impl JavaEnvironment {
    pub fn new(home: impl Into<PathBuf>, bin: impl Into<PathBuf>) -> Self {
        Self { home: home.into(), bin: bin.into() }
    }

    /// 从 Java 可执行文件路径推导出 Java home 和 bin 目录。
    pub fn from_java_executable(java_executable: &Path) -> Result<Self, CommandBuildError> {
        let bin = java_executable
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or(CommandBuildError::InvalidJavaExecutablePath {
                path: java_executable.to_path_buf(),
            })?;
        let home = bin
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(bin);

        Ok(Self::new(home, bin))
    }
}

/// 用于构造进程命令的输入，不包含主机特定状态。
///
/// `Custom` 模式接受传统的 shell 后端 `custom_command` 文本，或直接的
/// `custom_executable` 加 `custom_arguments`。两种形式互斥，参数仅对直接可执行文件有效。
#[derive(Debug)]
pub struct CommandBuildRequest<'a> {
    pub mode: CommandBuildMode,
    pub working_directory: &'a Path,
    pub java_executable: Option<&'a Path>,
    pub java_environment: Option<&'a JavaEnvironment>,
    pub jvm_arguments: &'a [OsString],
    pub entry_path: Option<&'a Path>,
    pub custom_command: Option<&'a str>,
    pub custom_executable: Option<&'a Path>,
    pub custom_arguments: &'a [OsString],
    pub installer_url: Option<&'a str>,
    pub windows_console_encoding: WindowsConsoleEncoding,
}

impl<'a> CommandBuildRequest<'a> {
    pub fn new(mode: CommandBuildMode, working_directory: &'a Path) -> Self {
        Self {
            mode,
            working_directory,
            java_executable: None,
            java_environment: None,
            jvm_arguments: &[],
            entry_path: None,
            custom_command: None,
            custom_executable: None,
            custom_arguments: &[],
            installer_url: None,
            windows_console_encoding: WindowsConsoleEncoding::Utf8,
        }
    }

    /// 返回由具体进程构造请求所隐含的输入策略。
    pub(crate) fn console_input_policy(&self) -> ConsoleInputPolicy {
        if matches!(self.mode, CommandBuildMode::DirectJar)
            || matches!(custom_launch(self), Ok(CustomLaunch::Executable(_)))
        {
            ConsoleInputPolicy::Enabled
        } else {
            ConsoleInputPolicy::Disabled
        }
    }
}

/// 标识无法构造进程命令的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBuildError {
    MissingJavaExecutable,
    MissingEntryPath { mode: CommandBuildMode },
    MissingCustomLaunch,
    ConflictingCustomLaunch,
    InvalidJavaExecutablePath { path: PathBuf },
    NonUnicodePath { field: &'static str, path: PathBuf },
    UnsupportedPlatform { mode: CommandBuildMode },
}

impl fmt::Display for CommandBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingJavaExecutable => write!(formatter, "a Java executable is required"),
            Self::MissingEntryPath { mode } => {
                write!(formatter, "a startup entry path is required for {} mode", mode.as_str())
            }
            Self::MissingCustomLaunch => {
                write!(formatter, "a custom command or executable is required")
            }
            Self::ConflictingCustomLaunch => {
                write!(formatter, "custom command text and executable arguments cannot be combined")
            }
            Self::InvalidJavaExecutablePath { path } => write!(
                formatter,
                "could not derive a Java environment from executable path {}",
                path.display()
            ),
            Self::NonUnicodePath { field, path } => write!(
                formatter,
                "{} must be valid Unicode before it can be passed to cmd.exe: {}",
                field,
                path.display()
            ),
            Self::UnsupportedPlatform { mode } => {
                write!(formatter, "{} mode is not supported on this platform", mode.as_str())
            }
        }
    }
}

impl std::error::Error for CommandBuildError {}

/// 为请求的服务器启动模式构建命令。
pub fn build_command(request: &CommandBuildRequest<'_>) -> Result<Command, CommandBuildError> {
    match request.mode {
        CommandBuildMode::DirectJar => build_direct_jar_command(request),
        CommandBuildMode::Custom => build_custom_command(request),
        CommandBuildMode::Batch => build_batch_command(request),
        CommandBuildMode::Shell => build_shell_command(request),
        CommandBuildMode::PowerShell => build_powershell_command(request),
    }
}

/// 将 `JAVA_HOME` 和以 Java bin 目录为前缀的 `PATH` 应用到命令。
pub fn apply_java_environment(command: &mut Command, java_environment: &JavaEnvironment) {
    command.env("JAVA_HOME", &java_environment.home);
    command.env("PATH", java_path_value(&java_environment.bin));
}

fn build_direct_jar_command(
    request: &CommandBuildRequest<'_>,
) -> Result<Command, CommandBuildError> {
    let java_executable = request
        .java_executable
        .ok_or(CommandBuildError::MissingJavaExecutable)?;
    let jar_path = required_entry_path(request)?;

    let mut command = Command::new(java_executable);
    command.args(request.jvm_arguments);
    command.arg("-jar");
    command.arg(direct_jar_launch_target(request.working_directory, jar_path));
    command.arg("nogui");
    if let Some(installer_url) = request.installer_url {
        command.arg("--installer");
        command.arg(installer_url);
    }
    command.current_dir(request.working_directory);
    Ok(command)
}

fn build_custom_command(request: &CommandBuildRequest<'_>) -> Result<Command, CommandBuildError> {
    let mut command = match custom_launch(request)? {
        CustomLaunch::Shell(command_text) => shell_command(command_text),
        CustomLaunch::Executable(executable) => {
            let mut command = Command::new(executable);
            command.args(request.custom_arguments);
            command
        }
    };

    apply_optional_java_environment(&mut command, request.java_environment);
    command.current_dir(request.working_directory);
    Ok(command)
}

enum CustomLaunch<'a> {
    Shell(&'a str),
    Executable(&'a Path),
}

fn custom_launch<'a>(
    request: &'a CommandBuildRequest<'a>,
) -> Result<CustomLaunch<'a>, CommandBuildError> {
    let custom_command = request
        .custom_command
        .map(str::trim)
        .filter(|command| !command.is_empty());
    let custom_executable = request
        .custom_executable
        .filter(|path| !path.as_os_str().is_empty());

    match (custom_command, custom_executable, request.custom_arguments.is_empty()) {
        (Some(command), None, true) => Ok(CustomLaunch::Shell(command)),
        (None, Some(executable), _) => Ok(CustomLaunch::Executable(executable)),
        (None, None, _) => Err(CommandBuildError::MissingCustomLaunch),
        _ => Err(CommandBuildError::ConflictingCustomLaunch),
    }
}

#[cfg(target_os = "windows")]
fn shell_command(command_text: &str) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/d", "/c", command_text]);
    command
}

#[cfg(not(target_os = "windows"))]
fn shell_command(command_text: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", command_text]);
    command
}

#[cfg(target_os = "windows")]
fn build_batch_command(request: &CommandBuildRequest<'_>) -> Result<Command, CommandBuildError> {
    let script_path = required_entry_path(request)?;
    let command_text = build_windows_batch_command_text(
        script_path,
        request.windows_console_encoding,
        request.java_environment,
    )?;

    let mut command = Command::new("cmd");
    command.args(["/d", "/c"]);
    command.raw_arg(command_text);
    command.current_dir(request.working_directory);
    Ok(command)
}

#[cfg(not(target_os = "windows"))]
fn build_batch_command(_request: &CommandBuildRequest<'_>) -> Result<Command, CommandBuildError> {
    Err(CommandBuildError::UnsupportedPlatform { mode: CommandBuildMode::Batch })
}

fn build_shell_command(request: &CommandBuildRequest<'_>) -> Result<Command, CommandBuildError> {
    let script_path = required_entry_path(request)?;

    let mut command = Command::new("sh");
    command.arg(script_path);
    command.arg("nogui");
    apply_optional_java_environment(&mut command, request.java_environment);
    command.current_dir(request.working_directory);
    Ok(command)
}

#[cfg(target_os = "windows")]
fn build_powershell_command(
    request: &CommandBuildRequest<'_>,
) -> Result<Command, CommandBuildError> {
    let script_path = required_entry_path(request)?;

    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]);
    command.arg(script_path);
    command.arg("nogui");
    apply_optional_java_environment(&mut command, request.java_environment);
    command.current_dir(request.working_directory);
    Ok(command)
}

#[cfg(not(target_os = "windows"))]
fn build_powershell_command(
    _request: &CommandBuildRequest<'_>,
) -> Result<Command, CommandBuildError> {
    Err(CommandBuildError::UnsupportedPlatform { mode: CommandBuildMode::PowerShell })
}

fn required_entry_path<'a>(
    request: &CommandBuildRequest<'a>,
) -> Result<&'a Path, CommandBuildError> {
    request
        .entry_path
        .ok_or(CommandBuildError::MissingEntryPath { mode: request.mode })
}

fn direct_jar_launch_target(working_directory: &Path, jar_path: &Path) -> PathBuf {
    if jar_path.parent() == Some(working_directory) {
        return jar_path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| jar_path.to_path_buf());
    }

    jar_path.to_path_buf()
}

fn apply_optional_java_environment(
    command: &mut Command,
    java_environment: Option<&JavaEnvironment>,
) {
    if let Some(java_environment) = java_environment {
        apply_java_environment(command, java_environment);
    }
}

fn java_path_value(java_bin: &Path) -> OsString {
    let mut value = java_bin.as_os_str().to_os_string();
    if let Some(existing_path) = env::var_os("PATH").filter(|path| !path.is_empty()) {
        value.push(path_separator());
        value.push(existing_path);
    }
    value
}

#[cfg(target_os = "windows")]
fn build_windows_batch_command_text(
    script_path: &Path,
    console_encoding: WindowsConsoleEncoding,
    java_environment: Option<&JavaEnvironment>,
) -> Result<String, CommandBuildError> {
    let script =
        escape_windows_cmd_argument(windows_command_path(script_path, "batch script path")?);
    let command_prefix = match java_environment {
        Some(java_environment) => format!(
            " & set \"JAVA_HOME={}\" & set \"PATH={};%PATH%\"",
            escape_windows_cmd_argument(windows_command_path(
                &java_environment.home,
                "JAVA_HOME path",
            )?),
            escape_windows_cmd_argument(windows_command_path(
                &java_environment.bin,
                "Java bin path"
            )?),
        ),
        None => String::new(),
    };

    Ok(format!(
        "chcp {}>nul{} & call \"{}\" nogui",
        console_encoding.code_page(),
        command_prefix,
        script,
    ))
}

#[cfg(target_os = "windows")]
fn windows_command_path<'a>(
    path: &'a Path,
    field: &'static str,
) -> Result<&'a str, CommandBuildError> {
    path.to_str()
        .ok_or_else(|| CommandBuildError::NonUnicodePath { field, path: path.to_path_buf() })
}

#[cfg(target_os = "windows")]
fn escape_windows_cmd_argument(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 8);
    for character in value.chars() {
        match character {
            '^' => escaped.push_str("^^"),
            '&' => escaped.push_str("^&"),
            '|' => escaped.push_str("^|"),
            '<' => escaped.push_str("^<"),
            '>' => escaped.push_str("^>"),
            '(' => escaped.push_str("^("),
            ')' => escaped.push_str("^)"),
            '%' => escaped.push_str("%%"),
            '"' => escaped.push_str("\"\""),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(target_os = "windows")]
fn path_separator() -> &'static str {
    ";"
}

#[cfg(not(target_os = "windows"))]
fn path_separator() -> &'static str {
    ":"
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;
    use std::process::Command;

    #[cfg(target_os = "windows")]
    use std::os::windows::ffi::OsStringExt;
    #[cfg(target_os = "windows")]
    use std::path::PathBuf;

    use super::{
        apply_java_environment, build_command, CommandBuildError, CommandBuildMode,
        CommandBuildRequest, ConsoleInputPolicy, JavaEnvironment,
    };

    fn arguments(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    fn environment(command: &Command) -> Vec<(String, Option<String>)> {
        command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect()
    }

    #[test]
    fn direct_jar_uses_a_filename_only_for_a_root_jar() {
        let working_directory = Path::new("servers/paper");
        let jar_path = working_directory.join("server.jar");
        let jvm_arguments = vec![OsString::from("-Xmx4G")];
        let mut request = CommandBuildRequest::new(CommandBuildMode::DirectJar, working_directory);
        request.java_executable = Some(Path::new("java"));
        request.jvm_arguments = &jvm_arguments;
        request.entry_path = Some(&jar_path);
        request.installer_url = Some("https://example.invalid/installer.jar");

        let command = build_command(&request).expect("direct JAR command should build");

        assert_eq!(command.get_program().to_string_lossy(), "java");
        assert_eq!(command.get_current_dir(), Some(working_directory));
        assert_eq!(
            arguments(&command),
            vec![
                "-Xmx4G",
                "-jar",
                "server.jar",
                "nogui",
                "--installer",
                "https://example.invalid/installer.jar",
            ]
        );
    }

    #[test]
    fn direct_jar_keeps_paths_outside_or_below_the_working_directory() {
        let working_directory = Path::new("servers/paper");
        let nested_jar = working_directory.join("libraries/server.jar");
        let external_jar = Path::new("shared/server.jar");

        for jar_path in [&nested_jar, external_jar] {
            let mut request =
                CommandBuildRequest::new(CommandBuildMode::DirectJar, working_directory);
            request.java_executable = Some(Path::new("java"));
            request.entry_path = Some(jar_path);

            let command = build_command(&request).expect("direct JAR command should build");
            let jar_argument = arguments(&command)[1].replace('\\', "/");

            assert_eq!(jar_argument, jar_path.to_string_lossy().replace('\\', "/"));
        }
    }

    #[test]
    fn custom_command_executes_a_program_with_literal_arguments() {
        let java_environment = JavaEnvironment::new("C:/Java/JDK 21", "C:/Java/JDK 21/bin");
        let mut request = CommandBuildRequest::new(CommandBuildMode::Custom, Path::new("server"));
        let custom_executable = Path::new("C:/Servers/launcher.exe");
        let custom_arguments = vec![OsString::from("--name"), OsString::from("my server")];
        request.custom_executable = Some(custom_executable);
        request.custom_arguments = &custom_arguments;
        request.java_environment = Some(&java_environment);

        let command = build_command(&request).expect("custom command should build");

        assert_eq!(command.get_program(), custom_executable);
        assert_eq!(arguments(&command), vec!["--name", "my server"]);

        let environment = environment(&command);
        assert!(environment.iter().any(|(key, value)| {
            key == "JAVA_HOME" && value.as_deref() == Some("C:/Java/JDK 21")
        }));
        assert!(environment.iter().any(|(key, value)| {
            key == "PATH"
                && value
                    .as_deref()
                    .is_some_and(|value| value.starts_with("C:/Java/JDK 21/bin"))
        }));
    }

    #[test]
    fn legacy_custom_command_still_uses_the_platform_shell() {
        let mut request = CommandBuildRequest::new(CommandBuildMode::Custom, Path::new("server"));
        request.custom_command = Some("java -jar server.jar");

        let command = build_command(&request).expect("legacy custom command should build");

        #[cfg(target_os = "windows")]
        {
            assert_eq!(command.get_program().to_string_lossy(), "cmd");
            assert_eq!(arguments(&command), vec!["/d", "/c", "java -jar server.jar"]);
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(command.get_program().to_string_lossy(), "sh");
            assert_eq!(arguments(&command), vec!["-c", "java -jar server.jar"]);
        }

        assert_eq!(request.console_input_policy(), ConsoleInputPolicy::Disabled);
    }

    #[test]
    fn console_input_policy_allows_only_direct_program_requests() {
        let direct_jar = CommandBuildRequest::new(CommandBuildMode::DirectJar, Path::new("server"));
        assert_eq!(direct_jar.console_input_policy(), ConsoleInputPolicy::Enabled);

        let custom_executable = Path::new("launcher.exe");
        let mut direct_custom =
            CommandBuildRequest::new(CommandBuildMode::Custom, Path::new("server"));
        direct_custom.custom_executable = Some(custom_executable);
        assert_eq!(direct_custom.console_input_policy(), ConsoleInputPolicy::Enabled);

        let mut shell_custom =
            CommandBuildRequest::new(CommandBuildMode::Custom, Path::new("server"));
        shell_custom.custom_command = Some("java -jar server.jar");
        assert_eq!(shell_custom.console_input_policy(), ConsoleInputPolicy::Disabled);

        for mode in [CommandBuildMode::Batch, CommandBuildMode::Shell, CommandBuildMode::PowerShell]
        {
            let request = CommandBuildRequest::new(mode, Path::new("server"));
            assert_eq!(request.console_input_policy(), ConsoleInputPolicy::Disabled);
        }
    }

    #[test]
    fn shell_command_passes_nogui_and_preserves_the_java_environment() {
        let java_environment = JavaEnvironment::new("C:/Java/JDK 21", "C:/Java/JDK 21/bin");
        let mut request = CommandBuildRequest::new(CommandBuildMode::Shell, Path::new("server"));
        request.entry_path = Some(Path::new("start.sh"));
        request.java_environment = Some(&java_environment);

        let command = build_command(&request).expect("shell command should build");

        assert_eq!(command.get_program().to_string_lossy(), "sh");
        assert_eq!(arguments(&command), vec!["start.sh", "nogui"]);
        assert!(environment(&command).iter().any(|(key, value)| {
            key == "JAVA_HOME" && value.as_deref() == Some("C:/Java/JDK 21")
        }));
    }

    #[test]
    fn java_environment_requires_an_executable_parent_directory() {
        let error = JavaEnvironment::from_java_executable(Path::new("java"))
            .expect_err("a bare Java executable name has no bin directory");

        assert!(matches!(error, CommandBuildError::InvalidJavaExecutablePath { .. }));
    }

    #[test]
    fn java_environment_injection_keeps_java_bin_first() {
        let java_environment = JavaEnvironment::new("C:/Java", "C:/Java/bin");
        let mut command = Command::new("java");

        apply_java_environment(&mut command, &java_environment);

        assert!(environment(&command).iter().any(|(key, value)| {
            key == "PATH"
                && value
                    .as_deref()
                    .is_some_and(|value| value.starts_with("C:/Java/bin"))
        }));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn windows_only_modes_fail_gracefully_on_other_platforms() {
        for mode in [CommandBuildMode::Batch, CommandBuildMode::PowerShell] {
            let mut request = CommandBuildRequest::new(mode, Path::new("server"));
            request.entry_path = Some(Path::new("start.bat"));

            let error = build_command(&request).expect_err("mode should not be available");

            assert_eq!(error, CommandBuildError::UnsupportedPlatform { mode });
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn batch_command_escapes_script_text_and_inlines_java_environment() {
        use super::WindowsConsoleEncoding;

        let java_environment = JavaEnvironment::new("C:/Java/JDK 21", "C:/Java/JDK 21/bin");
        let mut request = CommandBuildRequest::new(CommandBuildMode::Batch, Path::new("server"));
        request.entry_path = Some(Path::new("start &(1)%2.bat"));
        request.java_environment = Some(&java_environment);
        request.windows_console_encoding = WindowsConsoleEncoding::Gbk;

        let command = build_command(&request).expect("batch command should build");

        assert_eq!(command.get_program().to_string_lossy(), "cmd");
        assert_eq!(
            arguments(&command),
            vec![
                "/d",
                "/c",
                "chcp 936>nul & set \"JAVA_HOME=C:/Java/JDK 21\" & set \"PATH=C:/Java/JDK 21/bin;%PATH%\" & call \"start ^&^(1^)%%2.bat\" nogui",
            ]
        );
        assert!(environment(&command).is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn batch_command_rejects_a_non_unicode_script_path() {
        let non_unicode_path = PathBuf::from(OsString::from_wide(&[0xD800]));
        let mut request = CommandBuildRequest::new(CommandBuildMode::Batch, Path::new("server"));
        request.entry_path = Some(&non_unicode_path);

        let error = build_command(&request).expect_err("non-Unicode paths cannot form cmd text");

        assert!(matches!(
            error,
            CommandBuildError::NonUnicodePath { field: "batch script path", .. }
        ));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn powershell_command_uses_process_environment_injection() {
        let java_environment = JavaEnvironment::new("C:/Java/JDK 21", "C:/Java/JDK 21/bin");
        let mut request =
            CommandBuildRequest::new(CommandBuildMode::PowerShell, Path::new("server"));
        request.entry_path = Some(Path::new("start.ps1"));
        request.java_environment = Some(&java_environment);

        let command = build_command(&request).expect("PowerShell command should build");

        assert_eq!(command.get_program().to_string_lossy(), "powershell");
        assert_eq!(
            arguments(&command),
            vec![
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "start.ps1",
                "nogui",
            ]
        );
        assert!(environment(&command).iter().any(|(key, value)| {
            key == "JAVA_HOME" && value.as_deref() == Some("C:/Java/JDK 21")
        }));
    }
}
