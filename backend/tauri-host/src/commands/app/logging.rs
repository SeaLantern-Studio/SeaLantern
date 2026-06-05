use crate::services::global;
use crate::utils::logger::{format_log_entry, to_log_line, LogLine, GLOBAL_LOG_COLLECTOR};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use tauri::command;

fn ensure_developer_mode() -> Result<(), String> {
    if global::settings_manager().get().developer_mode {
        Ok(())
    } else {
        Err("开发者模式未开启".to_string())
    }
}

fn resolve_export_path(save_path: &str) -> Result<PathBuf, String> {
    let save = Path::new(save_path);

    if !save.is_absolute() {
        return Err("保存路径必须是绝对路径".to_string());
    }

    let file_name = save
        .file_name()
        .ok_or_else(|| "无效的保存路径".to_string())?;
    if file_name == "." || file_name == ".." {
        return Err("无效的保存路径".to_string());
    }

    let allowed_root = dirs_next::home_dir().ok_or_else(|| "无法获取用户目录".to_string())?;
    let canonical_root =
        std::fs::canonicalize(&allowed_root).map_err(|e| format!("无法规范化用户目录: {}", e))?;

    let parent = save.parent().ok_or_else(|| "无效的保存路径".to_string())?;
    let canonical_parent = resolve_parent_within_root(parent, &canonical_root)?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err("保存路径必须在用户目录内".to_string());
    }

    match std::fs::symlink_metadata(save) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err("保存路径不能是符号链接".to_string());
            }

            let canonical_target =
                std::fs::canonicalize(save).map_err(|e| format!("无效的保存路径: {}", e))?;
            if !canonical_target.starts_with(&canonical_root) {
                return Err("保存路径必须在用户目录内".to_string());
            }

            Ok(canonical_target)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(canonical_parent.join(file_name)),
        Err(error) => Err(format!("无效的保存路径: {}", error)),
    }
}

fn resolve_parent_within_root(parent: &Path, canonical_root: &Path) -> Result<PathBuf, String> {
    for component in parent.components() {
        if matches!(component, Component::ParentDir) {
            return Err("无效的保存路径".to_string());
        }
    }

    let mut existing_ancestor = parent;
    let mut pending_components = Vec::new();

    while !existing_ancestor.exists() {
        let name = existing_ancestor
            .file_name()
            .ok_or_else(|| "无效的保存路径".to_string())?;
        pending_components.push(name.to_os_string());
        existing_ancestor = existing_ancestor
            .parent()
            .ok_or_else(|| "无效的保存路径".to_string())?;
    }

    let mut resolved_parent =
        std::fs::canonicalize(existing_ancestor).map_err(|e| format!("无效的保存路径: {}", e))?;
    if !resolved_parent.starts_with(canonical_root) {
        return Err("保存路径必须在用户目录内".to_string());
    }

    for component in pending_components.iter().rev() {
        resolved_parent.push(component);
    }

    Ok(resolved_parent)
}

fn write_logs_to_file(lines: &[String], save_path: &str) -> Result<(), String> {
    let resolved_path = resolve_export_path(save_path)?;
    if let Some(parent) = resolved_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建保存目录失败: {}", e))?;
    }
    std::fs::write(resolved_path, lines.join("\n")).map_err(|e| format!("保存失败: {}", e))
}

#[command]
pub fn get_logs(limit: Option<usize>) -> Result<Vec<LogLine>, String> {
    ensure_developer_mode()?;
    Ok(GLOBAL_LOG_COLLECTOR
        .get_logs(limit)
        .into_iter()
        .map(to_log_line)
        .collect())
}

#[command]
pub fn clear_logs() -> Result<(), String> {
    ensure_developer_mode()?;
    GLOBAL_LOG_COLLECTOR.clear();
    Ok(())
}

#[command]
pub fn export_app_logs(save_path: String) -> Result<(), String> {
    ensure_developer_mode()?;
    let lines = GLOBAL_LOG_COLLECTOR
        .get_logs(None)
        .into_iter()
        .map(|entry| format_log_entry(&entry))
        .collect::<Vec<_>>();

    write_logs_to_file(&lines, &save_path)
}

#[command]
pub fn check_developer_mode() -> bool {
    global::settings_manager().get().developer_mode
}
