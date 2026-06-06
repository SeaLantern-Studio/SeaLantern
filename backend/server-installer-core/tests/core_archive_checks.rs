use std::fs;
use std::io::Write;

use sea_lantern_server_installer_core::{
    detect_core_type, detect_core_type_checked, detect_mc_version_from_mods,
    detect_mc_version_from_mods_checked, find_server_jar, find_server_jar_checked,
    parse_server_core_type, resolve_extracted_root, resolve_extracted_root_checked,
};
use zip::write::FileOptions;

fn write_manifest_jar(path: &std::path::Path, manifest: &str) {
    let file = fs::File::create(path).expect("jar file should create");
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("META-INF/MANIFEST.MF", FileOptions::<()>::default())
        .expect("manifest entry should start");
    zip.write_all(manifest.as_bytes())
        .expect("manifest should write");
    zip.finish().expect("jar should finish");
}

fn write_nested_manifest_zip(path: &std::path::Path, jar_relative_path: &str, manifest: &str) {
    let file = fs::File::create(path).expect("zip file should create");
    let mut zip = zip::ZipWriter::new(file);

    let mut jar_buffer = std::io::Cursor::new(Vec::<u8>::new());
    {
        let mut jar_zip = zip::ZipWriter::new(&mut jar_buffer);
        jar_zip
            .start_file("META-INF/MANIFEST.MF", FileOptions::<()>::default())
            .expect("manifest entry should start");
        jar_zip
            .write_all(manifest.as_bytes())
            .expect("manifest should write");
        jar_zip.finish().expect("jar should finish");
    }

    zip.start_file(jar_relative_path, FileOptions::<()>::default())
        .expect("jar entry should start");
    zip.write_all(&jar_buffer.into_inner())
        .expect("jar entry should write");
    zip.finish().expect("zip should finish");
}

#[test]
fn detect_core_type_prefers_filename_and_script_neighbor_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("paper-server");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::write(server_dir.join("paper-1.20.6.jar"), b"jar").expect("jar should write");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    assert_eq!(detect_core_type("paper-1.20.6.jar"), "Paper");
    assert_eq!(
        detect_core_type(server_dir.join("start.sh").to_string_lossy().as_ref()),
        "Paper"
    );
}

#[test]
fn detect_core_type_treats_cmd_as_script_and_uses_neighbor_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("paper-server-cmd");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::write(server_dir.join("paper-1.20.6.jar"), b"jar").expect("jar should write");
    fs::write(server_dir.join("start.cmd"), b"@echo off\n").expect("cmd script should write");

    assert_eq!(
        detect_core_type(server_dir.join("start.cmd").to_string_lossy().as_ref()),
        "Paper"
    );
}

#[test]
fn detect_core_type_prefers_server_like_neighbor_jar_over_helper_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("mixed-jars");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::write(server_dir.join("z-helper.jar"), b"helper").expect("helper jar should write");
    fs::write(server_dir.join("paper-server.jar"), b"server").expect("server jar should write");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    assert_eq!(
        detect_core_type(server_dir.join("start.sh").to_string_lossy().as_ref()),
        "Paper"
    );
}

#[test]
fn detect_core_type_checked_surfaces_neighbor_jar_scan_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("broken-script-neighbor");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::create_dir(server_dir.join("server.jar")).expect("directory-backed jar path should exist");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    let error = detect_core_type_checked(server_dir.join("start.sh").to_string_lossy().as_ref())
        .expect_err("checked core detection should surface neighbor jar scan failures");

    assert!(error.contains("server.jar"), "unexpected error: {}", error);
}

#[test]
fn find_server_jar_prefers_named_server_candidate_over_first_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::write(dir.path().join("a-helper.jar"), b"helper").expect("helper jar should write");
    fs::write(dir.path().join("paper-server.jar"), b"server").expect("server jar should write");

    let jar = find_server_jar(dir.path()).expect("server jar should be found");

    assert!(jar.ends_with("paper-server.jar"));
}

#[test]
fn find_server_jar_ignores_directory_named_like_preferred_server_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::create_dir(dir.path().join("server.jar"))
        .expect("directory-backed preferred name should exist");
    fs::write(dir.path().join("paper-server.jar"), b"server").expect("server jar should write");

    let jar = find_server_jar(dir.path()).expect("real jar file should still be found");

    assert!(jar.ends_with("paper-server.jar"));
}

#[test]
fn resolve_extracted_root_uses_single_top_level_directory() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let nested = dir.path().join("modpack-root");
    fs::create_dir_all(&nested).expect("nested root should exist");

    let resolved = resolve_extracted_root(dir.path());

    assert_eq!(resolved, nested);
}

#[test]
fn resolve_extracted_root_checked_surfaces_directory_entry_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::create_dir(dir.path().join("broken-dir"))
        .expect("directory-backed entry should exist for scan failure path");

    let error = resolve_extracted_root_checked(&dir.path().join("broken-dir").join("missing"))
        .expect_err("checked extracted root resolution should surface read failures");

    assert!(error.contains("读取解压目录失败"), "unexpected error: {}", error);
}

#[test]
fn detect_mc_version_from_mods_returns_none_when_mods_dir_missing() {
    let dir = tempfile::tempdir().expect("temp dir should exist");

    let (version, failed) = detect_mc_version_from_mods(dir.path(), &["1.20.1", "1.21.1"]);

    assert_eq!(version, None);
    assert!(failed);
}

#[test]
fn detect_mc_version_from_mods_checked_surfaces_mods_scan_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let mods_path = dir.path().join("mods");
    fs::create_dir_all(mods_path.join("broken.jar"))
        .expect("directory-backed mod entry should exist");

    let error = detect_mc_version_from_mods_checked(dir.path(), &["1.20.1"])
        .expect_err("checked mc version detection should surface mods scan failures");

    assert!(
        error.contains("读取 mods 目录项失败")
            || error.contains("读取 mods 目录失败")
            || error.contains("检测到目录伪装成 mod JAR 文件"),
        "unexpected error: {}",
        error
    );
    assert!(error.contains("broken.jar"), "unexpected error: {}", error);
}

#[test]
fn find_server_jar_checked_surfaces_directory_entry_failures_when_no_valid_jar_exists() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::create_dir(dir.path().join("server.jar")).expect("directory-backed jar path should exist");

    let error = find_server_jar_checked(dir.path())
        .expect_err("checked jar scan should surface invalid directory-backed jar entries");

    assert!(error.contains("server.jar"), "unexpected error: {}", error);
}

#[test]
fn parse_server_core_type_reads_folded_manifest_main_class() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("neoforge-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.neoforged.serverstarterjar.bootstrap.\r\n Main\r\n\r\n",
    );

    let parsed = parse_server_core_type(jar_path.to_string_lossy().as_ref())
        .expect("jar manifest should parse");

    assert_eq!(parsed.core_type, "Neoforge");
    assert_eq!(
        parsed.main_class.as_deref(),
        Some("net.neoforged.serverstarterjar.bootstrap.Main")
    );
    assert_eq!(parsed.jar_path.as_deref(), Some(jar_path.to_string_lossy().as_ref()));
}

#[test]
fn parse_server_core_type_returns_relative_jar_path_for_archive_source() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let archive_path = dir.path().join("modpack.zip");
    write_nested_manifest_zip(
        &archive_path,
        "server/server.jar",
        "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n",
    );

    let parsed = parse_server_core_type(archive_path.to_string_lossy().as_ref())
        .expect("archive core type should parse");

    assert_eq!(parsed.core_type, "Paper");
    assert_eq!(parsed.main_class.as_deref(), Some("io.papermc.paperclip.Main"));
    assert_eq!(
        parsed
            .jar_path
            .as_deref()
            .map(|path| path.replace('\\', "/")),
        Some("server/server.jar".to_string())
    );
}

#[test]
fn parse_server_core_type_surfaces_archive_extract_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let archive_path = dir.path().join("broken-modpack.zip");
    fs::write(&archive_path, b"not a real zip archive").expect("broken archive should write");

    let error = parse_server_core_type(archive_path.to_string_lossy().as_ref())
        .expect_err("broken archive should fail");

    assert!(error.contains("无法解析 ZIP 压缩包"), "unexpected error: {}", error);
}

#[test]
fn parse_server_core_type_surfaces_broken_jar_manifest_read_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("paper-server.jar");
    fs::write(&jar_path, b"not a real jar archive").expect("broken jar should write");

    let error = parse_server_core_type(jar_path.to_string_lossy().as_ref())
        .expect_err("broken jar should not be downgraded to parsed core info");

    assert!(error.contains("解析服务器核心类型失败"), "unexpected error: {}", error);
    assert!(
        error.contains("解析 JAR 压缩结构失败") || error.contains("读取 JAR manifest 失败"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn parse_server_core_type_surfaces_directory_scan_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::create_dir(dir.path().join("server.jar")).expect("directory-backed jar path should exist");

    let error = parse_server_core_type(dir.path().to_string_lossy().as_ref())
        .expect_err("directory scan failure should not be downgraded to unknown core type");

    assert!(error.contains("解析服务器核心类型失败"), "unexpected error: {}", error);
    assert!(error.contains("server.jar"), "unexpected error: {}", error);
}
