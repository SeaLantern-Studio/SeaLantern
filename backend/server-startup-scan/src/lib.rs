// License: GPL-3.0-only. Copyright (C) SeaLantern Studio.
//! Startup candidate scanning for folders, jars, and extracted modpack archives.

use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;
use server_installer::{CoreType, ParsedServerCoreInfo};

struct TempExtractDir(std::path::PathBuf);

impl TempExtractDir {
    fn new(prefix: &str) -> Result<Self, String> {
        let path = std::env::temp_dir().join(format!("{}_{}", prefix, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).map_err(|e| format!("无法创建临时解压目录: {}", e))?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempExtractDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Result returned after scanning a folder or archive for startup candidates.
pub struct StartupScanResult {
    pub parsed_core: ParsedServerCoreInfo,
    pub candidates: Vec<StartupCandidateItem>,
    pub detected_core_type_key: Option<String>,
    pub core_type_options: Vec<String>,
    pub mc_version_options: Vec<String>,
    pub detected_mc_version: Option<String>,
    pub mc_version_detection_failed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// One startup candidate discovered during folder or archive inspection.
pub struct StartupCandidateItem {
    pub id: String,
    pub mode: String,
    pub label: String,
    pub detail: String,
    pub path: String,
    pub recommended: u8,
}

const STARTER_MAIN_CLASS_PREFIX: &str = "net.neoforged.serverstarterjar";
const FORGE_SIMPLE_INSTALLER_MAIN_CLASS: &str = "net.minecraftforge.installer.SimpleInstaller";

fn is_pumpkin_executable(path: &Path) -> bool {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    filename.contains("pumpkin") && (extension == "exe" || extension.is_empty())
}

/// Scans a folder or archive source and ranks startup candidates for the setup flow.
pub fn scan_startup_candidates(
    source_path: &str,
    source_type: &str,
    mc_version_options: &[&str],
) -> Result<StartupScanResult, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("路径不存在: {}", source_path));
    }

    let source_kind = source_type.to_ascii_lowercase();

    if source_kind == "archive" {
        return scan_archive_source(source_path, mc_version_options);
    }

    if source_kind != "folder" {
        return Err("来源类型无效，仅支持 archive 或 folder".to_string());
    }

    scan_folder_source(source, mc_version_options)
}

fn scan_folder_source(
    source: &Path,
    mc_version_options: &[&str],
) -> Result<StartupScanResult, String> {
    let entries = collect_folder_entries_checked(source)?;

    let mut candidates = Vec::new();
    let mut detected_core: Option<(u8, bool, String, ParsedServerCoreInfo)> = None;

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

        if is_pumpkin_executable(&path) {
            let parsed_info = ParsedServerCoreInfo {
                core_type: "pumpkin".to_string(),
                main_class: None,
                jar_path: Some(full_path.clone()),
            };
            let should_replace = detected_core
                .as_ref()
                .map(|(best_recommended, best_unknown, best_label, _)| {
                    (1_u8, false, filename.to_ascii_lowercase())
                        < (*best_recommended, *best_unknown, best_label.clone())
                })
                .unwrap_or(true);

            if should_replace {
                detected_core = Some((1, false, filename.to_ascii_lowercase(), parsed_info));
            }

            candidates.push(StartupCandidateItem {
                id: format!("custom-{}", filename),
                mode: "custom".to_string(),
                label: "Pumpkin".to_string(),
                detail: "Pumpkin executable".to_string(),
                path: full_path,
                recommended: 1,
            });
            continue;
        }

        if extension == "jar" {
            let parsed = server_installer::parse_server_core_key(&full_path)
                .map_err(|error| format!("扫描启动候选失败: {}", error))?;

            let is_starter = is_starter_candidate(&parsed);
            let is_installer = is_forge_like_installer_main_class(&parsed);
            let is_server_jar = filename.eq_ignore_ascii_case("server.jar");
            let recommended = if is_starter {
                1
            } else if is_server_jar {
                3
            } else {
                4
            };
            let label = if is_installer {
                "Installer".to_string()
            } else if is_starter {
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
            let detection_rank = (
                recommended,
                is_unknown_parsed_core_info(&parsed_info),
                label.to_ascii_lowercase(),
            );
            let should_replace = detected_core
                .as_ref()
                .map(|(best_recommended, best_unknown, best_label, _)| {
                    detection_rank < (*best_recommended, *best_unknown, best_label.clone())
                })
                .unwrap_or(true);

            if should_replace {
                detected_core =
                    Some((recommended, detection_rank.1, detection_rank.2, parsed_info));
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

        if extension == "bat" || extension == "cmd" || extension == "sh" || extension == "ps1" {
            candidates.push(StartupCandidateItem {
                id: format!("{}-{}", extension, filename),
                mode: if extension == "cmd" {
                    "bat".to_string()
                } else {
                    extension
                },
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
        .map(|(_, _, _, parsed)| parsed)
        .unwrap_or_else(unknown_parsed_core_info);
    let (detected_mc_version, mc_version_detection_failed) =
        server_installer::detect_mc_version_from_mods_checked(source, mc_version_options)
            .map_err(|error| format!("扫描启动候选失败: {}", error))?;

    Ok(build_result(
        parsed_core,
        candidates,
        detected_mc_version,
        mc_version_detection_failed,
        mc_version_options,
    ))
}

fn scan_archive_source(
    source_path: &str,
    mc_version_options: &[&str],
) -> Result<StartupScanResult, String> {
    let source = Path::new(source_path);

    if source.is_file() {
        let extension = source
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();

        if extension == "jar" {
            let parsed = server_installer::parse_server_core_key(source_path)?;
            let is_starter = is_starter_candidate(&parsed);
            let is_installer = is_forge_like_installer_main_class(&parsed);
            let mode = if is_starter { "starter" } else { "jar" };
            let label = if is_installer {
                "Installer"
            } else if is_starter {
                "Starter"
            } else {
                "server.jar"
            };

            return Ok(build_result(
                parsed.clone(),
                vec![StartupCandidateItem {
                    id: format!("archive-{}", mode),
                    mode: mode.to_string(),
                    label: label.to_string(),
                    detail: startup_detail(&parsed),
                    path: source_path.to_string(),
                    recommended: if is_starter { 1 } else { 3 },
                }],
                None,
                false,
                mc_version_options,
            ));
        }

        if is_pumpkin_executable(source) {
            return Ok(build_result(
                ParsedServerCoreInfo {
                    core_type: "pumpkin".to_string(),
                    main_class: None,
                    jar_path: Some(source_path.to_string()),
                },
                vec![StartupCandidateItem {
                    id: "archive-custom-pumpkin".to_string(),
                    mode: "custom".to_string(),
                    label: "Pumpkin".to_string(),
                    detail: "Pumpkin executable".to_string(),
                    path: source_path.to_string(),
                    recommended: 1,
                }],
                None,
                false,
                mc_version_options,
            ));
        }
    }

    let mut temp_extract_dir: Option<TempExtractDir> = None;

    let inspect_root = if source.is_file() {
        let temp_dir = TempExtractDir::new("sea_lantern_startup_scan")?;
        server_installer::extract_modpack_archive(source, temp_dir.path())?;
        let root_dir = server_installer::resolve_extracted_root_checked(temp_dir.path())
            .map_err(|error| format!("扫描启动候选失败: {}", error))?;
        temp_extract_dir = Some(temp_dir);
        root_dir
    } else if source.is_dir() {
        source.to_path_buf()
    } else {
        return Err("archive 来源无效".to_string());
    };

    let mut parsed = server_installer::parse_server_core_key(&inspect_root.to_string_lossy())?;

    if let (Some(temp_dir), Some(jar_path)) = (temp_extract_dir.as_ref(), parsed.jar_path.clone()) {
        parsed.jar_path = Some(to_relative_archive_path(temp_dir.path(), &jar_path)?);
    }

    let (detected_mc_version, mc_version_detection_failed) =
        server_installer::detect_mc_version_from_mods_checked(&inspect_root, mc_version_options)
            .map_err(|error| format!("扫描启动候选失败: {}", error))?;

    let mut candidates = Vec::new();
    if let Some(jar_path) = parsed.jar_path.clone() {
        let is_starter = is_starter_candidate(&parsed);
        let is_installer = is_forge_like_installer_main_class(&parsed);
        let mode = if is_starter { "starter" } else { "jar" };
        let label = if is_installer {
            "Installer"
        } else if is_starter {
            "Starter"
        } else {
            "server.jar"
        };

        candidates.push(StartupCandidateItem {
            id: format!("archive-{}", mode),
            mode: mode.to_string(),
            label: label.to_string(),
            detail: startup_detail(&parsed),
            path: jar_path,
            recommended: if is_starter { 1 } else { 3 },
        });
    }

    Ok(build_result(
        parsed,
        candidates,
        detected_mc_version,
        mc_version_detection_failed,
        mc_version_options,
    ))
}

fn unknown_parsed_core_info() -> ParsedServerCoreInfo {
    ParsedServerCoreInfo {
        core_type: "unknown".to_string(),
        main_class: None,
        jar_path: None,
    }
}

fn collect_folder_entries_checked(source: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let entries = std::fs::read_dir(source).map_err(|e| format!("读取目录失败: {}", e))?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn is_unknown_parsed_core_info(parsed: &ParsedServerCoreInfo) -> bool {
    parsed.core_type.eq_ignore_ascii_case("unknown")
}

fn to_relative_archive_path(base_dir: &Path, absolute_path: &str) -> Result<String, String> {
    let absolute = Path::new(absolute_path);
    let relative = absolute
        .strip_prefix(base_dir)
        .map_err(|_| format!("扫描到的启动文件不在临时解压目录内: {}", absolute_path))?;

    if relative.as_os_str().is_empty() {
        return Err("扫描到的启动文件路径无效".to_string());
    }

    Ok(relative.to_string_lossy().to_string())
}

#[derive(Debug, Deserialize)]
struct SharedServerCoreTaxonomyDocument {
    entries: Vec<SharedServerCoreTaxonomyEntry>,
}

#[derive(Debug, Deserialize)]
struct SharedServerCoreTaxonomyEntry {
    key: String,
    label: String,
    #[serde(default)]
    aliases: Vec<SharedServerCoreTaxonomyAlias>,
}

#[derive(Debug, Deserialize)]
struct SharedServerCoreTaxonomyAlias {
    value: String,
    label: Option<String>,
}

fn shared_server_core_taxonomy() -> &'static SharedServerCoreTaxonomyDocument {
    static TAXONOMY: OnceLock<SharedServerCoreTaxonomyDocument> = OnceLock::new();
    TAXONOMY.get_or_init(|| {
        serde_json::from_str(include_str!("../../../shared/server-core-taxonomy.json"))
            .expect("shared server core taxonomy should be valid json")
    })
}

fn format_core_type_label(core_type: &str) -> String {
    let normalized = core_type.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return core_type.to_string();
    }

    let taxonomy = shared_server_core_taxonomy();
    for entry in &taxonomy.entries {
        for alias in &entry.aliases {
            if alias.value.eq_ignore_ascii_case(&normalized) {
                return alias.label.clone().unwrap_or_else(|| entry.label.clone());
            }
        }

        if entry.key.eq_ignore_ascii_case(&normalized) {
            return entry.label.clone();
        }
    }

    if let Some(canonical_key) = normalize_startup_scan_core_key(core_type) {
        for entry in &taxonomy.entries {
            if entry.key == canonical_key {
                return entry.label.clone();
            }
        }
    }

    core_type.to_string()
}

fn startup_detail(parsed: &ParsedServerCoreInfo) -> String {
    [Some(format_core_type_label(&parsed.core_type)), parsed.main_class.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join(" · ")
}

fn is_starter_main_class(parsed: &ParsedServerCoreInfo) -> bool {
    parsed
        .main_class
        .as_deref()
        .map(|main| main.starts_with(STARTER_MAIN_CLASS_PREFIX))
        .unwrap_or(false)
}

fn is_forge_like_installer_main_class(parsed: &ParsedServerCoreInfo) -> bool {
    parsed.main_class.as_deref() == Some(FORGE_SIMPLE_INSTALLER_MAIN_CLASS)
}

fn is_starter_candidate(parsed: &ParsedServerCoreInfo) -> bool {
    is_starter_main_class(parsed) || is_forge_like_installer_main_class(parsed)
}

fn normalize_startup_scan_core_key(input: &str) -> Option<String> {
    CoreType::normalize_to_api_core_key(input)
}

fn startup_scan_core_type_options() -> &'static [&'static str] {
    CoreType::all_api_core_keys()
}

fn build_result(
    parsed_core: ParsedServerCoreInfo,
    candidates: Vec<StartupCandidateItem>,
    detected_mc_version: Option<String>,
    mc_version_detection_failed: bool,
    mc_version_options: &[&str],
) -> StartupScanResult {
    let detected_core_type_key = normalize_startup_scan_core_key(&parsed_core.core_type);

    StartupScanResult {
        parsed_core,
        candidates,
        detected_core_type_key,
        core_type_options: startup_scan_core_type_options()
            .iter()
            .map(|value| value.to_string())
            .collect(),
        mc_version_options: mc_version_options
            .iter()
            .map(|value| value.to_string())
            .collect(),
        detected_mc_version,
        mc_version_detection_failed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_core_type_label, normalize_startup_scan_core_key, scan_startup_candidates,
        shared_server_core_taxonomy, startup_scan_core_type_options, StartupCandidateItem,
    };
    use server_installer::CoreType;
    use std::collections::HashSet;
    use std::fs;
    use std::io::Write;
    use zip::write::FileOptions;

    fn startup_scan_temp_dirs() -> HashSet<String> {
        fs::read_dir(std::env::temp_dir())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if !path.is_dir() {
                    return None;
                }

                let name = path.file_name()?.to_string_lossy().to_string();
                if name.starts_with("sea_lantern_startup_scan_") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    }

    fn candidate_modes(candidates: &[StartupCandidateItem]) -> Vec<String> {
        candidates
            .iter()
            .map(|candidate| candidate.mode.clone())
            .collect()
    }

    fn write_manifest_jar(path: &std::path::Path, manifest: &str) {
        let file = fs::File::create(path).expect("jar file should create");
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("META-INF/MANIFEST.MF", FileOptions::<()>::default())
            .expect("manifest entry should start");
        zip.write_all(manifest.as_bytes())
            .expect("manifest should write");
        zip.finish().expect("jar should finish");
    }

    #[test]
    fn scan_startup_candidates_rejects_missing_source_path() {
        let error = scan_startup_candidates("E:/missing/sea-lantern-startup-scan", "folder", &[])
            .expect_err("missing path should fail");

        assert!(error.contains("路径不存在"));
    }

    #[test]
    fn scan_startup_candidates_rejects_unknown_source_type() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let error = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "unknown", &[])
            .expect_err("unknown source type should fail");

        assert!(error.contains("来源类型无效"));
    }

    #[test]
    fn scan_startup_candidates_collects_and_sorts_script_candidates_for_folder() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        fs::write(dir.path().join("run.sh"), "#!/bin/sh\n").expect("shell script should write");
        fs::write(dir.path().join("start.bat"), "@echo off\n").expect("bat script should write");

        let result = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect("folder scan should succeed");

        assert_eq!(candidate_modes(&result.candidates), vec!["sh", "bat"]);
        assert_eq!(result.candidates[0].recommended, 2);
        assert_eq!(result.detected_core_type_key, None);
        assert_eq!(result.parsed_core.core_type, "unknown");
    }

    #[test]
    fn scan_startup_candidates_treats_ps1_as_script_candidate_on_any_host() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        fs::write(dir.path().join("launch.ps1"), "Write-Host boot\n")
            .expect("powershell script should write");

        let result = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect("folder scan should succeed");

        assert_eq!(candidate_modes(&result.candidates), vec!["ps1"]);
        assert_eq!(result.candidates[0].recommended, 2);
    }

    #[test]
    fn scan_startup_candidates_treats_cmd_as_bat_script_candidate() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        fs::write(dir.path().join("start.cmd"), "@echo off\n").expect("cmd script should write");

        let result = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect("folder scan should succeed");

        assert_eq!(candidate_modes(&result.candidates), vec!["bat"]);
        assert_eq!(result.candidates[0].label, "start.cmd");
        assert_eq!(result.candidates[0].recommended, 2);
    }

    #[test]
    fn scan_startup_candidates_uses_direct_jar_path_for_archive_source() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let jar_path = dir.path().join("paper-server.jar");
        write_manifest_jar(
            &jar_path,
            "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n",
        );

        let result = scan_startup_candidates(jar_path.to_string_lossy().as_ref(), "archive", &[])
            .expect("archive jar scan should succeed");

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].id, "archive-jar");
        assert_eq!(result.candidates[0].mode, "jar");
        assert_eq!(result.candidates[0].label, "server.jar");
        assert_eq!(result.candidates[0].path, jar_path.to_string_lossy());
        assert_eq!(result.candidates[0].recommended, 3);
        assert_eq!(
            result.parsed_core.jar_path.as_deref(),
            Some(jar_path.to_string_lossy().as_ref())
        );
        assert_eq!(result.detected_core_type_key.as_deref(), Some("paper"));
    }

    #[test]
    fn scan_startup_candidates_recognizes_pumpkin_executable_archive_source() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let exe_path = dir.path().join("pumpkin-X64-Windows.exe");
        fs::write(&exe_path, b"pumpkin").expect("pumpkin executable should write");

        let result = scan_startup_candidates(exe_path.to_string_lossy().as_ref(), "archive", &[])
            .expect("pumpkin archive scan should succeed");

        assert_eq!(result.parsed_core.core_type, "pumpkin");
        assert_eq!(result.detected_core_type_key.as_deref(), Some("pumpkin"));
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].mode, "custom");
        assert_eq!(result.candidates[0].label, "Pumpkin");
        assert_eq!(result.candidates[0].path, exe_path.to_string_lossy());
        assert_eq!(result.candidates[0].recommended, 1);
    }

    #[test]
    fn scan_startup_candidates_recognizes_pumpkin_executable_in_folder() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let exe_path = dir.path().join("pumpkin-X64-Windows.exe");
        fs::write(&exe_path, b"pumpkin").expect("pumpkin executable should write");
        fs::write(dir.path().join("start.bat"), "@echo off\n").expect("bat script should write");

        let result = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect("pumpkin folder scan should succeed");

        assert_eq!(result.parsed_core.core_type, "pumpkin");
        assert_eq!(result.detected_core_type_key.as_deref(), Some("pumpkin"));
        assert_eq!(candidate_modes(&result.candidates), vec!["custom", "bat"]);
        assert_eq!(result.candidates[0].label, "Pumpkin");
        assert_eq!(result.candidates[0].path, exe_path.to_string_lossy());
    }

    #[test]
    fn scan_startup_candidates_keeps_neoforge_type_for_legacy_simpleinstaller_manifest() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let jar_path = dir
            .path()
            .join("neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar");
        write_manifest_jar(
            &jar_path,
            "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\n\r\n",
        );

        let result = scan_startup_candidates(jar_path.to_string_lossy().as_ref(), "archive", &[])
            .expect("archive jar scan should succeed");

        assert_eq!(result.parsed_core.core_type, "neoforge");
        assert_eq!(result.detected_core_type_key.as_deref(), Some("neoforge"));
        assert_eq!(result.candidates[0].mode, "starter");
        assert_eq!(result.candidates[0].label, "Installer");
        assert_eq!(result.candidates[0].recommended, 1);
        assert!(result.candidates[0].detail.contains("NeoForge"));
    }

    #[test]
    fn scan_startup_candidates_surfaces_broken_jar_candidates_in_folder_scan() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        fs::write(dir.path().join("paper-server.jar"), b"not a real jar archive")
            .expect("broken jar should write");

        let error = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect_err("broken jar candidate should not be downgraded to unknown scan result");

        assert!(error.contains("扫描启动候选失败"), "unexpected error: {}", error);
        assert!(
            error.contains("解析 JAR 压缩结构失败") || error.contains("读取 JAR manifest 失败"),
            "unexpected error: {}",
            error
        );
    }

    #[test]
    fn scan_startup_candidates_prefers_known_core_over_unknown_helper_jar() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        write_manifest_jar(
            &dir.path().join("a-helper.jar"),
            "Manifest-Version: 1.0\r\nMain-Class: net.minecraft.client.Main\r\n\r\n",
        );
        write_manifest_jar(
            &dir.path().join("paper-server.jar"),
            "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n",
        );

        let result = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect("folder scan should succeed");

        assert_eq!(result.parsed_core.core_type, "paper");
        assert_eq!(result.detected_core_type_key.as_deref(), Some("paper"));
        assert!(result
            .parsed_core
            .jar_path
            .as_deref()
            .is_some_and(|path| path.ends_with("paper-server.jar")));
    }

    #[test]
    fn scan_startup_candidates_cleans_temp_extract_dir_when_archive_scan_fails() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let archive_path = dir.path().join("broken-modpack.zip");
        fs::write(&archive_path, b"not a real zip archive").expect("broken archive should write");

        let before = startup_scan_temp_dirs();

        let error =
            scan_startup_candidates(archive_path.to_string_lossy().as_ref(), "archive", &[])
                .expect_err("invalid archive should fail");

        let after = startup_scan_temp_dirs();

        assert!(error.contains("无法解析 ZIP 压缩包"), "unexpected error: {}", error);
        assert_eq!(after, before);
    }

    #[test]
    fn startup_scan_normalizes_published_taxonomy_aliases() {
        assert_eq!(normalize_startup_scan_core_key("Waterfall").as_deref(), Some("waterfall"));
        assert_eq!(normalize_startup_scan_core_key("AllayMC").as_deref(), Some("allay"));
        assert_eq!(
            normalize_startup_scan_core_key("bedrock-dedicated-server").as_deref(),
            Some("bds")
        );
        assert_eq!(normalize_startup_scan_core_key("Leaf").as_deref(), Some("leaves"));
        assert_eq!(normalize_startup_scan_core_key("nukkitx").as_deref(), Some("nukkit"));
        assert_eq!(normalize_startup_scan_core_key("spongeforge").as_deref(), Some("forge"));
    }

    #[test]
    fn startup_scan_options_reuse_backend_canonical_core_keys() {
        assert_eq!(startup_scan_core_type_options(), CoreType::all_api_core_keys());
    }

    #[test]
    fn shared_taxonomy_canonical_keys_match_backend_canonical_core_keys() {
        let mut shared_keys = shared_server_core_taxonomy()
            .entries
            .iter()
            .map(|entry| entry.key.clone())
            .collect::<Vec<String>>();
        let mut backend_keys = CoreType::all_api_core_keys()
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<String>>();

        shared_keys.sort_unstable();
        backend_keys.sort_unstable();

        assert_eq!(shared_keys, backend_keys);
    }

    #[test]
    fn startup_scan_result_exposes_backend_canonical_core_keys() {
        let dir = tempfile::tempdir().expect("temp dir should exist");

        let result = scan_startup_candidates(dir.path().to_string_lossy().as_ref(), "folder", &[])
            .expect("folder scan should succeed");

        assert_eq!(
            result.core_type_options,
            CoreType::all_api_core_keys()
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn startup_scan_labels_follow_shared_taxonomy_metadata() {
        assert_eq!(format_core_type_label("mohist"), "Mohist");
        assert_eq!(format_core_type_label("arclight-fabric"), "Arclight-Fabric");
        assert_eq!(format_core_type_label("Waterfall"), "Waterfall");
        assert_eq!(format_core_type_label("AllayMC"), "AllayMC");
    }
}
