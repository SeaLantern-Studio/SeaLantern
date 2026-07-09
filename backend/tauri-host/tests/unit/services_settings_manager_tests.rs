use super::*;
use crate::commands::app::settings::persist_personalization_background_for_test;
use crate::test_support::{lock_env, EnvGuard};

#[test]
fn load_settings_backs_up_corrupt_file_and_returns_default() {
    let temp_dir = std::env::temp_dir().join(format!(
        "sea-lantern-settings-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    std::fs::create_dir_all(&temp_dir).expect("create temp settings dir");

    let settings_path = temp_dir.join(SETTINGS_FILE);
    std::fs::write(&settings_path, "{invalid json").expect("write corrupt settings");

    let loaded = load_settings(temp_dir.to_string_lossy().as_ref());
    assert_eq!(loaded.agreed_to_terms, AppSettings::default().agreed_to_terms);

    let backup_count = std::fs::read_dir(&temp_dir)
        .expect("read temp dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains(&format!("{}.bak-corrupt-", SETTINGS_FILE))
        })
        .count();

    assert_eq!(backup_count, 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn load_settings_checked_backs_up_corrupt_file_and_surfaces_error() {
    let temp_dir = std::env::temp_dir().join(format!(
        "sea-lantern-settings-checked-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    std::fs::create_dir_all(&temp_dir).expect("create temp settings dir");

    let settings_path = temp_dir.join(SETTINGS_FILE);
    std::fs::write(&settings_path, "{invalid json").expect("write corrupt settings");

    let error = load_settings_checked(temp_dir.to_string_lossy().as_ref())
        .expect_err("corrupt settings should not silently downgrade in checked path");

    assert!(error.contains("Failed to parse settings"), "unexpected error: {}", error);
    assert!(
        error.contains(settings_path.to_string_lossy().as_ref()),
        "unexpected error: {}",
        error
    );

    let backup_count = std::fs::read_dir(&temp_dir)
        .expect("read temp dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains(&format!("{}.bak-corrupt-", SETTINGS_FILE))
        })
        .count();

    assert_eq!(backup_count, 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn persist_personalization_background_surfaces_app_data_dir_creation_failures() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let source_path = temp_dir.path().join("background.png");
    std::fs::write(&source_path, b"fake image bytes").expect("background image should exist");

    let blocked_root = temp_dir.path().join("blocked-root");
    std::fs::write(&blocked_root, b"not a directory")
        .expect("file-backed app data root should exist");
    let blocked_path = blocked_root.join("nested");
    let _env_lock = lock_env();
    let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

    let error = persist_personalization_background_for_test(&source_path)
        .expect_err("app data dir failure should not be silently downgraded");

    assert!(
        error.contains("Failed to resolve app data directory"),
        "unexpected error: {}",
        error
    );
    assert!(error.contains("blocked-root"), "unexpected error: {}", error);
}

#[test]
fn settings_manager_new_checked_surfaces_app_data_dir_creation_failures() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let blocked_root = temp_dir.path().join("blocked-root");
    std::fs::write(&blocked_root, b"not a directory")
        .expect("file-backed app data root should exist");
    let blocked_path = blocked_root.join("nested");
    let _env_lock = lock_env();
    let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

    let error = SettingsManager::new_checked()
        .err()
        .expect("app data dir failure should not be silently downgraded");

    assert!(
        error.contains("Failed to resolve app data directory"),
        "unexpected error: {}",
        error
    );
    assert!(error.contains("blocked-root"), "unexpected error: {}", error);
}

#[test]
fn settings_manager_new_checked_recovers_from_corrupt_settings_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let settings_path = temp_dir.path().join(SETTINGS_FILE);
    std::fs::write(&settings_path, "{invalid json").expect("write corrupt settings");

    let _env_lock = lock_env();
    let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());

    let manager = SettingsManager::new_checked()
        .expect("corrupt settings should be recovered during bootstrap initialization");

    assert_eq!(manager.get().agreed_to_terms, AppSettings::default().agreed_to_terms);

    let backup_count = std::fs::read_dir(temp_dir.path())
        .expect("read temp dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains(&format!("{}.bak-corrupt-", SETTINGS_FILE))
        })
        .count();

    assert_eq!(backup_count, 1);
    assert!(settings_path.exists(), "settings file should be recreated");
}

#[test]
fn settings_manager_new_checked_salvages_valid_fields_from_structured_corrupt_settings() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let settings_path = temp_dir.path().join(SETTINGS_FILE);
    std::fs::write(
        &settings_path,
        r#"{
  "theme": "dark",
  "font_size": 18,
  "default_max_memory": "invalid"
}"#,
    )
    .expect("write partially salvageable settings");

    let _env_lock = lock_env();
    let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &temp_dir.path().to_string_lossy());

    let manager = SettingsManager::new_checked()
        .expect("bootstrap should salvage valid settings fields from structured corruption");
    let settings = manager.get();

    assert_eq!(settings.theme, "dark");
    assert_eq!(settings.font_size, 18);
    assert_eq!(settings.default_max_memory, AppSettings::default().default_max_memory);
}
