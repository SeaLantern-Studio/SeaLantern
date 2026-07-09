use std::fs;
use std::io::Write;

use server_installer::{
    detect_core_key, detect_core_key_checked, detect_core_type, detect_core_type_checked,
    detect_mc_version_from_mods, detect_mc_version_from_mods_checked, find_server_jar,
    find_server_jar_checked, parse_server_core_key, parse_server_core_type, resolve_extracted_root,
    resolve_extracted_root_checked, resolve_imported_server_core_key, resolve_starter_core_key,
    resolve_starter_core_key_checked, should_copy_modpack_source_as_native_server_binary,
    should_delay_starter_runtime_file_writes, CoreType, DockerTypeResolution,
    StarterCoreKeyResolution, StarterInstallArgs,
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
fn detect_core_key_prefers_filename_and_script_neighbor_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("paper-server");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::write(server_dir.join("paper-1.20.6.jar"), b"jar").expect("jar should write");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    assert_eq!(detect_core_key("paper-1.20.6.jar"), "paper");
    assert_eq!(detect_core_key(server_dir.join("start.sh").to_string_lossy().as_ref()), "paper");
}

#[test]
fn detect_core_key_treats_cmd_as_script_and_uses_neighbor_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("paper-server-cmd");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::write(server_dir.join("paper-1.20.6.jar"), b"jar").expect("jar should write");
    fs::write(server_dir.join("start.cmd"), b"@echo off\n").expect("cmd script should write");

    assert_eq!(
        detect_core_key(server_dir.join("start.cmd").to_string_lossy().as_ref()),
        "paper"
    );
}

#[test]
fn detect_core_key_prefers_server_like_neighbor_jar_over_helper_jar() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("mixed-jars");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::write(server_dir.join("z-helper.jar"), b"helper").expect("helper jar should write");
    fs::write(server_dir.join("paper-server.jar"), b"server").expect("server jar should write");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    assert_eq!(detect_core_key(server_dir.join("start.sh").to_string_lossy().as_ref()), "paper");
}

#[test]
fn detect_core_key_checked_surfaces_neighbor_jar_scan_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("broken-script-neighbor");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::create_dir(server_dir.join("server.jar")).expect("directory-backed jar path should exist");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    let error = detect_core_key_checked(server_dir.join("start.sh").to_string_lossy().as_ref())
        .expect_err("checked core detection should surface neighbor jar scan failures");

    assert!(error.contains("server.jar"), "unexpected error: {}", error);
}

#[test]
fn docker_type_resolution_reuses_canonical_core_key_normalization() {
    assert_eq!(
        CoreType::docker_type_resolution("bedrock"),
        Some(DockerTypeResolution {
            api_core_key: "bds".to_string(),
            docker_type_value: "BDS".to_string(),
        })
    );
    assert_eq!(
        CoreType::docker_type_resolution("Arclight-Neoforge"),
        Some(DockerTypeResolution {
            api_core_key: "arclight-neoforge".to_string(),
            docker_type_value: "ARCLIGHT_NEOFORGE".to_string(),
        })
    );
    assert_eq!(
        CoreType::docker_type_resolution("Banner"),
        Some(DockerTypeResolution {
            api_core_key: "fabric".to_string(),
            docker_type_value: "FABRIC".to_string(),
        })
    );
    assert_eq!(CoreType::docker_type_resolution("   "), None);
}

#[test]
fn resolve_starter_core_key_prefers_explicit_core_and_falls_back_to_detected_path() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir
        .path()
        .join("neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\n\r\n",
    );

    assert_eq!(
        resolve_starter_core_key(Some("bedrock"), Some(jar_path.to_string_lossy().as_ref())),
        Some("bds".to_string())
    );
    assert_eq!(
        resolve_starter_core_key(None, Some(jar_path.to_string_lossy().as_ref())),
        Some("neoforge".to_string())
    );
    assert_eq!(resolve_starter_core_key(Some("   "), Some("   ")), None);
}

#[test]
fn resolve_starter_core_key_checked_preserves_non_starter_empty_and_detected_fallback() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir
        .path()
        .join("neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\n\r\n",
    );

    assert_eq!(
        resolve_starter_core_key_checked(
            "jar",
            Some("paper"),
            Some(jar_path.to_string_lossy().as_ref())
        ),
        StarterCoreKeyResolution {
            starter_core_key: String::new(),
            detected_core_key: String::new(),
        }
    );

    assert_eq!(
        resolve_starter_core_key_checked(
            "starter",
            None,
            Some(jar_path.to_string_lossy().as_ref())
        ),
        StarterCoreKeyResolution {
            starter_core_key: "neoforge".to_string(),
            detected_core_key: "neoforge".to_string(),
        }
    );

    assert_eq!(
        resolve_starter_core_key_checked(
            "starter",
            Some("bedrock"),
            Some(jar_path.to_string_lossy().as_ref())
        ),
        StarterCoreKeyResolution {
            starter_core_key: "bds".to_string(),
            detected_core_key: "neoforge".to_string(),
        }
    );
}

#[test]
fn starter_core_key_resolution_reports_only_real_unrecognized_starter_cases() {
    assert!(!StarterCoreKeyResolution {
        starter_core_key: String::new(),
        detected_core_key: String::new(),
    }
    .needs_unrecognized_error("jar"));

    assert!(StarterCoreKeyResolution {
        starter_core_key: String::new(),
        detected_core_key: "unknown-core".to_string(),
    }
    .needs_unrecognized_error("starter"));

    assert!(!StarterCoreKeyResolution {
        starter_core_key: "neoforge".to_string(),
        detected_core_key: "neoforge".to_string(),
    }
    .needs_unrecognized_error("starter"));
}

#[test]
fn starter_core_key_resolution_prefers_explicit_display_hint_then_detected_fallback() {
    assert_eq!(
        StarterCoreKeyResolution {
            starter_core_key: String::new(),
            detected_core_key: "neoforge".to_string(),
        }
        .unresolved_display_hint(Some("  bedrock  ")),
        "bedrock"
    );

    assert_eq!(
        StarterCoreKeyResolution {
            starter_core_key: String::new(),
            detected_core_key: "neoforge".to_string(),
        }
        .unresolved_display_hint(Some("   ")),
        "neoforge"
    );
}

#[test]
fn starter_install_args_match_shared_family_cli_contracts() {
    assert_eq!(
        CoreType::starter_install_args("neoforge"),
        Some(StarterInstallArgs {
            args: vec!["--install-server", ".", "--server-starter"],
        })
    );
    assert_eq!(
        CoreType::starter_install_args("catserver"),
        Some(StarterInstallArgs { args: vec!["--installServer", "."] })
    );
    assert_eq!(
        CoreType::starter_install_args("paper"),
        Some(StarterInstallArgs { args: vec!["--install-server", "."] })
    );
    assert_eq!(CoreType::starter_install_args("   "), None);
}

#[test]
fn resolve_imported_server_core_key_handles_custom_startup_and_detected_jars() {
    assert_eq!(
        resolve_imported_server_core_key("custom", "E:/servers/pumpkin-launcher.exe"),
        "pumpkin"
    );
    assert_eq!(
        resolve_imported_server_core_key("custom", "E:/servers/run-server.bat"),
        "custom"
    );
    assert_eq!(resolve_imported_server_core_key("jar", "E:/servers/nukkit.jar"), "nukkit");
}

#[test]
fn should_copy_modpack_source_as_native_server_binary_handles_exe_and_pumpkin_binaries() {
    assert!(should_copy_modpack_source_as_native_server_binary(std::path::Path::new(
        "E:/servers/pumpkin-X64-Windows.exe"
    )));
    assert!(should_copy_modpack_source_as_native_server_binary(std::path::Path::new(
        "E:/servers/pumpkin-server"
    )));
    assert!(!should_copy_modpack_source_as_native_server_binary(std::path::Path::new(
        "E:/servers/server-start"
    )));
}

#[test]
fn should_delay_starter_runtime_file_writes_only_for_starter_jar_sources() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("neoforge-installer.jar");
    std::fs::write(&jar_path, b"jar").expect("jar fixture should be created");

    assert!(should_delay_starter_runtime_file_writes("starter", &jar_path));
    assert!(!should_delay_starter_runtime_file_writes("jar", &jar_path));

    let folder_path = dir.path().join("modpack-dir");
    std::fs::create_dir_all(&folder_path).expect("folder fixture should be created");
    assert!(!should_delay_starter_runtime_file_writes("starter", &folder_path));
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
fn parse_server_core_key_reads_folded_manifest_main_class() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("neoforge-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.neoforged.serverstarterjar.bootstrap.\r\n Main\r\n\r\n",
    );

    let parsed = parse_server_core_key(jar_path.to_string_lossy().as_ref())
        .expect("jar manifest should parse");

    assert_eq!(parsed.core_type, "neoforge");
    assert_eq!(
        parsed.main_class.as_deref(),
        Some("net.neoforged.serverstarterjar.bootstrap.Main")
    );
    assert_eq!(parsed.jar_path.as_deref(), Some(jar_path.to_string_lossy().as_ref()));
}

#[test]
fn parse_server_core_key_keeps_neoforge_filename_when_installer_main_class_is_legacy_forge() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir
        .path()
        .join("neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\n\r\n",
    );

    let parsed = parse_server_core_key(jar_path.to_string_lossy().as_ref())
        .expect("neoforge installer manifest should parse");

    assert_eq!(parsed.core_type, "neoforge");
    assert_eq!(
        parsed.main_class.as_deref(),
        Some("net.minecraftforge.installer.SimpleInstaller")
    );
}

#[test]
fn parse_server_core_key_returns_canonical_key_for_neoforge_installer() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir
        .path()
        .join("neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\n\r\n",
    );

    let parsed = parse_server_core_key(jar_path.to_string_lossy().as_ref())
        .expect("canonical neoforge installer manifest should parse");

    assert_eq!(parsed.core_type, "neoforge");
    assert_eq!(
        parsed.main_class.as_deref(),
        Some("net.minecraftforge.installer.SimpleInstaller")
    );
}

#[test]
fn parse_server_core_key_keeps_arclight_neoforge_filename_when_installer_main_class_is_legacy_forge(
) {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir
        .path()
        .join("arclight-neoforge-26.1.0.0-alpha.1+snapshot-1-installer.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: net.minecraftforge.installer.SimpleInstaller\r\n\r\n",
    );

    let parsed = parse_server_core_key(jar_path.to_string_lossy().as_ref())
        .expect("arclight neoforge installer manifest should parse");

    assert_eq!(parsed.core_type, "arclight-neoforge");
    assert_eq!(
        parsed.main_class.as_deref(),
        Some("net.minecraftforge.installer.SimpleInstaller")
    );
}

#[test]
fn parse_server_core_key_returns_relative_jar_path_for_archive_source() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let archive_path = dir.path().join("modpack.zip");
    write_nested_manifest_zip(
        &archive_path,
        "server/server.jar",
        "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n",
    );

    let parsed = parse_server_core_key(archive_path.to_string_lossy().as_ref())
        .expect("archive core type should parse");

    assert_eq!(parsed.core_type, "paper");
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
fn parse_server_core_key_keeps_detected_key_when_archive_relative_path_cannot_be_re_detected() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let archive_path = dir.path().join("relative-archive-modpack.zip");
    write_nested_manifest_zip(
        &archive_path,
        "server/server.jar",
        "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n",
    );

    let parsed = parse_server_core_key(archive_path.to_string_lossy().as_ref())
        .expect("archive core key should parse");

    assert_eq!(parsed.core_type, "paper");
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
fn parse_server_core_key_surfaces_archive_extract_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let archive_path = dir.path().join("broken-modpack.zip");
    fs::write(&archive_path, b"not a real zip archive").expect("broken archive should write");

    let error = parse_server_core_key(archive_path.to_string_lossy().as_ref())
        .expect_err("broken archive should fail");

    assert!(error.contains("无法解析 ZIP 压缩包"), "unexpected error: {}", error);
}

#[test]
fn parse_server_core_key_surfaces_broken_jar_manifest_read_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("paper-server.jar");
    fs::write(&jar_path, b"not a real jar archive").expect("broken jar should write");

    let error = parse_server_core_key(jar_path.to_string_lossy().as_ref())
        .expect_err("broken jar should not be downgraded to parsed core info");

    assert!(error.contains("解析服务器核心类型失败"), "unexpected error: {}", error);
    assert!(
        error.contains("解析 JAR 压缩结构失败") || error.contains("读取 JAR manifest 失败"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn parse_server_core_key_surfaces_directory_scan_failures() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    fs::create_dir(dir.path().join("server.jar")).expect("directory-backed jar path should exist");

    let error = parse_server_core_key(dir.path().to_string_lossy().as_ref())
        .expect_err("directory scan failure should not be downgraded to unknown core type");

    assert!(error.contains("解析服务器核心类型失败"), "unexpected error: {}", error);
    assert!(error.contains("server.jar"), "unexpected error: {}", error);
}

#[test]
fn legacy_core_type_helpers_remain_available_for_display_compatibility() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("paper-server.jar");
    write_manifest_jar(
        &jar_path,
        "Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n",
    );

    assert_eq!(detect_core_type(jar_path.to_string_lossy().as_ref()), "Paper");

    let parsed = parse_server_core_type(jar_path.to_string_lossy().as_ref())
        .expect("legacy parser should stay available");
    assert_eq!(parsed.core_type, "Paper");
}

#[test]
fn legacy_core_type_checked_helpers_remain_available_for_error_compatibility() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let server_dir = dir.path().join("broken-script-neighbor-legacy");
    fs::create_dir_all(&server_dir).expect("server dir should exist");
    fs::create_dir(server_dir.join("server.jar")).expect("directory-backed jar path should exist");
    fs::write(server_dir.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

    let error = detect_core_type_checked(server_dir.join("start.sh").to_string_lossy().as_ref())
        .expect_err("legacy checked detection should surface neighbor jar scan failures");

    assert!(error.contains("server.jar"), "unexpected error: {}", error);
}
