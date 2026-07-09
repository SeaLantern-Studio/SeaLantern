//! Java 检测服务

use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Java 安装信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    pub vendor: String,
    pub is_64bit: bool,
    pub major_version: u32,
}

/// 扫描本机可用的 Java 安装
pub fn detect_java_installations() -> Vec<JavaInfo> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    if let Some(info) = java_manager::java_home() {
        push_unique(&mut results, &mut seen, info);
    }

    if let Ok(infos) = java_manager::quick_search() {
        for info in infos {
            push_unique(&mut results, &mut seen, info);
        }
    }

    if let Ok(infos) = java_manager::deep_search() {
        for info in infos {
            push_unique(&mut results, &mut seen, info);
        }
    }

    results.sort_by_key(|item| std::cmp::Reverse(item.major_version));
    results
}

/// 校验单个 Java 路径
pub fn validate_java(path: &str) -> Result<JavaInfo, String> {
    java_manager::JavaInfo::new(path.to_string())
        .map(to_app_java_info)
        .map_err(|e| format!("无法验证 Java 路径: {} ({})", path, e))
}

fn push_unique(
    results: &mut Vec<JavaInfo>,
    seen: &mut HashSet<String>,
    info: java_manager::JavaInfo,
) {
    let app_info = to_app_java_info(info);
    let key = app_info.path.to_lowercase();
    if seen.insert(key) {
        results.push(app_info);
    }
}

fn to_app_java_info(info: java_manager::JavaInfo) -> JavaInfo {
    let version = normalize_unknown(info.version);
    let vendor = normalize_unknown(info.vendor);
    let architecture = info.architecture.to_lowercase();

    JavaInfo {
        path: normalize_path(&info.path),
        major_version: parse_major_version(&version),
        version,
        vendor,
        is_64bit: architecture.contains("64")
            || matches!(architecture.as_str(), "amd64" | "x86_64" | "aarch64"),
    }
}

fn normalize_path(path: &Path) -> String {
    let path = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        path.strip_prefix(r"\\?\").unwrap_or(&path).to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.into_owned()
    }
}

fn normalize_unknown(value: String) -> String {
    if value == "UNKNOWN" {
        String::new()
    } else {
        value
    }
}

fn parse_major_version(version: &str) -> u32 {
    let Some(first) = version.split(['.', '-']).next() else {
        return 0;
    };

    let parsed = first.parse().unwrap_or(0);
    if parsed == 1 {
        version
            .split(['.', '-'])
            .nth(1)
            .and_then(|part| part.parse().ok())
            .unwrap_or(parsed)
    } else {
        parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_major_version_handles_legacy_and_modern_versions() {
        assert_eq!(parse_major_version("1.8.0_402"), 8);
        assert_eq!(parse_major_version("17.0.12"), 17);
        assert_eq!(parse_major_version("21.0.4-LTS"), 21);
        assert_eq!(parse_major_version(""), 0);
    }

    #[test]
    fn java_manager_info_maps_to_app_java_info() {
        let info = java_manager::JavaInfo {
            name: "OpenJDK".to_string(),
            version: "21.0.4".to_string(),
            path: PathBuf::from(r"C:\Java\jdk-21\bin\java.exe"),
            vendor: "Eclipse Adoptium".to_string(),
            architecture: "amd64".to_string(),
            java_home: PathBuf::from(r"C:\Java\jdk-21"),
        };

        let app_info = to_app_java_info(info);

        assert_eq!(app_info.version, "21.0.4");
        assert_eq!(app_info.vendor, "Eclipse Adoptium");
        assert_eq!(app_info.major_version, 21);
        assert!(app_info.is_64bit);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_path_strips_windows_extended_prefix() {
        let path = PathBuf::from(r"\\?\C:\Java\jdk-21\bin\java.exe");

        assert_eq!(normalize_path(&path), r"C:\Java\jdk-21\bin\java.exe");
    }
}
