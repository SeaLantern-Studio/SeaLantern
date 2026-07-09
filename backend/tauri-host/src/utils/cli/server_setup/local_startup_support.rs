#[cfg(test)]
pub(super) use sea_lantern_server_local_setup_core::detect_startup_mode_from_path_like;
pub(super) use sea_lantern_server_local_setup_core::{
    detect_startup_mode_from_folder, infer_local_create_startup_mode, normalize_cli_startup_mode,
    resolve_command_path_hint, resolve_custom_entry_hint_path, resolve_existing_attach_entry_path,
    resolve_existing_local_entry_path, validate_local_entry_startup_mode,
};

#[cfg(test)]
mod tests {
    use super::{
        detect_startup_mode_from_folder, detect_startup_mode_from_path_like,
        infer_local_create_startup_mode, normalize_cli_startup_mode, resolve_command_path_hint,
        resolve_custom_entry_hint_path, resolve_existing_attach_entry_path,
        resolve_existing_local_entry_path, validate_local_entry_startup_mode,
    };
    use crate::utils::cli::server_setup::local_folder_inspection::inspect_local_folder;
    use tempfile::tempdir;

    #[test]
    fn should_attach_existing_local_folder_detects_existing_server_shape() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("start.sh"), b"#!/bin/sh\n").unwrap();

        assert!(inspect_local_folder(temp_dir.path()).is_attachable());
    }

    #[test]
    fn should_attach_existing_local_folder_rejects_empty_target_folder() {
        let temp_dir = tempdir().expect("temp dir should exist");

        assert!(!inspect_local_folder(temp_dir.path()).is_attachable());
    }

    #[test]
    fn detect_startup_mode_from_folder_prefers_known_scripts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("start.ps1"), b"Write-Host start\n").unwrap();

        assert_eq!(detect_startup_mode_from_folder(temp_dir.path()), "ps1");
    }

    #[test]
    fn should_attach_existing_local_folder_accepts_custom_named_root_script() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(temp_dir.path().join("fabric-prod.ps1"), b"Write-Host start\n").unwrap();

        assert!(inspect_local_folder(temp_dir.path()).is_attachable());
        assert_eq!(detect_startup_mode_from_folder(temp_dir.path()), "ps1");
    }

    #[test]
    fn resolve_existing_attach_entry_path_checks_folder_relative_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("run.sh");
        std::fs::write(&script_path, b"#!/bin/sh\n").unwrap();

        let resolved = resolve_existing_attach_entry_path(temp_dir.path(), "run.sh")
            .expect("relative folder path should resolve");
        assert_eq!(resolved, script_path.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_existing_local_entry_path_returns_none_for_command_text() {
        assert!(resolve_existing_local_entry_path(None, "java -jar server.jar nogui").is_none());
    }

    #[test]
    fn resolve_existing_local_entry_path_checks_folder_relative_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let script_path = temp_dir.path().join("start.bat");
        std::fs::write(&script_path, b"@echo off\r\n").unwrap();

        let resolved = resolve_existing_local_entry_path(Some(temp_dir.path()), "start.bat")
            .expect("folder-relative local entry should resolve");

        assert_eq!(resolved, script_path.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_custom_entry_hint_path_extracts_jar_after_dash_jar() {
        let resolved = resolve_custom_entry_hint_path(
            Some("java -Xmx4G -Xms4G -jar E:/srv/server.jar nogui"),
            None,
            None,
        )
        .expect("-jar command should yield server jar path");

        assert_eq!(resolved.replace('\\', "/"), "E:/srv/server.jar");
    }

    #[test]
    fn resolve_custom_entry_hint_path_prefers_existing_entry_path_for_custom_mode() {
        let resolved = resolve_custom_entry_hint_path(
            Some("E:/srv/start.bat"),
            Some("E:/srv/start.bat"),
            None,
        )
        .expect("existing script path should be preserved as working-dir hint");

        assert_eq!(resolved.replace('\\', "/"), "E:/srv/start.bat");
    }

    #[test]
    fn resolve_command_path_hint_rejects_plain_program_name_without_path_context() {
        assert!(resolve_command_path_hint("java", None).is_none());
    }

    #[test]
    fn resolve_command_path_hint_prefers_folder_context_for_relative_paths() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let resolved = resolve_command_path_hint("server.jar", Some(temp_dir.path()))
            .expect("relative jar path should bind to folder context");

        assert_eq!(
            resolved,
            temp_dir
                .path()
                .join("server.jar")
                .to_string_lossy()
                .to_string()
        );
    }

    #[test]
    fn infer_local_create_startup_mode_prefers_existing_entry_path_extension() {
        assert_eq!(infer_local_create_startup_mode(false, Some("E:/srv/start.ps1")), "ps1");
    }

    #[test]
    fn infer_local_create_startup_mode_uses_custom_for_non_path_entry() {
        assert_eq!(infer_local_create_startup_mode(true, None), "custom");
    }

    #[test]
    fn validate_local_entry_startup_mode_allows_custom_with_existing_script_path() {
        validate_local_entry_startup_mode(
            "custom",
            Some("E:/srv/start.bat"),
            Some("E:/srv/start.bat"),
        )
        .expect("explicit custom mode should accept existing script path as command input");
    }

    #[test]
    fn validate_local_entry_startup_mode_rejects_script_mode_for_command_text() {
        let err = validate_local_entry_startup_mode("sh", Some("java -jar server.jar nogui"), None)
            .expect_err("script startup mode should reject command text");
        assert!(err.contains("--startup sh"));
        assert!(err.contains("custom"));
    }

    #[test]
    fn validate_local_entry_startup_mode_rejects_mismatched_file_extension() {
        let err = validate_local_entry_startup_mode(
            "sh",
            Some("E:/srv/start.bat"),
            Some("E:/srv/start.bat"),
        )
        .expect_err("mismatched file extension should fail fast");
        assert!(err.contains("不匹配"));
        assert!(err.contains("bat"));
    }

    #[test]
    fn detect_startup_mode_from_path_like_matches_supported_scripts() {
        assert_eq!(detect_startup_mode_from_path_like("E:/srv/start.bat"), "bat");
        assert_eq!(detect_startup_mode_from_path_like("E:/srv/start.sh"), "sh");
        assert_eq!(detect_startup_mode_from_path_like("E:/srv/start.ps1"), "ps1");
        assert_eq!(detect_startup_mode_from_path_like("E:/srv/server.jar"), "jar");
    }

    #[test]
    fn normalize_cli_startup_mode_rejects_unknown_value() {
        let err = normalize_cli_startup_mode(Some("python")).expect_err("unknown mode should fail");
        assert!(err.contains("startup mode"));
    }
}
