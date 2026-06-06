use super::{
    check_pending_update, clear_pending_update, remove_stale_pending_update_file,
    write_pending_update,
};

#[test]
fn write_and_check_pending_update_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let update_file = dir.path().join("SeaLantern.msi");
    std::fs::write(&update_file, b"payload").unwrap();

    let pending_file = dir.path().join("pending.json");
    write_pending_update(&pending_file, &update_file.to_string_lossy(), "9.9.9".to_string())
        .unwrap();

    let pending = check_pending_update(&pending_file, "1.0.0")
        .unwrap()
        .unwrap();
    assert_eq!(pending.version, "9.9.9");
    assert_eq!(pending.file_path, update_file.to_string_lossy());
}

#[test]
fn check_pending_update_clears_missing_payload() {
    let dir = tempfile::tempdir().unwrap();
    let pending_file = dir.path().join("pending.json");
    write_pending_update(&pending_file, "missing-file.exe", "9.9.9".to_string()).unwrap();

    assert!(check_pending_update(&pending_file, "1.0.0")
        .unwrap()
        .is_none());
    assert!(!pending_file.exists());
}

#[test]
fn remove_stale_pending_update_file_surfaces_cleanup_failure_for_missing_payload() {
    let dir = tempfile::tempdir().unwrap();
    let pending_path = dir.path().join("pending.json");
    std::fs::create_dir(&pending_path).unwrap();

    let error = remove_stale_pending_update_file(&pending_path, "missing payload")
        .expect_err("directory-backed pending path should not be silently ignored");

    assert!(error.contains("Failed to remove stale pending update file"));
    assert!(error.contains("missing payload"));
}

#[test]
fn remove_stale_pending_update_file_surfaces_cleanup_failure_for_stale_version() {
    let dir = tempfile::tempdir().unwrap();
    let pending_path = dir.path().join("pending.json");
    std::fs::create_dir(&pending_path).unwrap();

    let error = remove_stale_pending_update_file(&pending_path, "version no longer pending")
        .expect_err("directory-backed pending path should not be silently ignored");

    assert!(error.contains("Failed to remove stale pending update file"));
    assert!(error.contains("version no longer pending"));
}

#[test]
fn clear_pending_update_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let pending_file = dir.path().join("pending.json");
    std::fs::write(&pending_file, "{}").unwrap();

    clear_pending_update(&pending_file).unwrap();
    assert!(!pending_file.exists());
}

#[test]
fn check_pending_update_rejects_invalid_pending_version_string() {
    let dir = tempfile::tempdir().unwrap();
    let update_file = dir.path().join("SeaLantern.msi");
    std::fs::write(&update_file, b"payload").unwrap();

    let pending_file = dir.path().join("pending.json");
    write_pending_update(&pending_file, &update_file.to_string_lossy(), "1.x.0".to_string())
        .unwrap();

    let error = check_pending_update(&pending_file, "1.0.0")
        .expect_err("invalid pending version should not be silently treated as non-pending");

    assert!(error.contains("Failed to compare pending update version"));
    assert!(error.contains("版本号无效"));
}
