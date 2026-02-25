use crate::models::server::*;
use crate::services::global;
use std::path::Path;

fn manager() -> &'static crate::services::server_manager::ServerManager {
    global::server_manager()
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_server(
    name: String,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
    startup_mode: String,
) -> Result<ServerInstance, String> {
    let req = CreateServerRequest {
        name,
        core_type,
        mc_version,
        max_memory,
        min_memory,
        port,
        java_path,
        jar_path,
        startup_mode,
        custom_command: None,
    };
    manager().create_server(req)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn import_server(
    name: String,
    jar_path: String,
    startup_mode: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    online_mode: bool,
) -> Result<ServerInstance, String> {
    let req = ImportServerRequest {
        name,
        jar_path,
        startup_mode,
        custom_command: None,
        java_path,
        max_memory,
        min_memory,
        port,
        online_mode,
    };
    manager().import_server(req)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn add_existing_server(
    name: String,
    server_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    executable_path: Option<String>,
    custom_command: Option<String>,
) -> Result<ServerInstance, String> {
    let req = AddExistingServerRequest {
        name,
        server_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        executable_path,
        custom_command,
    };
    manager().add_existing_server(req)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn import_modpack(
    name: String,
    modpack_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    online_mode: bool,
    custom_command: Option<String>,
    run_path: String,
    use_software_data_dir: bool,
    startup_file_path: Option<String>,
    core_type: Option<String>,
    mc_version: Option<String>,
) -> Result<ServerInstance, String> {
    let req = ImportModpackRequest {
        name,
        modpack_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        online_mode,
        custom_command,
        run_path,
        use_software_data_dir,
        startup_file_path,
        core_type,
        mc_version,
    };
    manager().import_modpack(req)
}

#[tauri::command]
pub async fn parse_server_core_type(source_path: String) -> Result<ParsedServerCoreInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::server_installer::parse_server_core_type(&source_path)
    })
    .await
    .map_err(|e| format!("解析核心类型任务失败: {}", e))?
}

#[tauri::command]
pub async fn scan_startup_candidates(
    source_path: String,
    source_type: String,
) -> Result<StartupScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        scan_startup_candidates_blocking(source_path, source_type)
    })
    .await
    .map_err(|e| format!("扫描启动项任务失败: {}", e))?
}

fn unknown_parsed_core_info() -> ParsedServerCoreInfo {
    ParsedServerCoreInfo {
        core_type: "Unknown".to_string(),
        main_class: None,
        jar_path: None,
    }
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

fn scan_startup_candidates_blocking(
    source_path: String,
    source_type: String,
) -> Result<StartupScanResult, String> {
    const STARTER_MAIN_CLASS_PREFIX: &str = "net.neoforged.serverstarterjar";

    let source = Path::new(&source_path);
    if !source.exists() {
        return Err(format!("路径不存在: {}", source_path));
    }

    let mut candidates = Vec::new();
    let source_kind = source_type.to_ascii_lowercase();
    let core_type_options = crate::services::server_installer::CoreType::all_api_core_keys()
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<String>>();
    let mc_version_options = crate::services::server_installer::STARTER_MC_VERSION_OPTIONS
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<String>>();

    if source_kind == "archive" {
        let mut temp_extract_dir: Option<std::path::PathBuf> = None;

        let inspect_root = if source.is_file() {
            let temp_dir = std::env::temp_dir()
                .join(format!("sea_lantern_startup_scan_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("无法创建临时解压目录: {}", e))?;
            crate::services::server_installer::extract_modpack_archive(source, &temp_dir)?;
            let root_dir = crate::services::server_installer::resolve_extracted_root(&temp_dir);
            temp_extract_dir = Some(temp_dir);
            root_dir
        } else if source.is_dir() {
            source.to_path_buf()
        } else {
            return Err("archive 来源无效".to_string());
        };

        let mut parsed = crate::services::server_installer::parse_server_core_type(
            &inspect_root.to_string_lossy(),
        )?;

        if let (Some(temp_dir), Some(jar_path)) =
            (temp_extract_dir.as_ref(), parsed.jar_path.clone())
        {
            parsed.jar_path = Some(to_relative_archive_path(temp_dir, &jar_path)?);
        }

        let (detected_mc_version, mc_version_detection_failed) =
            crate::services::server_installer::detect_mc_version_from_mods(&inspect_root);
        let detected_core_type_key =
            crate::services::server_installer::CoreType::normalize_to_api_core_key(
                &parsed.core_type,
            );

        if let Some(jar_path) = parsed.jar_path.clone() {
            let is_starter = parsed
                .main_class
                .as_deref()
                .map(|main| main.starts_with(STARTER_MAIN_CLASS_PREFIX))
                .unwrap_or(false);
            let mode = if is_starter { "starter" } else { "jar" };
            let label = if is_starter { "Starter" } else { "server.jar" };
            let detail = [Some(parsed.core_type.clone()), parsed.main_class.clone()]
                .into_iter()
                .flatten()
                .collect::<Vec<String>>()
                .join(" · ");

            candidates.push(StartupCandidateItem {
                id: format!("archive-{}", mode),
                mode: mode.to_string(),
                label: label.to_string(),
                detail,
                path: jar_path,
                recommended: if is_starter { 1 } else { 3 },
            });
        }

        if let Some(temp_dir) = temp_extract_dir {
            let _ = std::fs::remove_dir_all(temp_dir);
        }

        return Ok(StartupScanResult {
            parsed_core: parsed,
            candidates,
            detected_core_type_key,
            core_type_options,
            mc_version_options,
            detected_mc_version,
            mc_version_detection_failed,
        });
    }

    if source_kind != "folder" {
        return Err("来源类型无效，仅支持 archive 或 folder".to_string());
    }

    let entries = std::fs::read_dir(source)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

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
            let parsed = crate::services::server_installer::parse_server_core_type(&full_path)
                .unwrap_or_else(|_| ParsedServerCoreInfo {
                    core_type: "Unknown".to_string(),
                    main_class: None,
                    jar_path: Some(full_path.clone()),
                });

            let is_starter = parsed
                .main_class
                .as_deref()
                .map(|main| main.starts_with(STARTER_MAIN_CLASS_PREFIX))
                .unwrap_or(false);
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

            let detail = [Some(parsed.core_type.clone()), parsed.main_class.clone()]
                .into_iter()
                .flatten()
                .collect::<Vec<String>>()
                .join(" · ");

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
                detail,
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
    let detected_core_type_key =
        crate::services::server_installer::CoreType::normalize_to_api_core_key(
            &parsed_core.core_type,
        );
    let (detected_mc_version, mc_version_detection_failed) =
        crate::services::server_installer::detect_mc_version_from_mods(source);

    Ok(StartupScanResult {
        parsed_core,
        candidates,
        detected_core_type_key,
        core_type_options,
        mc_version_options,
        detected_mc_version,
        mc_version_detection_failed,
    })
}

#[tauri::command]
pub fn collect_copy_conflicts(
    source_dir: String,
    target_dir: String,
) -> Result<Vec<String>, String> {
    let source = Path::new(&source_dir);
    let target = Path::new(&target_dir);

    if !source.exists() || !source.is_dir() {
        return Err(format!("源目录不存在或不可读: {}", source_dir));
    }

    // 只做冲突探测，不执行写入，避免误覆盖。
    let mut conflicts = Vec::new();
    collect_copy_conflicts_recursive(source, target, "", &mut conflicts)?;
    Ok(conflicts)
}

#[tauri::command]
pub fn copy_directory_contents(source_dir: String, target_dir: String) -> Result<(), String> {
    let source = Path::new(&source_dir);
    let target = Path::new(&target_dir);

    if !source.exists() || !source.is_dir() {
        return Err(format!("源目录不存在或不可读: {}", source_dir));
    }

    copy_directory_recursive(source, target).map_err(|e| format!("复制目录失败: {}", e))
}

fn collect_copy_conflicts_recursive(
    source: &Path,
    target: &Path,
    relative_prefix: &str,
    conflicts: &mut Vec<String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(source).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let source_entry = entry.path();
        let target_entry = target.join(&file_name);
        let relative = if relative_prefix.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", relative_prefix, file_name)
        };

        if target_entry.exists() {
            conflicts.push(relative.clone());
        }

        if source_entry.is_dir() {
            collect_copy_conflicts_recursive(&source_entry, &target_entry, &relative, conflicts)?;
        }
    }

    Ok(())
}

fn copy_directory_recursive(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    if !target.exists() {
        std::fs::create_dir_all(target)?;
    }

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_entry = entry.path();
        let target_entry = target.join(entry.file_name());

        if source_entry.is_dir() {
            copy_directory_recursive(&source_entry, &target_entry)?;
        } else if source_entry.is_file() {
            std::fs::copy(&source_entry, &target_entry)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn start_server(id: String) -> Result<(), String> {
    manager().start_server(&id)
}

#[tauri::command]
pub fn stop_server(id: String) -> Result<(), String> {
    manager().request_stop_server(&id)
}

#[tauri::command]
pub fn send_command(id: String, command: String) -> Result<(), String> {
    manager().send_command(&id, &command)
}

#[tauri::command]
pub fn get_server_list() -> Vec<ServerInstance> {
    manager().get_server_list()
}

#[tauri::command]
pub fn get_server_status(id: String) -> ServerStatusInfo {
    manager().get_server_status(&id)
}

#[tauri::command]
pub fn delete_server(id: String) -> Result<(), String> {
    manager().delete_server(&id)
}

#[tauri::command]
pub fn get_server_logs(id: String, since: usize) -> Vec<String> {
    manager().get_logs(&id, since)
}

#[tauri::command]
pub fn update_server_name(id: String, name: String) -> Result<(), String> {
    manager().update_server_name(&id, &name)
}
