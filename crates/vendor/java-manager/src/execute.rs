//! Running Java programs with controlled output and redirection.

use crate::{JavaError, JavaInfo};
use is_executable::is_executable;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

/// Controls which output streams are printed to the console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Both,
    OutputOnly,
    ErrorOnly,
}

impl JavaInfo {
    /// Executes the Java executable with the given arguments, printing both
    /// stdout and stderr to the console.
    ///
    /// The argument string is split using shell‑like rules (via `shell_words`).
    /// The child process's stdout and stderr are captured and printed line by line
    /// while the process runs.
    ///
    /// # Errors
    ///
    /// Returns `JavaError::IoError` if spawning or waiting fails.
    /// Returns `JavaError::Other` if the argument string cannot be parsed.
    /// Returns `JavaError::ExecutionFailed` if the Java process exits with a non‑zero status.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use java_manager::JavaInfo;
    /// # let java = JavaInfo::new("/path/to/java".into())?;
    /// java.execute("-version")?;
    /// # Ok::<_, java_manager::JavaError>(())
    /// ```
    pub fn execute(&self, args: &str) -> Result<(), JavaError> {
        self.run_java(args, OutputMode::Both)
    }

    /// Executes the Java executable, printing only stderr to the console.
    /// Stdout is captured and discarded.
    ///
    /// See [`execute`](JavaInfo::execute) for details.
    pub fn execute_with_error(&self, args: &str) -> Result<(), JavaError> {
        self.run_java(args, OutputMode::ErrorOnly)
    }

    /// Executes the Java executable, printing only stdout to the console.
    /// Stderr is captured and discarded.
    ///
    /// See [`execute`](JavaInfo::execute) for details.
    pub fn execute_with_output(&self, args: &str) -> Result<(), JavaError> {
        self.run_java(args, OutputMode::OutputOnly)
    }

    /// Internal implementation of Java execution with configurable output.
    fn run_java(&self, args: &str, mode: OutputMode) -> Result<(), JavaError> {
        let java_exe = self.java_executable()?;

        let arg_vec = shell_words::split(args)
            .map_err(|e| JavaError::Other(format!("Failed to parse arguments: {}", e)))?;

        let mut cmd = Command::new(java_exe);
        cmd.args(&arg_vec);

        let capture_stdout = matches!(mode, OutputMode::Both | OutputMode::OutputOnly);
        let capture_stderr = matches!(mode, OutputMode::Both | OutputMode::ErrorOnly);
        cmd.stdout(if capture_stdout {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stderr(if capture_stderr {
            Stdio::piped()
        } else {
            Stdio::null()
        });

        let mut child = cmd.spawn().map_err(JavaError::IoError)?;

        let stdout_handle = if capture_stdout {
            let stdout = child.stdout.take().ok_or_else(|| {
                JavaError::RuntimeError("Java stdout pipe was not created".to_string())
            })?;
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    println!("{}", line?);
                }
                Ok::<(), io::Error>(())
            }))
        } else {
            None
        };

        let stderr_handle = if capture_stderr {
            let stderr = child.stderr.take().ok_or_else(|| {
                JavaError::RuntimeError("Java stderr pipe was not created".to_string())
            })?;
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    eprintln!("{}", line?);
                }
                Ok::<(), io::Error>(())
            }))
        } else {
            None
        };

        let status = child.wait().map_err(JavaError::IoError);

        if let Some(handle) = stdout_handle {
            join_output_reader(handle, "stdout")?;
        }
        if let Some(handle) = stderr_handle {
            join_output_reader(handle, "stderr")?;
        }

        let status = status?;
        if status.success() {
            Ok(())
        } else {
            Err(JavaError::ExecutionFailed(format!(
                "process exited with status {}",
                format_process_status(&status)
            )))
        }
    }

    /// Returns the path to the `java` executable inside this installation's `JAVA_HOME/bin`.
    ///
    /// # Errors
    ///
    /// Returns `JavaError::NotFound` if the executable does not exist.
    fn java_executable(&self) -> Result<PathBuf, JavaError> {
        let java_home = &self.java_home;
        let exe_name = if cfg!(windows) { "java.exe" } else { "java" };
        let java_exe = java_home.join("bin").join(exe_name);
        if java_exe.is_file() && is_executable(&java_exe) {
            Ok(java_exe)
        } else {
            Err(JavaError::NotFound(format!(
                "Java executable not found or not executable: {}",
                java_exe.display()
            )))
        }
    }
}

fn join_output_reader(
    handle: thread::JoinHandle<io::Result<()>>,
    stream: &str,
) -> Result<(), JavaError> {
    handle
        .join()
        .map_err(|_| {
            JavaError::RuntimeError(format!("Java {stream} output reader thread panicked"))
        })?
        .map_err(JavaError::IoError)
}

fn format_process_status(status: &ExitStatus) -> String {
    status.code().map_or_else(
        || format!("terminated without an exit code ({status})"),
        |code| code.to_string(),
    )
}

/// A builder for configuring and executing a Java program (JAR or main class).
///
/// This struct allows you to set the Java runtime, JAR file or main class,
/// memory limits, program arguments, and I/O redirection before spawning the
/// process.
///
/// # Examples
///
/// ```no_run
/// use java_manager::{JavaRunner, JavaRedirect};
///
/// # let java = java_manager::java_home().unwrap();
/// JavaRunner::new()
///     .java(java)
///     .jar("myapp.jar")
///     .min_memory(256 * 1024 * 1024)   // 256 MB
///     .max_memory(1024 * 1024 * 1024)  // 1 GB
///     .arg("--server")
///     .redirect(JavaRedirect::new().output("out.log").error("err.log"))
///     .execute()?;
/// # Ok::<_, java_manager::JavaError>(())
/// ```
#[derive(Debug, Default)]
pub struct JavaRunner {
    java: Option<JavaInfo>,
    jar: Option<PathBuf>,
    min_memory: Option<String>,
    max_memory: Option<String>,
    main_class: Option<String>,
    args: Vec<String>,
    redirect: JavaRedirect,
    redirect_root: Option<PathBuf>,
}

/// I/O redirection options for a Java process.
///
/// Use the builder methods to specify files for stdout, stderr, and stdin.
/// If a stream is not redirected, it will inherit the parent's corresponding
/// stream (i.e., print to console or read from keyboard).
#[derive(Debug, Default)]
pub struct JavaRedirect {
    output: Option<PathBuf>,
    error: Option<PathBuf>,
    input: Option<PathBuf>,
}

impl JavaRedirect {
    /// Creates a new empty redirection configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Redirects the Java process's standard output to the given file.
    /// The file will be created (or truncated) before execution.
    pub fn output(mut self, path: impl AsRef<Path>) -> Self {
        self.output = Some(path.as_ref().to_path_buf());
        self
    }

    /// Redirects the Java process's standard error to the given file.
    /// The file will be created (or truncated) before execution.
    pub fn error(mut self, path: impl AsRef<Path>) -> Self {
        self.error = Some(path.as_ref().to_path_buf());
        self
    }

    /// Redirects the Java process's standard input from the given file.
    /// The file must exist and be readable.
    pub fn input(mut self, path: impl AsRef<Path>) -> Self {
        self.input = Some(path.as_ref().to_path_buf());
        self
    }
}

impl JavaRunner {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Java installation to use.
    ///
    /// This is mandatory before calling `execute`.
    pub fn java(mut self, java: JavaInfo) -> Self {
        self.java = Some(java);
        self
    }

    /// Sets the JAR file to execute (implies the `-jar` flag).
    ///
    /// Either `jar` or `main_class` must be set.
    pub fn jar(mut self, jar: impl AsRef<Path>) -> Self {
        self.jar = Some(jar.as_ref().to_path_buf());
        self
    }

    /// Sets the initial heap size (`-Xms`).
    ///
    /// The value is given in bytes and will be formatted as a memory string
    /// (e.g., `256m`, `1g`). If the size is not a multiple of a megabyte or gigabyte,
    /// it will be rounded to the nearest megabyte.
    pub fn min_memory(mut self, bytes: usize) -> Self {
        self.min_memory = Some(format_memory(bytes));
        self
    }

    /// Sets the maximum heap size (`-Xmx`).
    ///
    /// See [`min_memory`](JavaRunner::min_memory) for formatting details.
    pub fn max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = Some(format_memory(bytes));
        self
    }

    /// Sets the main class to execute (instead of a JAR file).
    ///
    /// Either `jar` or `main_class` must be set.
    pub fn main_class(mut self, class: impl Into<String>) -> Self {
        self.main_class = Some(class.into());
        self
    }

    /// Adds a single argument to be passed to the Java program.
    ///
    /// Arguments are appended in the order they are added.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Sets I/O redirection options.
    pub fn redirect(mut self, redirect: JavaRedirect) -> Self {
        self.redirect = redirect;
        self
    }

    /// 设置重定向文件的根目录；所有重定向路径必须是该目录下的相对路径。
    /// 未设置时使用当前工作目录，避免调用方无意间写入任意绝对路径。
    pub fn redirect_root(mut self, root: impl AsRef<Path>) -> Self {
        self.redirect_root = Some(root.as_ref().to_path_buf());
        self
    }

    /// Executes the configured Java program.
    ///
    /// # Errors
    ///
    /// Returns `JavaError::Other` if no Java installation has been set, or if
    /// neither a JAR file nor a main class has been specified.
    /// Returns `JavaError::NotFound` if the Java executable does not exist.
    /// Returns `JavaError::IoError` if file operations or process spawning fail.
    /// Returns `JavaError::ExecutionFailed` if the Java process exits with a non‑zero status.
    pub fn execute(self) -> Result<(), JavaError> {
        let redirect_root = self.redirect_root.as_deref();
        let java = self.java.ok_or_else(|| {
            JavaError::Other("Must set Java environment via `.java(...)`".to_string())
        })?;
        let java_exe = java.java_executable()?;

        let mut cmd = Command::new(java_exe);

        if let Some(min) = &self.min_memory {
            cmd.arg(format!("-Xms{}", min));
        }
        if let Some(max) = &self.max_memory {
            cmd.arg(format!("-Xmx{}", max));
        }

        if let Some(jar) = self.jar {
            cmd.arg("-jar");
            cmd.arg(jar);
        } else if let Some(main) = self.main_class {
            cmd.arg(main);
        } else {
            return Err(JavaError::Other("Must specify JAR file or main class".into()));
        }

        cmd.args(&self.args);

        // Configure redirection
        if let Some(output) = self.redirect.output {
            let output = resolve_redirect_path(redirect_root, &output, "stdout")?;
            let file = File::create(&output).map_err(|source| JavaError::RedirectIoError {
                stream: "stdout",
                path: output,
                source,
            })?;
            cmd.stdout(Stdio::from(file));
        } else {
            cmd.stdout(Stdio::inherit());
        }

        if let Some(error) = self.redirect.error {
            let error = resolve_redirect_path(redirect_root, &error, "stderr")?;
            let file = File::create(&error).map_err(|source| JavaError::RedirectIoError {
                stream: "stderr",
                path: error,
                source,
            })?;
            cmd.stderr(Stdio::from(file));
        } else {
            cmd.stderr(Stdio::inherit());
        }

        if let Some(input) = self.redirect.input {
            let input = resolve_redirect_path(redirect_root, &input, "stdin")?;
            let file = File::open(&input).map_err(|source| JavaError::RedirectIoError {
                stream: "stdin",
                path: input,
                source,
            })?;
            cmd.stdin(Stdio::from(file));
        } else {
            cmd.stdin(Stdio::inherit());
        }

        let status = cmd.status().map_err(JavaError::IoError)?;

        if status.success() {
            Ok(())
        } else {
            Err(JavaError::ExecutionFailed(format!(
                "process exited with status {}",
                format_process_status(&status)
            )))
        }
    }
}

fn resolve_redirect_path(
    root: Option<&Path>,
    path: &Path,
    stream: &'static str,
) -> Result<PathBuf, JavaError> {
    let invalid = |reason: &str| JavaError::InvalidRedirectPath {
        stream,
        path: path.to_path_buf(),
        reason: reason.to_string(),
    };

    if path.as_os_str().is_empty() {
        return Err(invalid("path must not be empty"));
    }
    if path.is_absolute() {
        return Err(invalid("use a relative path below the redirect root"));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(invalid("parent-directory components are not allowed"));
    }

    let root = match root {
        Some(root) => root.to_path_buf(),
        None => env::current_dir().map_err(|error| {
            invalid(&format!("failed to determine the default redirect root: {error}"))
        })?,
    };
    let root = fs::canonicalize(&root)
        .map_err(|error| invalid(&format!("redirect root is not accessible: {error}")))?;
    let candidate = root.join(path);

    match fs::symlink_metadata(&candidate) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(invalid("symbolic-link redirect targets are not allowed"));
            }

            let canonical = fs::canonicalize(&candidate)
                .map_err(|error| invalid(&format!("redirect target is not accessible: {error}")))?;
            if !canonical.starts_with(&root) {
                return Err(invalid("redirect target escapes the redirect root"));
            }
            Ok(canonical)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = candidate
                .parent()
                .ok_or_else(|| invalid("redirect target has no parent directory"))?;
            let canonical_parent = fs::canonicalize(parent).map_err(|error| {
                invalid(&format!("redirect parent directory is not accessible: {error}"))
            })?;
            if !canonical_parent.starts_with(&root) {
                return Err(invalid("redirect parent directory escapes the redirect root"));
            }
            let file_name = candidate
                .file_name()
                .ok_or_else(|| invalid("redirect target has no file name"))?;
            Ok(canonical_parent.join(file_name))
        }
        Err(error) => Err(invalid(&format!("redirect target cannot be inspected: {error}"))),
    }
}

/// Formats a memory size in bytes into a Java‑compatible string (`<n>m` or `<n>g`).
///
/// If the size is an exact multiple of 1 GiB, it is formatted as `<n>g`.
/// Otherwise, if it is an exact multiple of 1 MiB, it is formatted as `<n>m`.
/// If neither, it is rounded to the nearest mebibyte and formatted as `<n>m`.
fn format_memory(bytes: usize) -> String {
    const MB: usize = 1024 * 1024;
    const GB: usize = MB * 1024;

    if bytes.is_multiple_of(GB) {
        format!("{}g", bytes / GB)
    } else if bytes.is_multiple_of(MB) {
        format!("{}m", bytes / MB)
    } else {
        let mb = bytes.saturating_add(MB / 2) / MB;
        format!("{}m", mb)
    }
}

#[cfg(test)]
mod tests {
    use super::{format_memory, resolve_redirect_path};
    use std::path::Path;

    #[test]
    fn memory_formatting_does_not_overflow() {
        let value = format_memory(usize::MAX);

        assert!(value.ends_with('m'));
    }

    #[test]
    fn redirect_paths_cannot_escape_the_default_root() {
        assert!(resolve_redirect_path(None, Path::new("../outside.log"), "stdout").is_err());
        assert!(resolve_redirect_path(None, &std::env::temp_dir(), "stdout").is_err());
    }
}
