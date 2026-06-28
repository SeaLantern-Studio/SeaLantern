use crate::commands::server::common::{server_t, server_t1};
use crate::hardcode_data::app_files::SERVER_PATH_PERMISSION_TEST_FILE_NAME;
use crate::models::server::ValidateServerPathResult;
use sea_lantern_server_local_setup_core::resolve_local_startup_entry_checked;
use std::path::Path;

fn ensure_copy_source_dir_readable(source: &Path, source_dir: &str) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(source)
        .map_err(|_| server_t1("server.manage.source_dir_unreadable", source_dir))?;

    if !metadata.is_dir() {
        return Err(server_t1("server.manage.source_dir_unreadable", source_dir));
    }

    if metadata.file_type().is_symlink() {
        return Err(server_t1(
            "server.manage.copy_symlink_forbidden",
            source.display().to_string(),
        ));
    }

    Ok(())
}

fn copy_symlink_forbidden_error(path: &Path) -> String {
    server_t1("server.manage.copy_symlink_forbidden", path.display().to_string())
}

fn copy_symlink_forbidden_io_error(path: &Path) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::PermissionDenied, copy_symlink_forbidden_error(path))
}

/// 收集目录复制时会发生覆盖的文件路径
pub(super) fn collect_copy_conflicts(
    source_dir: String,
    target_dir: String,
) -> Result<Vec<String>, String> {
    let source = Path::new(&source_dir);
    let target = Path::new(&target_dir);

    ensure_copy_source_dir_readable(source, &source_dir)?;

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

    ensure_copy_source_dir_readable(source, &source_dir)?;

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

    for entry in entries {
        let entry = entry.map_err(|e| server_t1("server.manage.read_dir_failed", e.to_string()))?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let source_entry = entry.path();
        let target_entry = target.join(&file_name);
        let relative = if relative_prefix.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", relative_prefix, file_name)
        };
        let metadata = std::fs::symlink_metadata(&source_entry)
            .map_err(|e| server_t1("server.manage.read_dir_failed", e.to_string()))?;

        if metadata.file_type().is_symlink() {
            return Err(copy_symlink_forbidden_error(&source_entry));
        }

        if target_entry.exists() {
            conflicts.push(relative.clone());
        }

        if metadata.is_dir() {
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
        let metadata = std::fs::symlink_metadata(&source_entry)?;

        if metadata.file_type().is_symlink() {
            return Err(copy_symlink_forbidden_io_error(&source_entry));
        }

        if metadata.is_dir() {
            copy_directory_recursive(&source_entry, &target_entry)?;
        } else if metadata.is_file() {
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

#[cfg(test)]
mod tests {
    use super::{collect_copy_conflicts, copy_directory_contents};
    use std::fs;
    use std::io;
    use std::path::Path;
    use tempfile::tempdir;

    #[cfg(unix)]
    fn create_dir_symlink(link: &Path, target: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(unix)]
    fn create_file_symlink(link: &Path, target: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_dir_symlink(link: &Path, target: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_dir(target, link)
    }

    #[cfg(windows)]
    fn create_file_symlink(link: &Path, target: &Path) -> io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }

    fn skip_if_symlink_unavailable(result: io::Result<()>) -> bool {
        match result {
            Ok(()) => false,
            Err(error) => {
                eprintln!("skipping symlink test: {error}");
                true
            }
        }
    }

    #[test]
    fn collect_copy_conflicts_reports_regular_conflicts() {
        let source = tempdir().expect("create source dir");
        let target = tempdir().expect("create target dir");

        fs::create_dir_all(source.path().join("config")).expect("create source config dir");
        fs::write(source.path().join("config").join("server.properties"), b"online-mode=true")
            .expect("write source file");
        fs::create_dir_all(target.path().join("config")).expect("create target config dir");
        fs::write(target.path().join("config").join("server.properties"), b"motd=test")
            .expect("write target file");

        let conflicts = collect_copy_conflicts(
            source.path().to_string_lossy().to_string(),
            target.path().to_string_lossy().to_string(),
        )
        .expect("collect conflicts");

        assert_eq!(conflicts, vec!["config".to_string(), "config/server.properties".to_string()]);
    }

    #[test]
    fn collect_copy_conflicts_rejects_nested_directory_symlink() {
        let source = tempdir().expect("create source dir");
        let target = tempdir().expect("create target dir");
        let outside = tempdir().expect("create outside dir");
        let link = source.path().join("linked-dir");

        if skip_if_symlink_unavailable(create_dir_symlink(&link, outside.path())) {
            return;
        }

        let error = collect_copy_conflicts(
            source.path().to_string_lossy().to_string(),
            target.path().to_string_lossy().to_string(),
        )
        .expect_err("symlink source should be rejected");

        assert!(error.contains("linked-dir"), "unexpected error: {error}");
    }

    #[test]
    fn copy_directory_contents_copies_regular_files() {
        let source = tempdir().expect("create source dir");
        let target = tempdir().expect("create target dir");

        fs::create_dir_all(source.path().join("mods")).expect("create source mods dir");
        fs::write(source.path().join("mods").join("example.jar"), b"jar-bytes")
            .expect("write source file");

        copy_directory_contents(
            source.path().to_string_lossy().to_string(),
            target.path().to_string_lossy().to_string(),
        )
        .expect("copy directory contents");

        assert_eq!(
            fs::read(target.path().join("mods").join("example.jar")).expect("read copied file"),
            b"jar-bytes"
        );
    }

    #[test]
    fn copy_directory_contents_rejects_directory_symlink() {
        let source = tempdir().expect("create source dir");
        let target = tempdir().expect("create target dir");
        let outside = tempdir().expect("create outside dir");
        let link = source.path().join("world");

        if skip_if_symlink_unavailable(create_dir_symlink(&link, outside.path())) {
            return;
        }

        let error = copy_directory_contents(
            source.path().to_string_lossy().to_string(),
            target.path().to_string_lossy().to_string(),
        )
        .expect_err("symlink directory should be rejected");

        assert!(error.contains("world"), "unexpected error: {error}");
    }

    #[test]
    fn copy_directory_contents_rejects_file_symlink() {
        let source = tempdir().expect("create source dir");
        let target = tempdir().expect("create target dir");
        let outside = tempdir().expect("create outside dir");
        let target_file = outside.path().join("server.jar");
        let link = source.path().join("server.jar");

        fs::write(&target_file, b"jar-bytes").expect("write target file");

        if skip_if_symlink_unavailable(create_file_symlink(&link, &target_file)) {
            return;
        }

        let error = copy_directory_contents(
            source.path().to_string_lossy().to_string(),
            target.path().to_string_lossy().to_string(),
        )
        .expect_err("symlink file should be rejected");

        assert!(error.contains("server.jar"), "unexpected error: {error}");
    }
}
