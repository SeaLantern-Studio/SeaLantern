use crate::commands::server::common::{server_t, server_t1};
use crate::hardcode_data::app_files::SERVER_PATH_PERMISSION_TEST_FILE_NAME;
use crate::models::server::ValidateServerPathResult;
use sea_lantern_server_local_setup_core::resolve_local_startup_entry_checked;
use std::path::Path;

/// 收集目录复制时会发生覆盖的文件路径
pub(super) fn collect_copy_conflicts(
    source_dir: String,
    target_dir: String,
) -> Result<Vec<String>, String> {
    let source = Path::new(&source_dir);
    let target = Path::new(&target_dir);

    if !source.exists() || !source.is_dir() {
        return Err(server_t1("server.manage.source_dir_unreadable", source_dir));
    }

    // 只做冲突探测，不执行写入，避免误覆盖。
    let mut conflicts = Vec::new();
    collect_copy_conflicts_recursive(source, target, "", &mut conflicts)?;
    Ok(conflicts)
}

/// 复制目录内容
pub(super) fn copy_directory_contents(
    source_dir: String,
    target_dir: String,
) -> Result<(), String> {
    let source = Path::new(&source_dir);
    let target = Path::new(&target_dir);

    if !source.exists() || !source.is_dir() {
        return Err(server_t1("server.manage.source_dir_unreadable", source_dir));
    }

    copy_directory_recursive(source, target)
        .map_err(|e| server_t1("server.manage.copy_directory_failed", e.to_string()))
}

/// 校验服务器目录是否可写，并尝试识别启动文件
pub(super) fn validate_server_path(new_path: String) -> Result<ValidateServerPathResult, String> {
    let path = std::path::Path::new(&new_path);

    let test_file = path.join(SERVER_PATH_PERMISSION_TEST_FILE_NAME);
    if std::fs::write(&test_file, "").is_err() {
        return Ok(ValidateServerPathResult {
            valid: false,
            message: server_t("server.manage.server_dir_write_denied"),
            message_key: Some("server.manage.server_dir_write_denied".to_string()),
            jar_path: None,
            startup_mode: None,
        });
    }
    let _ = std::fs::remove_file(&test_file);

    let (jar_path, startup_mode) = match find_server_executable_for_validation(path) {
        Ok((jar, mode)) => (Some(jar), Some(mode)),
        Err(_) => (None, None),
    };

    let valid = jar_path.is_some();
    let message = if valid {
        server_t("server.manage.validate_path_success")
    } else {
        server_t("server.manage.validate_executable_missing")
    };

    Ok(ValidateServerPathResult {
        valid,
        message,
        message_key: Some(
            if valid {
                "server.manage.validate_path_success"
            } else {
                "server.manage.validate_executable_missing"
            }
            .to_string(),
        ),
        jar_path,
        startup_mode,
    })
}

fn collect_copy_conflicts_recursive(
    source: &Path,
    target: &Path,
    relative_prefix: &str,
    conflicts: &mut Vec<String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(source)
        .map_err(|e| server_t1("server.manage.read_dir_failed", e.to_string()))?;

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

fn find_server_executable_for_validation(
    server_path: &std::path::Path,
) -> Result<(String, String), String> {
    if let Some((path, mode)) = resolve_local_startup_entry_checked(server_path)? {
        return Ok((path, mode));
    }

    Err(server_t("server.manage.startup_file_unavailable"))
}
