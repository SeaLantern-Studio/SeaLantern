use std::path::Path;

use crate::utils::cli::server_args::CliServerCommand;

use super::local_folder_inspection::inspect_local_folder;

pub(super) fn detect_startup_mode_from_folder(folder: &Path) -> String {
    inspect_local_folder(folder)
        .startup_mode
        .unwrap_or_else(|| "jar".to_string())
}

pub(super) fn resolve_existing_attach_entry_path(folder: &Path, entry: &str) -> Option<String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return None;
    }

    let direct = Path::new(trimmed);
    if direct.exists() {
        return Some(direct.to_string_lossy().to_string());
    }

    let relative_to_folder = folder.join(trimmed);
    if relative_to_folder.exists() {
        return Some(relative_to_folder.to_string_lossy().to_string());
    }

    None
}

pub(super) fn resolve_existing_local_entry_path(
    folder: Option<&Path>,
    entry: &str,
) -> Option<String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else if let Some(folder) = folder {
        let relative_to_folder = folder.join(path);
        if relative_to_folder.exists() {
            Some(relative_to_folder.to_string_lossy().to_string())
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn resolve_custom_entry_hint_path(
    entry: Option<&str>,
    resolved_entry_path: Option<&str>,
    folder: Option<&Path>,
) -> Option<String> {
    if let Some(path) = resolved_entry_path {
        return Some(path.to_string());
    }

    let entry = entry?.trim();
    if entry.is_empty() {
        return None;
    }

    let tokens = shlex::split(entry)?;
    if tokens.is_empty() {
        return None;
    }

    for window in tokens.windows(2) {
        if window[0].eq_ignore_ascii_case("-jar") {
            return resolve_command_path_hint(&window[1], folder);
        }
    }

    resolve_command_path_hint(&tokens[0], folder)
}

pub(super) fn resolve_command_path_hint(token: &str, folder: Option<&Path>) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    let looks_like_launch_path = path.is_absolute()
        || trimmed.contains(['/', '\\'])
        || trimmed.starts_with('.')
        || matches!(extension.as_deref(), Some("jar" | "bat" | "sh" | "ps1" | "cmd"));

    if !looks_like_launch_path {
        return None;
    }

    if path.is_absolute() {
        Some(path.to_string_lossy().to_string())
    } else if let Some(folder) = folder {
        Some(folder.join(path).to_string_lossy().to_string())
    } else {
        std::env::current_dir()
            .ok()
            .map(|current_dir| current_dir.join(path).to_string_lossy().to_string())
            .or_else(|| Some(trimmed.to_string()))
    }
}

pub(super) fn infer_local_create_startup_mode(
    command: &CliServerCommand,
    resolved_entry_path: Option<&str>,
) -> String {
    if let Some(entry_path) = resolved_entry_path {
        return detect_startup_mode_from_path_like(entry_path);
    }
    if command.entry.is_some() {
        return "custom".to_string();
    }
    "jar".to_string()
}

pub(super) fn validate_local_entry_startup_mode(
    startup_mode: &str,
    entry: Option<&str>,
    resolved_entry_path: Option<&str>,
) -> Result<(), String> {
    let Some(entry) = entry.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    if startup_mode == "custom" {
        return Ok(());
    }

    let Some(entry_path) = resolved_entry_path else {
        return Err(format!(
            "--startup {} 需要可解析的启动文件路径；当前 --entry 更像命令文本，请改用 --startup custom 或提供实际脚本/JAR 路径",
            startup_mode
        ));
    };

    let detected_mode = detect_startup_mode_from_path_like(entry_path);
    if detected_mode != startup_mode {
        return Err(format!(
            "--startup {} 与 --entry={} 的文件类型不匹配，检测到的是 {}",
            startup_mode, entry, detected_mode
        ));
    }

    Ok(())
}

pub(super) fn detect_startup_mode_from_path_like(path: &str) -> String {
    let extension = Path::new(path)
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

pub(super) fn normalize_cli_startup_mode(value: Option<&str>) -> Result<String, String> {
    let raw = value.unwrap_or("jar").trim().to_ascii_lowercase();
    match raw.as_str() {
        "jar" | "bat" | "sh" | "ps1" | "starter" | "custom" => Ok(raw),
        _ => Err(format!("不支持的 startup mode: {}", raw)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_startup_mode_from_folder, detect_startup_mode_from_path_like,
        infer_local_create_startup_mode, normalize_cli_startup_mode, resolve_command_path_hint,
        resolve_custom_entry_hint_path, resolve_existing_attach_entry_path,
        resolve_existing_local_entry_path, validate_local_entry_startup_mode,
    };
    use crate::utils::cli::server_args::CliServerCommand;
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
        let command = CliServerCommand::default();
        assert_eq!(infer_local_create_startup_mode(&command, Some("E:/srv/start.ps1")), "ps1");
    }

    #[test]
    fn infer_local_create_startup_mode_uses_custom_for_non_path_entry() {
        let command = CliServerCommand {
            entry: Some("java -jar server.jar nogui".to_string()),
            ..Default::default()
        };
        assert_eq!(infer_local_create_startup_mode(&command, None), "custom");
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
