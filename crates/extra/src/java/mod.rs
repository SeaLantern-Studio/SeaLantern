//! Java 环境检测适配层。
//!
//! 这个模块隔离 `java-manager` 的第三方类型和扫描策略，向 `extra` 其余
//! 部分只暴露项目自己的 Java 信息模型。单个扫描来源失败时保留其它来源
//! 的结果，并通过 tracing 记录失败来源，避免本机没有 Everything 等可选
//! 工具时整个检测流程失效。

use std::{collections::HashSet, path::Path};

use java_manager::{JavaError, JavaInfo as VendorJavaInfo};

pub use crate::config::JavaInfo;
use crate::observability;

/// 扫描本机可用的 Java 安装。
pub fn detect_java_installations() -> Vec<JavaInfo> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    if let Some(info) = java_manager::java_home() {
        push_unique(&mut results, &mut seen, info);
    }

    collect_search_results(&mut results, &mut seen, "quick_search", java_manager::quick_search());

    let deep_search_succeeded =
        collect_search_results(&mut results, &mut seen, "deep_search", java_manager::deep_search());

    // Windows 的 deep_search 依赖 Everything。不可用时退回不依赖外部
    // 常驻程序的完整扫描，Linux/macOS 则继续沿用 vendor 的平台实现。
    if !deep_search_succeeded {
        collect_search_results(&mut results, &mut seen, "full_search", java_manager::full_search());
    }

    results.sort_by_key(|info| std::cmp::Reverse(info.major_version));
    results
}

/// 校验并读取指定路径下的 Java 安装信息。
pub fn validate_java(path: &str) -> Result<JavaInfo, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("Java 路径不能为空".to_string());
    }

    VendorJavaInfo::new(path.to_string())
        .map(to_app_java_info)
        .map_err(|error| format!("无法验证 Java 路径 '{}': {}", path, error))
}

fn collect_search_results(
    results: &mut Vec<JavaInfo>,
    seen: &mut HashSet<String>,
    source: &str,
    search: Result<Vec<VendorJavaInfo>, JavaError>,
) -> bool {
    match search {
        Ok(infos) => {
            for info in infos {
                push_unique(results, seen, info);
            }
            true
        }
        Err(error) => {
            observability::java_search_failed(source, &error);
            false
        }
    }
}

fn push_unique(results: &mut Vec<JavaInfo>, seen: &mut HashSet<String>, info: VendorJavaInfo) {
    let app_info = to_app_java_info(info);
    let key = normalize_path_key(&app_info.path);
    if seen.insert(key) {
        results.push(app_info);
    }
}

fn to_app_java_info(info: VendorJavaInfo) -> JavaInfo {
    let version = normalize_unknown(info.version);
    let vendor = normalize_unknown(info.vendor);
    let architecture = info.architecture.to_ascii_lowercase();
    let major_version = parse_major_version(&version);

    JavaInfo {
        path: normalize_path(&info.path),
        version,
        vendor,
        is_64bit: architecture.contains("64")
            || matches!(architecture.as_str(), "amd64" | "x86_64" | "aarch64"),
        major_version,
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

fn normalize_path_key(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        path.to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string()
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
    fn vendor_info_maps_to_project_info() {
        let info = VendorJavaInfo {
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

    #[test]
    fn unknown_metadata_is_exposed_as_empty_values() {
        assert!(normalize_unknown("UNKNOWN".to_string()).is_empty());
        assert_eq!(normalize_unknown("OpenJDK".to_string()), "OpenJDK");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_path_strips_windows_extended_prefix() {
        let path = PathBuf::from(r"\\?\C:\Java\jdk-21\bin\java.exe");

        assert_eq!(normalize_path(&path), r"C:\Java\jdk-21\bin\java.exe");
    }
}
