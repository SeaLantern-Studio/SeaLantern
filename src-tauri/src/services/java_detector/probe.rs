use super::models::JavaInfo;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::process::{Command, Output};

#[cfg(target_os = "windows")]
use crate::utils::constants::CREATE_NO_WINDOW;

static VERSION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:java|openjdk) version "\s*(?P<version>[^"\s]+)\s*""#).unwrap()
});

/// 检查单个 Java 可执行文件
pub(super) fn check_java(path: &str) -> Option<JavaInfo> {
    let output = command_output(path, &["-version"])?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = if stderr.is_empty() { stdout } else { stderr };

    if combined.is_empty() {
        return None;
    }

    let caps = VERSION_RE.captures(&combined)?;
    let version = caps["version"].to_string();
    let major_version = parse_major_version(&version);
    let combined_lower = combined.to_lowercase();
    let is_64bit = combined_lower.contains("64-bit");

    let vendor = if combined_lower.contains("zulu") {
        "Zulu".to_string()
    } else if combined_lower.contains("openjdk") {
        "OpenJDK".to_string()
    } else {
        "Oracle".to_string()
    };

    let resolved = if path == "java" {
        resolve_path_from_env(path)?
    } else {
        let canonical = fs::canonicalize(path).ok()?;
        #[cfg(target_os = "windows")]
        {
            let path_str = canonical.to_string_lossy();
            if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                path_str.into_owned()
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            canonical.to_string_lossy().into_owned()
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

fn command_output(program: &str, args: &[&str]) -> Option<Output> {
    let mut command = Command::new(program);
    command.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.output().ok()
}
