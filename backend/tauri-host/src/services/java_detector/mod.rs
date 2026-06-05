//! Java 检测服务

mod models;
mod probe;
#[cfg(target_os = "windows")]
mod registry;
mod scan;

pub use models::JavaInfo;

use probe::check_java;
use scan::get_candidate_paths;

/// 扫描本机可用的 Java 安装
pub fn detect_java_installations() -> Vec<JavaInfo> {
    let mut results = Vec::new();
    let candidate_paths = get_candidate_paths();

    #[cfg(target_os = "windows")]
    let candidate_paths = {
        let mut paths = candidate_paths;
        if let Ok(reg_paths) = registry::get_javas_from_registry() {
            paths.extend(reg_paths);
        }
        paths
    };

    for path in candidate_paths {
        if let Some(info) = check_java(&path) {
            if !results.iter().any(|item: &JavaInfo| item.path == info.path) {
                results.push(info);
            }
        }
    }

    results.sort_by_key(|item| std::cmp::Reverse(item.major_version));
    results
}

/// 校验单个 Java 路径
pub fn validate_java(path: &str) -> Result<JavaInfo, String> {
    check_java(path).ok_or_else(|| format!("无法验证 Java 路径: {}", path))
}
