use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
use std::process::Command;

// 静默执行命令
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// 获取 win 注册表内容
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// 常见 Java 目录别名
#[allow(dead_code)] // 我实在绷不住了, 本地clippy能过但是github action上会报错 fuck the linter
const JAVA_PATH_ALIASES: &[&str] = &[
    "java", "jdk", "jre", "graalvm", "corretto", "temurin", "zulu", "openjdk", "gvl", "ojdk",
];

#[cfg(target_os = "windows")]
const ENV_VARS: &[&str] = &["JAVA_HOME", "JDK_HOME", "GRAALVM_HOME"];

#[cfg(target_os = "windows")]
const PROGRAM_FILES_JAVA_DIRS: &[&str] = &["Java", "Zulu", "Eclipse Adoptium", "BellSoft"];

#[cfg(not(target_os = "windows"))]
const ENV_VARS: &[&str] = &["JAVA_HOME", "JDK_HOME", "GRAALVM_HOME"];

#[cfg(not(target_os = "windows"))]
const COMMON_JAVA_DIRS: &[&str] =
    &["/usr/lib/jvm", "/usr/local/lib/jvm", "/Library/Java/JavaVirtualMachines"];

#[cfg(target_os = "windows")]
const MAX_SCAN_DEPTH: u32 = 5;

#[cfg(not(target_os = "windows"))]
const MAX_SCAN_DEPTH: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    pub vendor: String,
    pub is_64bit: bool,
    pub major_version: u32,
}

pub fn detect_java_installations() -> Vec<JavaInfo> {
    let mut results = Vec::new();
    let candidate_paths = get_candidate_paths();

    #[cfg(target_os = "windows")]
    let candidate_paths = {
        let mut paths = candidate_paths;
        if let Ok(reg_paths) = get_javas_from_registry() {
            paths.extend(reg_paths);
        }
        paths
    };

    for path in candidate_paths {
        if let Some(info) = check_java(&path) {
            if !results.iter().any(|r: &JavaInfo| r.path == info.path) {
                results.push(info);
            }
        }
    }

    results.sort_by(|a, b| b.major_version.cmp(&a.major_version));
    results
}

pub fn validate_java(path: &str) -> Result<JavaInfo, String> {
    check_java(path).ok_or_else(|| format!("无法验证 Java 路径: {}", path))
}

fn get_candidate_paths() -> Vec<String> {
    let mut paths = Vec::with_capacity(32);
    paths.push(String::from("java"));

    for env_var in ENV_VARS {
        if let Ok(val) = std::env::var(env_var) {
            push_java_exe(&val, &mut paths);
        }
    }

    #[cfg(target_os = "windows")]
    {
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

        for root in scan_roots {
            deep_scan_recursive(&root, &mut paths, MAX_SCAN_DEPTH);
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

    #[cfg(not(target_os = "windows"))]
    {
        for dir in COMMON_JAVA_DIRS {
            deep_scan_recursive(Path::new(dir), &mut paths, MAX_SCAN_DEPTH);
        }
    }

    paths
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
                if path.file_name().is_some_and(|n| n == "bin") {
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

fn check_java(path: &str) -> Option<JavaInfo> {
    let output = command_output(path, &["-version"])?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = if stderr.is_empty() { stdout } else { stderr };

    if combined.is_empty() {
        return None;
    }

    let re = Regex::new(r#"(?i)(?:java|openjdk) version "\s*(?P<version>[^"\s]+)\s*""#).ok()?;
    let caps = re.captures(&combined)?;
    let version = caps["version"].to_string();

    let major_version = parse_major_version(&version);
    let is_64bit = combined.contains("64-Bit") || combined.contains("64-bit");

    let vendor = if combined.to_lowercase().contains("zulu") {
        "Zulu".to_string()
    } else if combined.to_lowercase().contains("openjdk") {
        "OpenJDK".to_string()
    } else {
        "Oracle".to_string()
    };

    let resolved = if path == "java" {
        resolve_path_from_env(path)?
    } else {
        let p = fs::canonicalize(path).ok()?;
        #[cfg(target_os = "windows")]
        {
            let path_str = p.to_string_lossy();
            if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                path_str.into_owned()
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            p.to_string_lossy().into_owned()
        }
    };

    Some(JavaInfo {
        path: resolved,
        version,
        vendor,
        is_64bit,
        major_version,
    })
}

fn parse_major_version(version: &str) -> u32 {
    let parts: Vec<&str> = version.split('.').collect();
    let first: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    if first == 1 && parts.len() > 1 {
        parts[1].parse().unwrap_or(first)
    } else {
        first
    }
}

#[cfg(target_os = "windows")]
fn get_javas_from_registry() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut found = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let keys = vec!["SOFTWARE\\JavaSoft", "SOFTWARE\\WOW6432Node\\JavaSoft"];

    for key_path in keys {
        if let Ok(root_key) = hklm.open_subkey(key_path) {
            search_reg_recursive(&root_key, &mut found);
        }
    }
    Ok(found)
}

#[cfg(target_os = "windows")]
fn search_reg_recursive(key: &RegKey, results: &mut Vec<String>) {
    if let Ok(home) = key.get_value::<String, _>("JavaHome") {
        let p = Path::new(&home).join("bin").join("java.exe");
        if p.exists() {
            results.push(p.to_string_lossy().into_owned());
        }
    }
    for name in key.enum_keys().flatten() {
        if let Ok(sub) = key.open_subkey(&name) {
            search_reg_recursive(&sub, results);
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

fn resolve_path_from_env(cmd: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = command_output("where", &[cmd])?;
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .map(|s| s.trim().to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = command_output("which", &[cmd])?;
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .map(|s| s.trim().to_string())
    }
}

fn command_output(program: &str, args: &[&str]) -> Option<std::process::Output> {
    let mut command = Command::new(program);
    command.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.output().ok()
}
