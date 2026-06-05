use std::fs;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::path::PathBuf;

use crate::utils::constants::{ENV_VARS, MAX_SCAN_DEPTH};

#[cfg(target_os = "windows")]
use crate::utils::constants::{JAVA_PATH_ALIASES, PROGRAM_FILES_JAVA_DIRS, USER_PROFILE_JAVA_DIRS};

#[cfg(not(target_os = "windows"))]
use crate::utils::constants::COMMON_JAVA_DIRS;

/// 收集全部可能的 Java 路径
pub(super) fn get_candidate_paths() -> Vec<String> {
    let mut paths = Vec::with_capacity(32);
    paths.push(String::from("java"));

    for env_var in ENV_VARS {
        if let Ok(value) = std::env::var(env_var) {
            push_java_exe(&value, &mut paths);
        }
    }

    #[cfg(target_os = "windows")]
    {
        collect_windows_candidate_paths(&mut paths);
    }

    #[cfg(not(target_os = "windows"))]
    {
        for dir in COMMON_JAVA_DIRS {
            deep_scan_recursive(Path::new(dir), &mut paths, MAX_SCAN_DEPTH);
        }
    }

    paths
}

#[cfg(target_os = "windows")]
fn collect_windows_candidate_paths(paths: &mut Vec<String>) {
    let mut scan_roots = Vec::with_capacity(128);

    for drive_letter in b'C'..=b'Z' {
        let drive = format!("{}:\\", drive_letter as char);
        if !Path::new(&drive).exists() {
            continue;
        }

        let drive_path = PathBuf::from(&drive);
        let program_files = drive_path.join("Program Files");

        for java_dir in PROGRAM_FILES_JAVA_DIRS {
            scan_roots.push(program_files.join(java_dir));
        }

        for java_dir in JAVA_PATH_ALIASES {
            let java_path = drive_path.join(java_dir);
            if java_path.exists() {
                scan_roots.push(java_path);
            }
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        let minecraft_root = PathBuf::from(&appdata).join(".minecraft");
        scan_roots.push(minecraft_root.join("runtime"));
        scan_roots.push(minecraft_root.join("cache").join("java"));
    }

    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        scan_roots.push(
            PathBuf::from(&local_appdata)
                .join("Programs")
                .join("Adoptium"),
        );
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let user_profile_path = PathBuf::from(&user_profile);
        for java_dir in USER_PROFILE_JAVA_DIRS {
            scan_roots.push(user_profile_path.join(java_dir));
        }
    }

    for root in scan_roots {
        deep_scan_recursive(&root, paths, MAX_SCAN_DEPTH);
    }

    if let Some(output) = command_output("where", &["java"]) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.push(trimmed.to_string());
            }
        }
    }
}

fn deep_scan_recursive(dir: &Path, paths: &mut Vec<String>, depth: u32) {
    if depth == 0 || !dir.is_dir() {
        return;
    }

    let target_name = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "bin") {
                    let java_exe = path.join(target_name);
                    if java_exe.exists() {
                        paths.push(java_exe.to_string_lossy().into_owned());
                    }
                }
                deep_scan_recursive(&path, paths, depth - 1);
            }
        }
    }
}

fn push_java_exe(dir: &str, paths: &mut Vec<String>) {
    let bin = Path::new(dir)
        .join("bin")
        .join(if cfg!(target_os = "windows") {
            "java.exe"
        } else {
            "java"
        });
    if bin.exists() {
        paths.push(bin.to_string_lossy().into_owned());
    }
}

#[cfg(target_os = "windows")]
fn command_output(program: &str, args: &[&str]) -> Option<std::process::Output> {
    use crate::utils::constants::CREATE_NO_WINDOW;
    use std::process::Command;

    let mut command = Command::new(program);
    command.args(args);

    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);

    command.output().ok()
}
