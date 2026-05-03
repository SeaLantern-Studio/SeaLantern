use super::common::{
    build_result, is_starter_main_class, startup_detail, to_relative_archive_path,
};
use crate::models::server::{StartupCandidateItem, StartupScanResult};
use std::path::Path;

pub(super) fn scan_archive_source(source_path: String) -> Result<StartupScanResult, String> {
    let source = Path::new(&source_path);

    if source.is_file() {
        let extension = source
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();

        if extension == "jar" {
            let parsed = crate::services::server::installer::parse_server_core_type(&source_path)?;
            let is_starter = is_starter_main_class(&parsed);
            let mode = if is_starter { "starter" } else { "jar" };
            let label = if is_starter { "Starter" } else { "server.jar" };

            return Ok(build_result(
                parsed.clone(),
                vec![StartupCandidateItem {
                    id: format!("archive-{}", mode),
                    mode: mode.to_string(),
                    label: label.to_string(),
                    detail: startup_detail(&parsed),
                    path: source_path,
                    recommended: if is_starter { 1 } else { 3 },
                }],
                None,
                false,
            ));
        }
    }

    let mut temp_extract_dir: Option<std::path::PathBuf> = None;

    let inspect_root = if source.is_file() {
        let temp_dir = std::env::temp_dir()
            .join(format!("sea_lantern_startup_scan_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("无法创建临时解压目录: {}", e))?;
        crate::services::server::installer::extract_modpack_archive(source, &temp_dir)?;
        let root_dir = crate::services::server::installer::resolve_extracted_root(&temp_dir);
        temp_extract_dir = Some(temp_dir);
        root_dir
    } else if source.is_dir() {
        source.to_path_buf()
    } else {
        return Err("archive 来源无效".to_string());
    };

    let mut parsed = crate::services::server::installer::parse_server_core_type(
        &inspect_root.to_string_lossy(),
    )?;

    if let (Some(temp_dir), Some(jar_path)) = (temp_extract_dir.as_ref(), parsed.jar_path.clone()) {
        parsed.jar_path = Some(to_relative_archive_path(temp_dir, &jar_path)?);
    }

    let (detected_mc_version, mc_version_detection_failed) =
        crate::services::server::installer::detect_mc_version_from_mods(&inspect_root);

    let mut candidates = Vec::new();
    if let Some(jar_path) = parsed.jar_path.clone() {
        let is_starter = is_starter_main_class(&parsed);
        let mode = if is_starter { "starter" } else { "jar" };
        let label = if is_starter { "Starter" } else { "server.jar" };

        candidates.push(StartupCandidateItem {
            id: format!("archive-{}", mode),
            mode: mode.to_string(),
            label: label.to_string(),
            detail: startup_detail(&parsed),
            path: jar_path,
            recommended: if is_starter { 1 } else { 3 },
        });
    }

    if let Some(temp_dir) = temp_extract_dir {
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    Ok(build_result(
        parsed,
        candidates,
        detected_mc_version,
        mc_version_detection_failed,
    ))
}
