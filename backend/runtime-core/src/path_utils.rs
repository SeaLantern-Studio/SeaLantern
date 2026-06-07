use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::ffi::OsString;
use std::path::PathBuf;

const APP_DOCKER_DATA_DIR: &str = "./data";
const APP_HIDDEN_DIRECTORY_NAME: &str = ".SeaLantern";

#[cfg(any(target_os = "windows", target_os = "macos"))]
const APP_DIRECTORY_NAME: &str = "Sea Lantern";

#[cfg(target_os = "linux")]
const APP_DIRECTORY_NAME_LOWERCASE: &str = "sea-lantern";

pub fn is_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    if bytes.len() >= 3
        && bytes[1] == b':'
        && bytes[0].is_ascii_alphabetic()
        && matches!(bytes[2], b'/' | b'\\')
    {
        return true;
    }

    path.starts_with("\\\\") || path.starts_with("//")
}

fn explicit_app_data_dir_from_env() -> Option<PathBuf> {
    let value = std::env::var("SEALANTERN_DATA_DIR").ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(PathBuf::from(trimmed))
}

#[cfg(target_os = "windows")]
fn is_msi_installation() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let exe_str = parent.to_string_lossy().to_lowercase();
            if exe_str.contains(r"\program files\") {
                return true;
            }
        }
    }
    false
}

pub fn get_app_data_dir() -> PathBuf {
    if let Some(explicit) = explicit_app_data_dir_from_env() {
        crate::log_trace_ctx(
            "runtime.path",
            "get_app_data_dir",
            &format!(
                "[utils.path] action=resolve_app_data_dir source=env path={}",
                explicit.display()
            ),
        );
        return explicit;
    }

    if std::path::Path::new("/.dockerenv").exists() {
        let path = PathBuf::from(APP_DOCKER_DATA_DIR);
        crate::log_trace_ctx(
            "runtime.path",
            "get_app_data_dir",
            &format!(
                "[utils.path] action=resolve_app_data_dir source=docker path={}",
                path.display()
            ),
        );
        return path;
    }

    #[cfg(target_os = "windows")]
    {
        if is_msi_installation() {
            if let Some(data_dir) = dirs_next::data_dir() {
                return data_dir.join(APP_DIRECTORY_NAME);
            }
            if let Some(home_dir) = dirs_next::home_dir() {
                return home_dir.join(APP_HIDDEN_DIRECTORY_NAME);
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                crate::log_trace_ctx(
                    "runtime.path",
                    "get_app_data_dir",
                    &format!(
                        "[utils.path] action=resolve_app_data_dir source=portable_exe path={}",
                        exe_dir.display()
                    ),
                );
                return exe_dir.to_path_buf();
            }
        }

        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir.join(APP_HIDDEN_DIRECTORY_NAME);
        }
        PathBuf::from(".")
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(data_dir) = dirs_next::data_dir() {
            return data_dir.join(APP_DIRECTORY_NAME);
        }
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir
                .join("Library")
                .join("Application Support")
                .join(APP_DIRECTORY_NAME);
        }
        PathBuf::from(".")
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(data_dir) = dirs_next::data_dir() {
            return data_dir.join(APP_DIRECTORY_NAME_LOWERCASE);
        }
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir.join(APP_HIDDEN_DIRECTORY_NAME);
        }
        PathBuf::from(".")
    }
}

pub fn get_or_create_app_data_dir() -> String {
    let data_dir = get_app_data_dir();

    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("警告：无法创建数据目录：{}", e);
    }

    data_dir.to_string_lossy().to_string()
}

pub fn get_or_create_app_data_dir_checked() -> Result<String, String> {
    let data_dir = get_app_data_dir();

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("无法创建数据目录 '{}': {}", data_dir.display(), e))?;

    Ok(data_dir.to_string_lossy().to_string())
}

pub fn find_executable_in_path(executable: &str) -> Option<PathBuf> {
    if executable.trim().is_empty() {
        return None;
    }

    let path_var = std::env::var_os("PATH")?;

    #[cfg(target_os = "windows")]
    {
        find_executable_in_path_with(
            path_var.as_os_str(),
            std::env::var_os("PATHEXT").as_deref(),
            executable,
        )
    }

    #[cfg(not(target_os = "windows"))]
    {
        find_executable_in_path_with(path_var.as_os_str(), executable)
    }
}

#[cfg(not(target_os = "windows"))]
fn find_executable_in_path_with(path_var: &OsStr, executable: &str) -> Option<PathBuf> {
    std::env::split_paths(path_var)
        .map(|dir| dir.join(executable))
        .find(|candidate| is_executable_file(candidate))
}

#[cfg(not(target_os = "windows"))]
fn is_executable_file(candidate: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = std::fs::metadata(candidate) else {
        return false;
    };

    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(target_os = "windows")]
fn find_executable_in_path_with(
    path_var: &OsStr,
    pathext: Option<&OsStr>,
    executable: &str,
) -> Option<PathBuf> {
    let candidate_names = windows_path_candidate_names(executable, pathext);

    std::env::split_paths(path_var).find_map(|dir| {
        candidate_names
            .iter()
            .map(|name| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}

#[cfg(target_os = "windows")]
fn windows_path_candidate_names(executable: &str, pathext: Option<&OsStr>) -> Vec<OsString> {
    let exact_name = OsString::from(executable);
    if std::path::Path::new(executable).extension().is_some() {
        return vec![exact_name];
    }

    let mut names = windows_path_extensions(pathext)
        .into_iter()
        .map(|extension| {
            let mut candidate = exact_name.clone();
            candidate.push(extension);
            candidate
        })
        .collect::<Vec<_>>();
    names.push(exact_name);
    names
}

#[cfg(target_os = "windows")]
fn windows_path_extensions(pathext: Option<&OsStr>) -> Vec<String> {
    let mut extensions = pathext
        .and_then(|value| value.to_str())
        .map(|value| {
            value
                .split(';')
                .filter_map(|segment| {
                    let trimmed = segment.trim();
                    if trimmed.is_empty() {
                        return None;
                    }

                    let normalized = if trimmed.starts_with('.') {
                        trimmed.to_string()
                    } else {
                        format!(".{trimmed}")
                    };
                    Some(normalized)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if extensions.is_empty() {
        extensions =
            vec![".COM".to_string(), ".EXE".to_string(), ".BAT".to_string(), ".CMD".to_string()];
    }

    let mut unique = Vec::new();
    for extension in extensions {
        if unique
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&extension))
        {
            continue;
        }
        unique.push(extension);
    }

    unique
}

pub(crate) fn normalize_path_for_compare(path: &std::path::Path) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    let normalized = normalized
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();

    #[cfg(target_os = "windows")]
    {
        normalized.to_ascii_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        normalized
    }
}

pub fn strip_path_prefix_for_compare(
    path: &std::path::Path,
    prefix: &std::path::Path,
) -> Option<String> {
    let path_norm = normalize_path_for_compare(path);
    let prefix_norm = normalize_path_for_compare(prefix);

    if path_norm == prefix_norm {
        return Some(String::new());
    }

    let remainder = path_norm.strip_prefix(&(prefix_norm + "/"))?;
    Some(remainder.to_string())
}

pub(crate) fn startup_file_extension_priority(extension: &str) -> Option<u8> {
    match extension.to_ascii_lowercase().as_str() {
        "bat" => Some(0),
        "cmd" => Some(0),
        "sh" => Some(1),
        "ps1" => Some(2),
        "jar" => Some(3),
        _ => None,
    }
}

pub fn find_root_startup_file(dir: &std::path::Path) -> Option<PathBuf> {
    find_root_startup_file_checked(dir).ok().flatten()
}

pub fn find_root_startup_file_checked(dir: &std::path::Path) -> Result<Option<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取启动目录失败: {}", e))?;
    let mut candidates = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取启动目录项失败: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        let Some(priority) = startup_file_extension_priority(extension) else {
            continue;
        };
        candidates.push((priority, path));
    }

    candidates.sort_by(|left, right| {
        left.0.cmp(&right.0).then_with(|| {
            let left_name = left
                .1
                .file_name()
                .map(|name| name.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            let right_name = right
                .1
                .file_name()
                .map(|name| name.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            left_name.cmp(&right_name)
        })
    });

    Ok(candidates.into_iter().map(|(_, path)| path).next())
}

pub fn validate_file_name_only(file_name: &str) -> Result<&str, String> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err("文件名不能为空".to_string());
    }

    let path = std::path::Path::new(trimmed);
    if path.is_absolute() {
        return Err("文件名不能是绝对路径".to_string());
    }

    if trimmed == "." || trimmed == ".." {
        return Err("文件名不合法".to_string());
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("文件名里不能包含路径分隔符".to_string());
    }

    let base_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "文件名不合法".to_string())?;

    if base_name != trimmed {
        return Err("文件名不合法".to_string());
    }

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "sea_lantern_runtime_path_utils_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).expect("temp dir should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn find_executable_in_path_resolves_extensionless_java_to_java_exe() {
        let dir = TempDirGuard::new("java_lookup");
        let java_path = dir.path().join("java.exe");
        fs::write(&java_path, b"java").expect("java.exe should be created");

        let path_var = std::env::join_paths([dir.path()]).expect("PATH should be built");
        let resolved = find_executable_in_path_with(
            path_var.as_os_str(),
            Some(OsStr::new(".COM;.EXE;.BAT;.CMD")),
            "java",
        )
        .expect("java should resolve from PATH");

        assert_eq!(resolved.parent(), java_path.parent());
        assert!(resolved
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("java.exe")));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn find_executable_in_path_keeps_exact_match_when_extension_is_provided() {
        let dir = TempDirGuard::new("docker_lookup");
        let docker_cmd = dir.path().join("docker.cmd");
        let docker_exe = dir.path().join("docker.exe");
        fs::write(&docker_cmd, b"docker-cmd").expect("docker.cmd should be created");
        fs::write(&docker_exe, b"docker-exe").expect("docker.exe should be created");

        let path_var = std::env::join_paths([dir.path()]).expect("PATH should be built");
        let resolved = find_executable_in_path_with(
            path_var.as_os_str(),
            Some(OsStr::new(".EXE;.CMD")),
            "docker.cmd",
        );

        assert_eq!(resolved, Some(docker_cmd));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn find_executable_in_path_skips_non_executable_files_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDirGuard::new("non_exec_lookup");
        let java_path = dir.path().join("java");
        fs::write(&java_path, b"#!/bin/sh\nexit 0\n").expect("java file should be created");
        let mut perms = fs::metadata(&java_path)
            .expect("metadata should exist")
            .permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&java_path, perms).expect("permissions should be updated");

        let path_var = std::env::join_paths([dir.path()]).expect("PATH should be built");
        let resolved = find_executable_in_path_with(path_var.as_os_str(), "java");

        assert_eq!(resolved, None);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn find_executable_in_path_accepts_executable_files_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDirGuard::new("exec_lookup");
        let java_path = dir.path().join("java");
        fs::write(&java_path, b"#!/bin/sh\nexit 0\n").expect("java file should be created");
        let mut perms = fs::metadata(&java_path)
            .expect("metadata should exist")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&java_path, perms).expect("permissions should be updated");

        let path_var = std::env::join_paths([dir.path()]).expect("PATH should be built");
        let resolved = find_executable_in_path_with(path_var.as_os_str(), "java");

        assert_eq!(resolved, Some(java_path));
    }

    #[test]
    fn find_root_startup_file_recognizes_cmd_script_as_startup_candidate() {
        let dir = TempDirGuard::new("cmd_startup");
        let cmd_path = dir.path().join("start.cmd");
        let jar_path = dir.path().join("server.jar");
        fs::write(&cmd_path, b"@echo off\n").expect("start.cmd should be created");
        fs::write(&jar_path, b"jar").expect("server.jar should be created");

        let resolved = find_root_startup_file(dir.path());

        assert_eq!(resolved, Some(cmd_path));
    }

    #[test]
    fn find_root_startup_file_checked_surfaces_directory_read_failures() {
        let dir = TempDirGuard::new("missing_startup_dir");
        let missing = dir.path().join("missing");

        let error = find_root_startup_file_checked(&missing)
            .expect_err("checked startup scan should surface directory read failures");

        assert!(error.contains("读取启动目录失败"), "unexpected error: {}", error);
    }

    #[test]
    fn find_root_startup_file_legacy_wrapper_downgrades_directory_read_failures() {
        let dir = TempDirGuard::new("missing_startup_dir_legacy");
        let missing = dir.path().join("missing");

        let resolved = find_root_startup_file(&missing);

        assert_eq!(resolved, None);
    }

    #[test]
    fn get_or_create_app_data_dir_checked_creates_explicit_env_dir() {
        let dir = TempDirGuard::new("app_data_create_checked");
        let target = dir.path().join("nested").join("app-data");
        let _lock = crate::test_support::lock_env();
        let _guard =
            crate::test_support::EnvGuard::set("SEALANTERN_DATA_DIR", &target.to_string_lossy());

        let resolved = get_or_create_app_data_dir_checked()
            .expect("checked app data dir creation should succeed for writable target");

        assert_eq!(PathBuf::from(&resolved), target);
        assert!(target.exists());
        assert!(target.is_dir());
    }

    #[test]
    fn get_or_create_app_data_dir_checked_surfaces_directory_creation_failures() {
        let dir = TempDirGuard::new("app_data_create_checked_fail");
        let file_path = dir.path().join("data-root-file");
        fs::write(&file_path, b"not a directory").expect("file-backed app data root should exist");
        let blocked = file_path.join("nested");
        let _lock = crate::test_support::lock_env();
        let _guard =
            crate::test_support::EnvGuard::set("SEALANTERN_DATA_DIR", &blocked.to_string_lossy());

        let error = get_or_create_app_data_dir_checked()
            .expect_err("checked app data dir creation should surface file-backed path failures");

        assert!(error.contains("无法创建数据目录"), "unexpected error: {}", error);
        assert!(error.contains("data-root-file"), "unexpected error: {}", error);
    }
}
