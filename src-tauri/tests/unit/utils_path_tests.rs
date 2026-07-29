use super::*;

#[test]
fn test_get_app_data_dir_not_empty() {
    let dir = get_app_data_dir();
    assert!(!dir.as_path().as_os_str().is_empty());
}

#[test]
fn test_get_or_create_app_data_dir() {
    let dir_str = get_or_create_app_data_dir();
    assert!(!dir_str.is_empty());

    let path = PathBuf::from(&dir_str);
    assert!(path.exists());
    assert!(path.is_dir());
}
