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
    };
    manager().add_existing_server(req)
}

#[tauri::command]
pub fn import_modpack(
    name: String,
    modpack_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
) -> Result<ServerInstance, String> {
    let req = ImportModpackRequest {
        name,
        modpack_path,
        java_path,
        max_memory,
        min_memory,
        port,
    };
    manager().import_modpack(req)
}

#[tauri::command]
pub fn parse_server_core_type(source_path: String) -> Result<ParsedServerCoreInfo, String> {
    crate::services::server_installer::parse_server_core_type(&source_path)
}

#[tauri::command]
pub fn scan_startup_candidates(
    source_path: String,
    source_type: String,
) -> Result<Vec<StartupCandidateItem>, String> {
    const STARTER_MAIN_CLASS_PREFIX: &str = "net.neoforged.serverstarterjar";

    let source = Path::new(&source_path);
    if !source.exists() {
        return Err(format!("路径不存在: {}", source_path));
    }

    let mut candidates = Vec::new();
    let source_kind = source_type.to_ascii_lowercase();

    // 压缩包来源无法先看到脚本文件，所以这里先返回 starter/server.jar 候选。
    if source_kind == "archive" {
        let parsed = crate::services::server_installer::parse_server_core_type(&source_path)?;
        if let Some(jar_path) = parsed.jar_path {
            let is_starter = parsed
                .main_class
                .as_deref()
                .map(|main| main.starts_with(STARTER_MAIN_CLASS_PREFIX))
                .unwrap_or(false);
            let mode = if is_starter { "starter" } else { "jar" };
            let label = if is_starter { "Starter" } else { "server.jar" };
            let detail = [Some(parsed.core_type), parsed.main_class]
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

        return Ok(candidates);
    }

    if source_kind != "folder" {
        return Err("来源类型无效，仅支持 archive 或 folder".to_string());
    }

    // 目录扫描在后端执行，避免前端并发读取文件系统造成卡顿。
    let entries = std::fs::read_dir(source)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    // 文件夹来源：逐个文件识别，统一返回给前端做选择展示。
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
                .unwrap_or(ParsedServerCoreInfo {
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
            let label = if is_starter {
                "Starter".to_string()
            } else if is_server_jar {
                "server.jar".to_string()
            } else {
                filename.clone()
            };

            let detail = [Some(parsed.core_type), parsed.main_class]
                .into_iter()
                .flatten()
                .collect::<Vec<String>>()
                .join(" · ");

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
                recommended: if is_starter {
                    1
                } else if is_server_jar {
                    3
                } else {
                    4
                },
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

    Ok(candidates)
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
