use crate::{JavaError, JavaInfo};
#[cfg(target_os = "windows")]
use std::collections::VecDeque;
use std::env;
#[cfg(target_os = "windows")]
use std::fs;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
const MAX_SEARCH_DEPTH: usize = 6;

#[cfg(any(target_os = "windows", target_os = "linux"))]
const JAVA_KEYWORDS: &[&str] = &["java", "jdk", "jre", "jvm", "openjdk", "graalvm"];

#[cfg(any(target_os = "windows", target_os = "linux"))]
const EXCLUDE_FOLDERS: &[&str] = &[
    "node_modules",
    ".git",
    "__pycache__",
    "vendor",
    "cache",
    "temp",
    "tmp",
    "logs",
    "log",
];

/// 自动发现结果，以及可继续处理的候选和来源错误。
#[derive(Debug, Default)]
pub struct SearchReport {
    pub installations: Vec<JavaInfo>,
    pub errors: Vec<SearchError>,
    source_failed: bool,
}

/// 与发现来源或候选路径关联的错误。
#[derive(Debug)]
pub struct SearchError {
    pub path: Option<PathBuf>,
    pub error: JavaError,
}

impl SearchError {
    fn source(error: JavaError) -> Self {
        Self { path: None, error }
    }

    fn candidate(path: PathBuf, error: JavaError) -> Self {
        Self { path: Some(path), error }
    }
}

impl SearchReport {
    /// 返回发现来源本身是否无法查询。
    pub fn source_failed(&self) -> bool {
        self.source_failed
    }

    fn source_error(error: JavaError) -> Self {
        Self {
            installations: Vec::new(),
            errors: vec![SearchError::source(error)],
            source_failed: true,
        }
    }

    fn into_result(self) -> Result<Vec<JavaInfo>, JavaError> {
        if self.installations.is_empty()
            && let Some(error) = self.errors.into_iter().next()
        {
            return Err(error.error);
        }

        Ok(self.installations)
    }
}

fn collect_candidate(report: &mut SearchReport, path: PathBuf) {
    match JavaInfo::from_discovered_path(path.to_string_lossy().into_owned()) {
        Ok(info) => report.installations.push(info),
        Err(error) => report.errors.push(SearchError::candidate(path, error)),
    }
}

pub fn quick_search() -> Result<Vec<JavaInfo>, JavaError> {
    quick_search_with_diagnostics().into_result()
}

/// 搜索 PATH，但不执行候选二进制，并保留候选错误。
pub fn quick_search_with_diagnostics() -> SearchReport {
    let mut report = SearchReport::default();

    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let java_exe = if cfg!(windows) { "java.exe" } else { "java" };
            let java_path = path.join(java_exe);

            if java_path.is_file() {
                collect_candidate(&mut report, java_path);
            }
        }
    }

    report
}

pub fn deep_search() -> Result<Vec<JavaInfo>, JavaError> {
    deep_search_with_diagnostics().into_result()
}

/// 执行平台相关的深度搜索，但不执行候选二进制。
pub fn deep_search_with_diagnostics() -> SearchReport {
    #[cfg(target_os = "windows")]
    {
        deep_search_everything()
    }

    #[cfg(target_os = "linux")]
    {
        full_search_with_diagnostics()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        SearchReport::default()
    }
}

pub fn full_search() -> Result<Vec<JavaInfo>, JavaError> {
    full_search_with_diagnostics().into_result()
}

/// 执行完整的平台搜索，并保留候选错误。
pub fn full_search_with_diagnostics() -> SearchReport {
    let mut paths: Vec<PathBuf> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        paths.extend(scan_registry());
        paths.extend(scan_default_paths());
        paths.extend(scan_microsoft_store());
        paths.extend(scan_where_command());
    }

    #[cfg(target_os = "linux")]
    {
        paths.extend(scan_linux());
    }

    paths.sort();
    paths.dedup();

    let mut report = SearchReport::default();
    for path in paths {
        collect_candidate(&mut report, path);
    }
    report
}

#[cfg(target_os = "windows")]
fn deep_search_everything() -> SearchReport {
    use everything_sdk::*;

    let mut everything = match global().try_lock() {
        Ok(everything) => everything,
        Err(_) => {
            return SearchReport::source_error(JavaError::RuntimeError(
                "Failed to lock Everything global state".to_string(),
            ));
        }
    };

    match everything.is_db_loaded() {
        Ok(false) => {
            return SearchReport::source_error(JavaError::ExecuteError(
                "Everything database is not fully loaded".to_string(),
            ));
        }
        Err(EverythingError::Ipc) => {
            return SearchReport::source_error(JavaError::ExecuteError(
                "Everything is not running in the background. Please start Everything.exe"
                    .to_string(),
            ));
        }
        _ => {}
    }

    let mut searcher = everything.searcher();
    searcher.set_search("\"java.exe\" !C:\\Windows\\");
    searcher.set_request_flags(
        RequestFlags::EVERYTHING_REQUEST_FILE_NAME
            | RequestFlags::EVERYTHING_REQUEST_PATH
            | RequestFlags::EVERYTHING_REQUEST_SIZE,
    );
    searcher.set_sort(SortType::EVERYTHING_SORT_NAME_ASCENDING);

    if searcher.get_match_case() {
        return SearchReport::source_error(JavaError::RuntimeError(
            "Everything searcher unexpectedly uses case-sensitive matching".to_string(),
        ));
    }

    let query_results = searcher.query();
    let mut report = SearchReport::default();

    for item in query_results.iter() {
        match item.filepath() {
            Ok(path) => collect_candidate(&mut report, path),
            Err(error) => report
                .errors
                .push(SearchError::source(JavaError::RuntimeError(format!(
                    "Failed to read Everything result path: {error}"
                )))),
        }
    }

    report
}

#[cfg(target_os = "windows")]
fn scan_registry() -> Vec<PathBuf> {
    use winreg::RegKey;
    use winreg::enums::*;

    let mut results = Vec::new();

    let javasoft_paths = [
        r"SOFTWARE\JavaSoft\Java Development Kit",
        r"SOFTWARE\JavaSoft\Java Runtime Environment",
        r"SOFTWARE\WOW6432Node\JavaSoft\Java Development Kit",
        r"SOFTWARE\WOW6432Node\JavaSoft\Java Runtime Environment",
    ];

    for reg_path in &javasoft_paths {
        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(reg_path) {
            for subkey_name in key.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(subkey) = key.open_subkey(subkey_name)
                    && let Ok(java_home) = subkey.get_value::<String, _>("JavaHome")
                {
                    let java_exe = Path::new(&java_home).join("bin").join("java.exe");
                    if java_exe.exists() {
                        results.push(java_exe);
                    }
                }
            }
        }
    }

    let brand_paths = [
        (r"SOFTWARE\Azul Systems\Zulu", "InstallationPath"),
        (r"SOFTWARE\BellSoft\Liberica", "InstallationPath"),
    ];

    for (reg_path, value_name) in &brand_paths {
        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(reg_path) {
            for subkey_name in key.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(subkey) = key.open_subkey(subkey_name)
                    && let Ok(install_path) = subkey.get_value::<String, _>(value_name)
                {
                    let java_exe = Path::new(&install_path).join("bin").join("java.exe");
                    if java_exe.exists() {
                        results.push(java_exe);
                    }
                }
            }
        }
    }

    results
}

#[cfg(target_os = "windows")]
fn scan_default_paths() -> Vec<PathBuf> {
    let mut results = Vec::new();
    let roots = default_path_roots();

    for root in roots {
        if !root.exists() {
            continue;
        }

        let mut queue = VecDeque::new();
        queue.push_back((root, 0));

        while let Some((current, depth)) = queue.pop_front() {
            if depth > MAX_SEARCH_DEPTH {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let java_exe = path.join("java.exe");
                    if java_exe.exists() {
                        results.push(java_exe);
                        continue;
                    }

                    if should_explore_deeper(&path) {
                        queue.push_back((path, depth + 1));
                    }
                }
            }
        }
    }

    results
}

#[cfg(target_os = "windows")]
fn default_path_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(appdata) = env::var_os("APPDATA") {
        roots.push(Path::new(&appdata).join(".minecraft").join("runtime"));
    }

    if let Some(localappdata) = env::var_os("LOCALAPPDATA") {
        roots.push(Path::new(&localappdata).to_path_buf());
    }

    if let Some(profile) = env::var_os("USERPROFILE") {
        roots.push(Path::new(&profile).to_path_buf());
    }

    for drive in fixed_drives() {
        let prog_files = Path::new(&drive).join("Program Files");
        if prog_files.exists() {
            roots.push(prog_files);
        }

        let prog_files_x86 = Path::new(&drive).join("Program Files (x86)");
        if prog_files_x86.exists() {
            roots.push(prog_files_x86);
        }

        if let Ok(entries) = fs::read_dir(&drive) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_lowercase();
                if JAVA_KEYWORDS.iter().any(|kw| dir_name.contains(kw)) {
                    roots.push(entry.path());
                }
            }
        }
    }

    roots
}

#[cfg(target_os = "windows")]
fn fixed_drives() -> Vec<String> {
    let mut drives = Vec::new();
    for letter in 'A'..='Z' {
        let drive = format!("{letter}:\\");
        let p = Path::new(&drive);
        if p.exists() && fs::read_dir(p).is_ok() {
            drives.push(drive);
        }
    }
    drives
}

#[cfg(target_os = "windows")]
fn should_explore_deeper(path: &Path) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy(),
        None => return false,
    };

    let lower = name.to_lowercase();

    for ex in EXCLUDE_FOLDERS {
        if lower.contains(ex) {
            return false;
        }
    }

    for kw in JAVA_KEYWORDS {
        if lower.contains(kw) {
            return true;
        }
    }

    is_version_like(&name)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn is_version_like(name: &str) -> bool {
    if name.is_empty() || name.len() > 20 {
        return false;
    }

    let has_digit = name.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '_' || c == '-')
}

#[cfg(target_os = "windows")]
fn scan_microsoft_store() -> Vec<PathBuf> {
    let localappdata = match env::var_os("LOCALAPPDATA") {
        Some(v) => v,
        None => return Vec::new(),
    };

    let base = Path::new(&localappdata)
        .join(r"Packages\Microsoft.4297127D64EC6_8wekyb3d8bbwe\LocalCache\Local\runtime");

    if !base.exists() {
        return Vec::new();
    }

    let mut results = Vec::new();

    if let Ok(runtimes) = fs::read_dir(&base) {
        for runtime in runtimes.flatten() {
            let runtime_path = runtime.path();
            let name = runtime_path.file_name().map(|n| n.to_string_lossy());
            if !name
                .as_deref()
                .is_some_and(|n| n.starts_with("java-runtime"))
            {
                continue;
            }

            if let Ok(archs) = fs::read_dir(&runtime_path) {
                for arch in archs.flatten() {
                    let arch_path = arch.path();
                    if !arch_path.is_dir() {
                        continue;
                    }

                    if let Ok(versions) = fs::read_dir(&arch_path) {
                        for version in versions.flatten() {
                            let java_exe = version.path().join("bin").join("java.exe");
                            if java_exe.exists() {
                                results.push(java_exe);
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

#[cfg(target_os = "windows")]
fn scan_where_command() -> Vec<PathBuf> {
    let mut results = Vec::new();

    if let Ok(output) = std::process::Command::new("where").arg("java").output()
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let p = Path::new(trimmed);
                if p.exists() {
                    results.push(p.to_path_buf());
                }
            }
        }
    }

    results
}

#[cfg(target_os = "linux")]
fn scan_linux() -> Vec<PathBuf> {
    use walkdir::WalkDir;

    let mut results = Vec::new();

    let mut search_dirs: Vec<PathBuf> =
        vec!["/usr/lib/jvm".into(), "/usr/java".into(), "/opt".into(), "/usr/local".into()];

    if let Ok(home) = env::var("HOME") {
        let mc_runtime = Path::new(&home).join(".minecraft").join("runtime");
        if mc_runtime.exists() {
            search_dirs.push(mc_runtime);
        }
    }

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(&dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            if entry_path.file_name() != Some(std::ffi::OsStr::new("java")) {
                continue;
            }
            if !entry_path.is_file() {
                continue;
            }

            if let Some(parent) = entry_path.parent() {
                if !should_explore_linux(parent) {
                    continue;
                }
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = entry_path.metadata() {
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o111 != 0 {
                        results.push(entry_path.to_path_buf());
                    }
                }
            }
            #[cfg(not(unix))]
            {
                results.push(entry_path.to_path_buf());
            }
        }
    }

    results
}

#[cfg(target_os = "linux")]
fn should_explore_linux(path: &Path) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy(),
        None => return true,
    };

    let lower = name.to_lowercase();

    for ex in EXCLUDE_FOLDERS {
        if lower.contains(ex) {
            return false;
        }
    }

    for kw in JAVA_KEYWORDS {
        if lower.contains(kw) {
            return true;
        }
    }

    is_version_like(&name)
}
