use super::common::{app_t, app_t1};
use crate::services::global;
use crate::utils::logger::{format_log_entry, to_log_line, LogLine, GLOBAL_LOG_COLLECTOR};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use tauri::command;

fn ensure_developer_mode() -> Result<(), String> {
    if global::settings_manager().get().developer_mode {
        Ok(())
    } else {
        Err(app_t("app.logging.developer_mode_disabled"))
    }
}

fn resolve_export_path(save_path: &str) -> Result<PathBuf, String> {
    let save = Path::new(save_path);

    if !save.is_absolute() {
        return Err(app_t("app.logging.save_path_must_be_absolute"));
    }

    let file_name = save
        .file_name()
        .ok_or_else(|| app_t("app.logging.invalid_save_path"))?;
    if file_name == "." || file_name == ".." {
        return Err(app_t("app.logging.invalid_save_path"));
    }

    let allowed_root =
        dirs_next::home_dir().ok_or_else(|| app_t("app.logging.user_home_unavailable"))?;
    let canonical_root = std::fs::canonicalize(&allowed_root)
        .map_err(|e| app_t1("app.logging.user_home_canonicalize_failed", e.to_string()))?;

    let parent = save
        .parent()
        .ok_or_else(|| app_t("app.logging.invalid_save_path"))?;
    let canonical_parent = resolve_parent_within_root(parent, &canonical_root)?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err(app_t("app.logging.save_path_must_be_within_home"));
    }

    match std::fs::symlink_metadata(save) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(app_t("app.logging.save_path_symlink_forbidden"));
            }

            let canonical_target = std::fs::canonicalize(save)
                .map_err(|e| app_t1("app.logging.invalid_save_path_with_detail", e.to_string()))?;
            if !canonical_target.starts_with(&canonical_root) {
                return Err(app_t("app.logging.save_path_must_be_within_home"));
            }

            Ok(canonical_target)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(canonical_parent.join(file_name)),
        Err(error) => Err(app_t1("app.logging.invalid_save_path_with_detail", error.to_string())),
    }
}

fn resolve_parent_within_root(parent: &Path, canonical_root: &Path) -> Result<PathBuf, String> {
    for component in parent.components() {
        if matches!(component, Component::ParentDir) {
            return Err(app_t("app.logging.invalid_save_path"));
        }
    }

    let mut existing_ancestor = parent;
    let mut pending_components = Vec::new();

    while !existing_ancestor.exists() {
        let name = existing_ancestor
            .file_name()
            .ok_or_else(|| app_t("app.logging.invalid_save_path"))?;
        pending_components.push(name.to_os_string());
        existing_ancestor = existing_ancestor
            .parent()
            .ok_or_else(|| app_t("app.logging.invalid_save_path"))?;
    }

    let mut resolved_parent = std::fs::canonicalize(existing_ancestor)
        .map_err(|e| app_t1("app.logging.invalid_save_path_with_detail", e.to_string()))?;
    if !resolved_parent.starts_with(canonical_root) {
        return Err(app_t("app.logging.save_path_must_be_within_home"));
    }

    for component in pending_components.iter().rev() {
        resolved_parent.push(component);
    }

    Ok(resolved_parent)
}

fn write_logs_to_file(lines: &[String], save_path: &str) -> Result<(), String> {
    let resolved_path = resolve_export_path(save_path)?;
    if let Some(parent) = resolved_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| app_t1("app.logging.create_save_dir_failed", e.to_string()))?;
    }
    std::fs::write(resolved_path, lines.join("\n"))
        .map_err(|e| app_t1("app.logging.save_failed", e.to_string()))
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
