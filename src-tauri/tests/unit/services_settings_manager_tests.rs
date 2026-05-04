use super::*;

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
                .contains("settings.bak-corrupt-")
        })
        .count();

    assert_eq!(backup_count, 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
