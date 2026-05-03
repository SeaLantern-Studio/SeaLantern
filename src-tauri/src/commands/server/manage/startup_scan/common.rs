use crate::models::server::{ParsedServerCoreInfo, StartupCandidateItem, StartupScanResult};
use std::path::Path;

pub(super) const STARTER_MAIN_CLASS_PREFIX: &str = "net.neoforged.serverstarterjar";

pub(super) fn unknown_parsed_core_info() -> ParsedServerCoreInfo {
    ParsedServerCoreInfo {
        core_type: "Unknown".to_string(),
        main_class: None,
        jar_path: None,
    }
}

pub(super) fn to_relative_archive_path(
    base_dir: &Path,
    absolute_path: &str,
) -> Result<String, String> {
    let absolute = Path::new(absolute_path);
    let relative = absolute
        .strip_prefix(base_dir)
        .map_err(|_| format!("扫描到的启动文件不在临时解压目录内: {}", absolute_path))?;

    if relative.as_os_str().is_empty() {
        return Err("扫描到的启动文件路径无效".to_string());
    }

    Ok(relative.to_string_lossy().to_string())
}

pub(super) fn core_type_options() -> Vec<String> {
    crate::services::server::installer::CoreType::all_api_core_keys()
        .iter()
        .map(|value| value.to_string())
        .collect()
}

pub(super) fn mc_version_options() -> Vec<String> {
    crate::utils::constants::STARTER_MC_VERSION_OPTIONS
        .iter()
        .map(|value| value.to_string())
        .collect()
}

pub(super) fn startup_detail(parsed: &ParsedServerCoreInfo) -> String {
    [Some(parsed.core_type.clone()), parsed.main_class.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join(" · ")
}

pub(super) fn is_starter_main_class(parsed: &ParsedServerCoreInfo) -> bool {
    parsed
        .main_class
        .as_deref()
        .map(|main| main.starts_with(STARTER_MAIN_CLASS_PREFIX))
        .unwrap_or(false)
}

pub(super) fn build_result(
    parsed_core: ParsedServerCoreInfo,
    candidates: Vec<StartupCandidateItem>,
    detected_mc_version: Option<String>,
    mc_version_detection_failed: bool,
) -> StartupScanResult {
    let detected_core_type_key = crate::services::server::installer::CoreType::normalize_to_api_core_key(
        &parsed_core.core_type,
    );

    StartupScanResult {
        parsed_core,
        candidates,
        detected_core_type_key,
        core_type_options: core_type_options(),
        mc_version_options: mc_version_options(),
        detected_mc_version,
        mc_version_detection_failed,
    }
}
