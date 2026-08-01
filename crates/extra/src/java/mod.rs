//! Java 环境检测适配层。
//!
//! 这个模块隔离 `java-manager` 的第三方类型和扫描策略，向 `extra` 其余
//! 部分只暴露项目自己的 Java 信息模型。单个扫描来源失败时保留其它来源
//! 的结果，并通过 tracing 记录失败来源，避免本机没有 Everything 等可选
//! 工具时整个检测流程失效。

use std::{collections::HashSet, fmt, path::Path};

use java_manager::{JavaInfo as VendorJavaInfo, SearchError, SearchReport};

pub use crate::config::JavaInfo;
use crate::observability;

/// Java 自动检测中单个来源或候选产生的非致命错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaDiscoveryError {
    pub source: String,
    pub message: String,
}

impl fmt::Display for JavaDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} discovery failed: {}", self.source, self.message)
    }
}

impl std::error::Error for JavaDiscoveryError {}

/// Java 自动检测结果；成功安装和非致命错误同时保留。
#[derive(Debug, Default)]
pub struct JavaDetectionReport {
    pub installations: Vec<JavaInfo>,
    pub errors: Vec<JavaDiscoveryError>,
}

/// 显式 Java 路径校验错误。
#[derive(Debug)]
pub enum JavaValidationError {
    EmptyPath,
    InvalidInstallation { path: String, message: String },
}

impl fmt::Display for JavaValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("Java 路径不能为空"),
            Self::InvalidInstallation { path, message } => {
                write!(formatter, "无法验证 Java 路径 '{}': {message}", path)
            }
        }
    }
}

impl std::error::Error for JavaValidationError {}

/// 扫描本机可用的 Java 安装。
pub fn detect_java_installations() -> Vec<JavaInfo> {
    detect_java_installations_with_diagnostics().installations
}

/// 扫描本机可用的 Java 安装，并保留可操作的来源与候选错误。
pub fn detect_java_installations_with_diagnostics() -> JavaDetectionReport {
    observability::java_detection_started();

    let mut report = JavaDetectionReport::default();
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    match java_manager::java_home_with_diagnostics() {
        Ok(Some(info)) => {
            push_unique(&mut results, &mut seen, info);
            observability::java_search_completed("java_home", 1, 0);
        }
        Ok(None) => observability::java_search_completed("java_home", 0, 0),
        Err(error) => {
            observability::java_search_failed("java_home", &error);
            report.errors.push(JavaDiscoveryError {
                source: "java_home".to_string(),
                message: error.to_string(),
            });
            observability::java_search_completed("java_home", 0, 1);
        }
    }

    collect_search_results(
        &mut report,
        &mut results,
        &mut seen,
        "quick_search",
        java_manager::quick_search_with_diagnostics(),
    );

    let deep_search_succeeded = collect_search_results(
        &mut report,
        &mut results,
        &mut seen,
        "deep_search",
        java_manager::deep_search_with_diagnostics(),
    );

    // Windows 的 deep_search 依赖 Everything。不可用时退回不依赖外部
    // 常驻程序的完整扫描，Linux/macOS 则继续沿用 vendor 的平台实现。
    if !deep_search_succeeded {
        observability::java_search_fallback("deep_search", "full_search");
        collect_search_results(
            &mut report,
            &mut results,
            &mut seen,
            "full_search",
            java_manager::full_search_with_diagnostics(),
        );
    }

    results.sort_by_key(|info| std::cmp::Reverse(info.major_version));
    report.installations = results;
    observability::java_detection_completed(report.installations.len(), report.errors.len());
    report
}

/// 校验并读取指定路径下的 Java 安装信息。
pub fn validate_java(path: &str) -> Result<JavaInfo, JavaValidationError> {
    let path = path.trim();
    observability::java_validation_started(path);
    if path.is_empty() {
        let error = JavaValidationError::EmptyPath;
        observability::java_validation_failed(path, &error);
        return Err(error);
    }

    match VendorJavaInfo::new(path.to_string()) {
        Ok(info) => {
            let info = to_app_java_info(info);
            observability::java_validation_completed(path, info.major_version);
            Ok(info)
        }
        Err(error) => {
            let error = JavaValidationError::InvalidInstallation {
                path: path.to_string(),
                message: error.to_string(),
            };
            observability::java_validation_failed(path, &error);
            Err(error)
        }
    }
}

fn collect_search_results(
    report: &mut JavaDetectionReport,
    results: &mut Vec<JavaInfo>,
    seen: &mut HashSet<String>,
    source: &str,
    search: SearchReport,
) -> bool {
    let source_failed = search.source_failed();
    let SearchReport { installations, errors, .. } = search;
    let installation_count = installations.len();
    let error_count = errors.len();

    for info in installations {
        push_unique(results, seen, info);
    }

    for error in errors {
        record_search_error(report, source, error);
    }

    observability::java_search_completed(source, installation_count, error_count);
    !source_failed
}

fn record_search_error(report: &mut JavaDetectionReport, source: &str, error: SearchError) {
    if let Some(path) = error.path.as_deref() {
        observability::java_candidate_rejected(source, path, &error.error);
    } else {
        observability::java_search_failed(source, &error.error);
    }

    report.errors.push(JavaDiscoveryError {
        source: source.to_string(),
        message: error.error.to_string(),
    });
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

    #[test]
    fn validation_error_keeps_path_context() {
        let error = JavaValidationError::InvalidInstallation {
            path: "C:\\Java\\missing".to_string(),
            message: "not found".to_string(),
        };

        assert!(error.to_string().contains("C:\\Java\\missing"));
        assert!(error.to_string().contains("not found"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_path_strips_windows_extended_prefix() {
        let path = PathBuf::from(r"\\?\C:\Java\jdk-21\bin\java.exe");

        assert_eq!(normalize_path(&path), r"C:\Java\jdk-21\bin\java.exe");
    }
}
