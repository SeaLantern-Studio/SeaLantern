use std::collections::HashSet;

use java_manager::{SearchError, SearchReport};

use super::error::JavaDiscoveryError;
use super::index::{
    from_vendor_search_index, to_vendor_search_index, JavaSearchIndex, JAVA_SEARCH_INDEX_VERSION,
};
use super::mapping::push_unique;
use crate::models::JavaInfo;
use crate::observability;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JavaSearchSource {
    JavaHome,
    QuickSearch,
    DeepSearch,
    FullSearch,
    GlobalSearch,
}

impl JavaSearchSource {
    const fn as_str(self) -> &'static str {
        match self {
            Self::JavaHome => "java_home",
            Self::QuickSearch => "quick_search",
            Self::DeepSearch => "deep_search",
            Self::FullSearch => "full_search",
            Self::GlobalSearch => "global_search",
        }
    }
}

/// Java 自动检测结果；成功安装和非致命错误同时保留。
#[derive(Debug, Default)]
pub struct JavaDetectionReport {
    pub installations: Vec<JavaInfo>,
    pub errors: Vec<JavaDiscoveryError>,
}

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
            observability::java_search_completed(JavaSearchSource::JavaHome.as_str(), 1, 0);
        }
        Ok(None) => observability::java_search_completed(JavaSearchSource::JavaHome.as_str(), 0, 0),
        Err(error) => {
            observability::java_search_failed(JavaSearchSource::JavaHome.as_str(), &error);
            report.errors.push(JavaDiscoveryError {
                source: JavaSearchSource::JavaHome.as_str().to_string(),
                message: error.to_string(),
            });
            observability::java_search_completed(JavaSearchSource::JavaHome.as_str(), 0, 1);
        }
    }

    collect_search_results(
        &mut report,
        &mut results,
        &mut seen,
        JavaSearchSource::QuickSearch,
        java_manager::quick_search_with_diagnostics(),
    );

    let deep_search_succeeded = collect_search_results(
        &mut report,
        &mut results,
        &mut seen,
        JavaSearchSource::DeepSearch,
        java_manager::deep_search_with_diagnostics(),
    );

    // Windows 的 deep_search 依赖 Everything。不可用时退回不依赖外部
    // 常驻程序的完整扫描，Linux/macOS 则继续沿用 vendor 的平台实现。
    if !deep_search_succeeded {
        observability::java_search_fallback(
            JavaSearchSource::DeepSearch.as_str(),
            JavaSearchSource::FullSearch.as_str(),
        );
        collect_search_results(
            &mut report,
            &mut results,
            &mut seen,
            JavaSearchSource::FullSearch,
            java_manager::full_search_with_diagnostics(),
        );
    }

    results.sort_by_key(|info| std::cmp::Reverse(info.major_version));
    report.installations = results;
    observability::java_detection_completed(report.installations.len(), report.errors.len());
    report
}

/// 执行全局 Java 搜索，并返回可交给上游持久化的最新索引。
///
/// `complete` 为 `false` 时使用交互式快速策略；后台任务可传入 `true` 执行
/// 不限制深度的 BFS。传入上次返回的索引后，只会重新枚举目录元数据发生变化
/// 的目录，同时重新验证索引中的候选路径。
pub fn detect_java_installations_with_global_search(
    previous: Option<&JavaSearchIndex>,
    complete: bool,
) -> (JavaDetectionReport, JavaSearchIndex) {
    let previous_vendor = match previous {
        None => None,
        Some(index) if index.schema_version == JAVA_SEARCH_INDEX_VERSION => {
            Some(to_vendor_search_index(index))
        }
        Some(index) => {
            observability::java_global_search_index_ignored(
                index.schema_version,
                JAVA_SEARCH_INDEX_VERSION,
            );
            None
        }
    };
    observability::java_global_search_started(previous_vendor.is_some(), complete);

    let options = if complete {
        java_manager::GlobalSearchOptions::complete()
    } else {
        java_manager::GlobalSearchOptions::fast()
    };
    let search = java_manager::global_search_with_index(previous_vendor.as_ref(), options);
    let vendor_index = search.index.clone().unwrap_or_default();
    let mut report = JavaDetectionReport::default();
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    collect_search_results(
        &mut report,
        &mut results,
        &mut seen,
        JavaSearchSource::GlobalSearch,
        search,
    );
    results.sort_by_key(|info| std::cmp::Reverse(info.major_version));
    report.installations = results;

    let index = from_vendor_search_index(vendor_index);
    observability::java_global_search_completed(
        report.installations.len(),
        report.errors.len(),
        index.directories.len(),
        index.candidates.len(),
    );
    (report, index)
}

fn collect_search_results(
    report: &mut JavaDetectionReport,
    results: &mut Vec<JavaInfo>,
    seen: &mut HashSet<String>,
    source: JavaSearchSource,
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

    observability::java_search_completed(source.as_str(), installation_count, error_count);
    !source_failed
}

fn record_search_error(
    report: &mut JavaDetectionReport,
    source: JavaSearchSource,
    error: SearchError,
) {
    if let Some(path) = error.path.as_deref() {
        observability::java_candidate_rejected(source.as_str(), path, &error.error);
    } else {
        observability::java_search_failed(source.as_str(), &error.error);
    }

    report.errors.push(JavaDiscoveryError {
        source: source.as_str().to_string(),
        message: error.error.to_string(),
    });
}
