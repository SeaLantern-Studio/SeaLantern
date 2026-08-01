//! Core type representing a Java installation and its metadata.

use crate::JavaError;
use is_executable::is_executable;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::str;
use std::thread;
use std::time::{Duration, Instant};

const UNKNOWN: &str = "UNKNOWN";
const VERSION_PROBE_OUTPUT_LIMIT: usize = 64 * 1024;
const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const VERSION_PROBE_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Represents a discovered Java installation.
///
/// This struct holds metadata about a Java runtime, such as its version,
/// vendor, architecture, and the location of its `java` executable and
/// `JAVA_HOME` directory.
#[derive(Debug)]
pub struct JavaInfo {
    /// Human-readable name of the Java implementation (e.g., "OpenJDK").
    pub name: String,
    /// Version string (e.g., "11.0.2").
    pub version: String,
    /// Full path to the `java` executable (or the path originally provided).
    pub path: PathBuf,
    /// Vendor name (e.g., "Oracle", "OpenJDK").
    pub vendor: String,
    /// Architecture (e.g., "64-Bit", "32-Bit").
    pub architecture: String,
    /// The `JAVA_HOME` directory corresponding to this installation.
    pub java_home: PathBuf,
}

impl JavaInfo {
    /// Creates a new `JavaInfo` from a path pointing either to a `java` executable
    /// or directly to a `JAVA_HOME` directory.
    ///
    /// The path is canonicalized, and if it is an executable, the `JAVA_HOME` is
    /// located by walking up the directory tree until a `bin/java` (or `java.exe`)
    /// is found. Metadata is then extracted from the `release` file inside
    /// `JAVA_HOME`, and any missing fields are filled by running `java -version`.
    ///
    /// # Errors
    ///
    /// Returns `JavaError::InvalidJavaPath` if the path does not exist,
    /// or if `JAVA_HOME` cannot be determined from an executable.
    /// Returns other `JavaError` variants if I/O or command execution fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use java_manager::JavaInfo;
    ///
    /// let info = JavaInfo::new("/usr/lib/jvm/java-11-openjdk/bin/java".into())?;
    /// println!("Java version: {}", info.version);
    /// # Ok::<_, java_manager::JavaError>(())
    /// ```
    pub fn new(path: String) -> Result<Self, JavaError> {
        let mut info = Self::from_path(path)?;

        if info.is_complete() {
            return Ok(info);
        }

        // 显式校验允许执行用户指定的 Java，但探测必须有超时和输出上限。
        let version_info = read_version(&info.java_home.join("bin").join(if cfg!(windows) {
            "java.exe"
        } else {
            "java"
        }))?;

        if info.name == UNKNOWN
            && let Some(name) = version_info.name
        {
            info.name = name;
        }
        if info.version == UNKNOWN
            && let Some(ver) = version_info.version
        {
            info.version = ver;
        }
        if info.vendor == UNKNOWN
            && let Some(vend) = version_info.vendor
        {
            info.vendor = vend;
        }
        if info.architecture == UNKNOWN
            && let Some(arch) = version_info.arch
        {
            info.architecture = arch;
        }

        Ok(info)
    }

    /// 从自动发现结果构建信息，但不会执行发现到的可执行文件。
    pub(crate) fn from_discovered_path(path: String) -> Result<Self, JavaError> {
        let info = Self::from_path(path)?;
        if info.is_complete() {
            Ok(info)
        } else {
            Err(JavaError::RuntimeError(format!(
                "Java release metadata is incomplete: {}",
                info.java_home.display()
            )))
        }
    }

    fn from_path(path: String) -> Result<Self, JavaError> {
        let path_obj = Path::new(&path);
        if !path_obj.exists() {
            return Err(JavaError::InvalidJavaPath(format!("Path does not exist: {}", path)));
        }

        // Resolve symlinks to get the real absolute path
        let canonical_path = fs::canonicalize(path_obj).map_err(JavaError::IoError)?;

        let (java_home, exec_path) = if canonical_path.is_file() {
            if !is_executable(&canonical_path) {
                return Err(JavaError::InvalidJavaPath(format!(
                    "Path is not an executable: {}",
                    canonical_path.display()
                )));
            }

            let home = find_java_home_from_exe(&canonical_path).ok_or_else(|| {
                JavaError::InvalidJavaPath(format!(
                    "Unable to determine JAVA_HOME from executable: {}",
                    canonical_path.display()
                ))
            })?;
            (home, Some(canonical_path))
        } else if canonical_path.is_dir() {
            (canonical_path, None)
        } else {
            return Err(JavaError::InvalidJavaPath(format!(
                "Path is neither a file nor a directory: {}",
                canonical_path.display()
            )));
        };

        // Path to the java executable inside JAVA_HOME
        let java_exe = java_home
            .join("bin")
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        if !java_exe.is_file() || !is_executable(&java_exe) {
            return Err(JavaError::NotFound(format!(
                "Java executable not found or not executable: {}",
                java_exe.display()
            )));
        }

        // Store either the original executable path or the default one from bin
        let stored_path = exec_path.unwrap_or_else(|| java_exe.clone());

        let mut info = JavaInfo {
            name: UNKNOWN.to_string(),
            version: UNKNOWN.to_string(),
            path: stored_path,
            vendor: UNKNOWN.to_string(),
            architecture: UNKNOWN.to_string(),
            java_home,
        };

        // --- Step 1: read from release file (if possible) ---
        if let Some(release) = read_release(&info.java_home) {
            if let Some(name) = release.name {
                info.name = name;
            }
            if let Some(version) = release.version {
                info.version = version;
            }
            if let Some(vendor) = release.vendor {
                info.vendor = vendor;
            }
            if let Some(arch) = release.arch {
                info.architecture = arch;
            }
        }

        Ok(info)
    }
}

impl Default for JavaInfo {
    fn default() -> Self {
        Self {
            name: UNKNOWN.to_string(),
            version: UNKNOWN.to_string(),
            path: PathBuf::new(),
            vendor: UNKNOWN.to_string(),
            architecture: UNKNOWN.to_string(),
            java_home: PathBuf::new(),
        }
    }
}

impl JavaInfo {
    fn is_complete(&self) -> bool {
        self.name != UNKNOWN
            && self.version != UNKNOWN
            && self.vendor != UNKNOWN
            && self.architecture != UNKNOWN
    }
}

// -----------------------------------------------------------------------------
// Helper: locate JAVA_HOME from a java executable path
// -----------------------------------------------------------------------------
fn find_java_home_from_exe(exec_path: &Path) -> Option<PathBuf> {
    let mut current = exec_path.parent()?;
    loop {
        let bin_java = current
            .join("bin")
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        if bin_java.exists() && is_executable(&bin_java) {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

// -----------------------------------------------------------------------------
// Data extracted from the release file
// -----------------------------------------------------------------------------
struct ReleaseInfo {
    name: Option<String>,
    version: Option<String>,
    vendor: Option<String>,
    arch: Option<String>,
}

fn read_release(java_home: &Path) -> Option<ReleaseInfo> {
    let release_path = java_home.join("release");
    let file = File::open(release_path).ok()?;
    let reader = BufReader::new(file);
    let mut properties = HashMap::new();

    for line in reader.lines() {
        let line = line.ok()?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            properties.insert(key, value);
        }
    }

    Some(ReleaseInfo {
        name: properties.get("IMPLEMENTOR").cloned(),
        version: properties.get("JAVA_VERSION").cloned(),
        vendor: properties.get("IMPLEMENTOR").cloned(),
        arch: properties.get("OS_ARCH").cloned(),
    })
}

// -----------------------------------------------------------------------------
// Data extracted from `java -version` output
// -----------------------------------------------------------------------------
struct VersionInfo {
    name: Option<String>,
    version: Option<String>,
    vendor: Option<String>,
    arch: Option<String>,
}

struct LimitedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_limited<R: Read>(mut reader: R, limit: usize) -> io::Result<LimitedOutput> {
    let mut bytes = Vec::with_capacity(limit.min(8 * 1024));
    let mut buffer = [0_u8; 8 * 1024];
    let mut truncated = false;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        let remaining = limit.saturating_sub(bytes.len());
        let kept = read.min(remaining);
        bytes.extend_from_slice(&buffer[..kept]);
        truncated |= kept < read;
    }

    Ok(LimitedOutput { bytes, truncated })
}

fn join_limited_reader(
    handle: thread::JoinHandle<io::Result<LimitedOutput>>,
    stream: &str,
) -> Result<LimitedOutput, JavaError> {
    handle
        .join()
        .map_err(|_| {
            JavaError::RuntimeError(format!("Java version {stream} reader thread panicked"))
        })?
        .map_err(|error| {
            JavaError::ExecuteError(format!("Failed to read Java version {stream}: {error}"))
        })
}

fn read_version(java_exe: &Path) -> Result<VersionInfo, JavaError> {
    let mut child = Command::new(java_exe)
        .arg("-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            JavaError::ExecuteError(format!("Failed to execute java -version: {error}"))
        })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        JavaError::RuntimeError("Java version probe stdout pipe was not created".to_string())
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        JavaError::RuntimeError("Java version probe stderr pipe was not created".to_string())
    })?;

    let stdout_handle = thread::spawn(move || read_limited(stdout, VERSION_PROBE_OUTPUT_LIMIT));
    let stderr_handle = thread::spawn(move || read_limited(stderr, VERSION_PROBE_OUTPUT_LIMIT));

    let started_at = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if started_at.elapsed() >= VERSION_PROBE_TIMEOUT => {
                timed_out = true;
                let _ = child.kill();
                break child.wait().map_err(|error| {
                    JavaError::ExecuteError(format!(
                        "Failed to stop timed-out java -version process: {error}"
                    ))
                });
            }
            Ok(None) => thread::sleep(VERSION_PROBE_POLL_INTERVAL),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(JavaError::ExecuteError(format!(
                    "Failed to wait for java -version: {error}"
                )));
            }
        }
    };

    let stdout = join_limited_reader(stdout_handle, "stdout")?;
    let stderr = join_limited_reader(stderr_handle, "stderr")?;
    let status = status?;

    if timed_out {
        return Err(JavaError::ProcessTimeout {
            executable: java_exe.to_path_buf(),
            timeout: VERSION_PROBE_TIMEOUT,
        });
    }

    if stdout.truncated || stderr.truncated {
        return Err(JavaError::OutputLimitExceeded {
            executable: java_exe.to_path_buf(),
            limit: VERSION_PROBE_OUTPUT_LIMIT,
        });
    }

    if !status.success() {
        return Err(JavaError::ExecuteError(format!(
            "java -version command failed with status: {}",
            format_process_status(&status)
        )));
    }

    let stderr = str::from_utf8(&stderr.bytes).map_err(|e| {
        JavaError::RuntimeError(format!("Failed to decode java -version output: {}", e))
    })?;

    let mut version = None;
    let mut vendor = None;
    let mut arch = None;

    for line in stderr.lines() {
        // Extract version from lines like `openjdk version "11.0.2" 2019-01-15`
        if line.contains(" version ")
            && let Some(start) = line.find('"')
            && let Some(end) = line[start + 1..].find('"')
        {
            version = Some(line[start + 1..start + 1 + end].to_string());
        }

        // Extract vendor from "Runtime Environment" line
        if line.contains("Runtime Environment")
            && let Some(idx) = line.find("Runtime Environment")
        {
            let rest = &line[idx + "Runtime Environment".len()..];
            let vendor_part = rest.split_whitespace().next().unwrap_or("");
            let vendor_cleaned = vendor_part
                .split(['-', '('])
                .next()
                .unwrap_or("")
                .to_string();
            if !vendor_cleaned.is_empty() {
                vendor = Some(vendor_cleaned);
            }
        }

        // Extract architecture (64‑Bit / 32‑Bit)
        if line.contains("VM") && line.contains("Bit") {
            if line.contains("64-Bit") {
                arch = Some("64-Bit".to_string());
            } else if line.contains("32-Bit") {
                arch = Some("32-Bit".to_string());
            }
        }
    }

    // Fallback vendor from first line
    if vendor.is_none()
        && let Some(first_line) = stderr.lines().next()
    {
        if first_line.starts_with("openjdk") {
            vendor = Some("OpenJDK".to_string());
        } else if first_line.starts_with("java") {
            vendor = Some("Oracle".to_string());
        }
    }

    // Name is derived from vendor (keeps original behaviour)
    let name = vendor.clone();

    Ok(VersionInfo { name, version, vendor, arch })
}

fn format_process_status(status: &ExitStatus) -> String {
    status.code().map_or_else(
        || format!("terminated without an exit code ({status})"),
        |code| code.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::read_limited;
    use std::io::Cursor;

    #[test]
    fn limited_reader_keeps_prefix_and_drains_remaining_output() {
        let output = read_limited(Cursor::new(b"123456"), 4).expect("reader should succeed");

        assert_eq!(output.bytes, b"1234");
        assert!(output.truncated);
    }
}
