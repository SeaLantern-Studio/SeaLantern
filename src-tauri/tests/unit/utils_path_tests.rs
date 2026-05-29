use super::*;
use std::fs;

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

#[test]
fn test_get_app_data_dir_prefers_explicit_env_override() {
    let expected = std::env::temp_dir().join("sealantern-data-dir-override-test");
    std::env::set_var("SEALANTERN_DATA_DIR", expected.to_string_lossy().to_string());

    let actual = get_app_data_dir();

    std::env::remove_var("SEALANTERN_DATA_DIR");
    assert_eq!(actual, expected);
}

#[test]
fn test_find_root_startup_file_prefers_scripts_over_jar() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    fs::write(temp_dir.path().join("server.jar"), b"jar").unwrap();
    fs::write(temp_dir.path().join("fabric-boot.ps1"), b"Write-Host boot\n").unwrap();

    let resolved = find_root_startup_file(temp_dir.path()).expect("startup file should resolve");

    assert_eq!(resolved.file_name().and_then(|name| name.to_str()), Some("fabric-boot.ps1"));
}

#[test]
fn test_find_root_startup_file_prefers_bat_then_sh_then_ps1() {
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    fs::write(temp_dir.path().join("z-launch.ps1"), b"Write-Host boot\n").unwrap();
    fs::write(temp_dir.path().join("a-run.sh"), b"#!/bin/sh\n").unwrap();
    fs::write(temp_dir.path().join("m-start.bat"), b"@echo off\r\n").unwrap();

    let resolved = find_root_startup_file(temp_dir.path()).expect("startup file should resolve");

    assert_eq!(resolved.file_name().and_then(|name| name.to_str()), Some("m-start.bat"));
}

#[test]
fn test_strip_path_prefix_for_compare_supports_unix_style_prefixes_on_windows() {
    let remainder = strip_path_prefix_for_compare(
        std::path::Path::new("/app/data/servers/paper-docker"),
        std::path::Path::new("/app/data/servers"),
    )
    .expect("unix-style prefix should strip");

    assert_eq!(remainder, "paper-docker");
}

#[test]
fn test_strip_path_prefix_for_compare_returns_empty_for_same_path() {
    let remainder = strip_path_prefix_for_compare(
        std::path::Path::new("E:/srv/sealantern/servers"),
        std::path::Path::new("E:/srv/sealantern/servers"),
    )
    .expect("same path should strip to empty remainder");

    assert!(remainder.is_empty());
}
