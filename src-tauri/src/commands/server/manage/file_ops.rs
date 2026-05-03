use crate::hardcode_data::app_files::SERVER_PATH_PERMISSION_TEST_FILE_NAME;
use crate::models::server::ValidateServerPathResult;
use std::path::Path;

/// 收集目录复制时会发生覆盖的文件路径
pub(super) fn collect_copy_conflicts(
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

/// 复制目录内容
pub(super) fn copy_directory_contents(
    source_dir: String,
    target_dir: String,
) -> Result<(), String> {
    let source = Path::new(&source_dir);
    let target = Path::new(&target_dir);

    if !source.exists() || !source.is_dir() {
        return Err(format!("源目录不存在或不可读: {}", source_dir));
    }

    copy_directory_recursive(source, target).map_err(|e| format!("复制目录失败: {}", e))
}

/// 校验服务器目录是否可写，并尝试识别启动文件
pub(super) fn validate_server_path(
    new_path: String,
) -> Result<ValidateServerPathResult, String> {
    let path = std::path::Path::new(&new_path);

    let test_file = path.join(SERVER_PATH_PERMISSION_TEST_FILE_NAME);
    if std::fs::write(&test_file, "").is_err() {
        return Ok(ValidateServerPathResult {
            valid: false,
            message: "无法写入服务器目录，请检查权限".to_string(),
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
        "路径验证成功，找到可执行文件".to_string()
    } else {
        "未找到可执行文件（.jar/.bat/.sh/.ps1），请确保路径正确".to_string()
    };

    Ok(ValidateServerPathResult {
        valid,
        message,
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

fn find_server_executable_for_validation(
    server_path: &std::path::Path,
) -> Result<(String, String), String> {
    let preferred_scripts = [
        "start.bat",
        "run.bat",
        "launch.bat",
        "start.sh",
        "run.sh",
        "launch.sh",
        "start.ps1",
        "run.ps1",
        "launch.ps1",
    ];

    for script in preferred_scripts {
        let script_path = server_path.join(script);
        if script_path.exists() {
            let mode = detect_startup_mode_from_path(&script_path);
            return Ok((script_path.to_string_lossy().to_string(), mode));
        }
    }

    let server_jar = server_path.join("server.jar");
    if server_jar.exists() {
        return Ok((server_jar.to_string_lossy().to_string(), "jar".to_string()));
    }

    let entries =
        std::fs::read_dir(server_path).map_err(|e| format!("无法读取服务器目录: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();

        if extension == "jar" || extension == "bat" || extension == "sh" || extension == "ps1" {
            let mode = detect_startup_mode_from_path(&path);
            return Ok((path.to_string_lossy().to_string(), mode));
        }
    }

    Err("未找到可用的启动文件".to_string())
}

fn detect_startup_mode_from_path(path: &std::path::Path) -> String {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "bat" => "bat".to_string(),
        "sh" => "sh".to_string(),
        "ps1" => "ps1".to_string(),
        _ => "jar".to_string(),
    }
}
