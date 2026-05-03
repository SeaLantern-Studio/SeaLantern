use super::common::{build_result, is_starter_main_class, startup_detail, unknown_parsed_core_info};
use crate::models::server::{ParsedServerCoreInfo, StartupCandidateItem, StartupScanResult};
use std::path::Path;

pub(super) fn scan_folder_source(source: &Path) -> Result<StartupScanResult, String> {
    let entries = std::fs::read_dir(source)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    let mut candidates = Vec::new();
    let mut detected_core: Option<(u8, ParsedServerCoreInfo)> = None;

    for path in entries {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let full_path = path.to_string_lossy().to_string();

        if extension == "jar" {
            let parsed = crate::services::server::installer::parse_server_core_type(&full_path)
                .unwrap_or_else(|_| ParsedServerCoreInfo {
                    core_type: "Unknown".to_string(),
                    main_class: None,
                    jar_path: Some(full_path.clone()),
                });

            let is_starter = is_starter_main_class(&parsed);
            let is_server_jar = filename.eq_ignore_ascii_case("server.jar");
            let recommended = if is_starter {
                1
            } else if is_server_jar {
                3
            } else {
                4
            };
            let label = if is_starter {
                "Starter".to_string()
            } else if is_server_jar {
                "server.jar".to_string()
            } else {
                filename.clone()
            };

            let parsed_info = ParsedServerCoreInfo {
                core_type: parsed.core_type.clone(),
                main_class: parsed.main_class.clone(),
                jar_path: Some(full_path.clone()),
            };
            if detected_core
                .as_ref()
                .map(|(best_recommended, _)| recommended < *best_recommended)
                .unwrap_or(true)
            {
                detected_core = Some((recommended, parsed_info));
            }

            candidates.push(StartupCandidateItem {
                id: format!("jar-{}", filename),
                mode: if is_starter {
                    "starter".to_string()
                } else {
                    "jar".to_string()
                },
                label,
                detail: startup_detail(&parsed),
                path: full_path,
                recommended,
            });
            continue;
        }

        if extension == "bat" || extension == "sh" || (cfg!(windows) && extension == "ps1") {
            candidates.push(StartupCandidateItem {
                id: format!("{}-{}", extension, filename),
                mode: extension,
                label: filename,
                detail: "Script".to_string(),
                path: full_path,
                recommended: 2,
            });
        }
    }

    candidates.sort_by(|a, b| {
        a.recommended
            .cmp(&b.recommended)
            .then_with(|| a.label.cmp(&b.label))
    });

    let parsed_core = detected_core
        .map(|(_, parsed)| parsed)
        .unwrap_or_else(unknown_parsed_core_info);
    let (detected_mc_version, mc_version_detection_failed) =
        crate::services::server::installer::detect_mc_version_from_mods(source);

    Ok(build_result(
        parsed_core,
        candidates,
        detected_mc_version,
        mc_version_detection_failed,
    ))
}
