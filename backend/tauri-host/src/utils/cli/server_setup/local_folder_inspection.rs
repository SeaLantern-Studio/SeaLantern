use std::path::{Path, PathBuf};

use crate::services::server::installer::{detect_core_type, find_server_jar};
use crate::utils::path::find_root_startup_file;

use super::local_startup_support::detect_startup_mode_from_path_like;
use super::metadata_support::infer_mc_version_hint;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct LocalFolderInspection {
    pub startup_entry_path: Option<String>,
    pub startup_mode: Option<String>,
    pub detected_jar_path: Option<String>,
    pub inferred_core_type: Option<String>,
    pub inferred_mc_version: Option<String>,
}

impl LocalFolderInspection {
    pub(super) fn is_attachable(&self) -> bool {
        self.startup_entry_path.is_some() || self.detected_jar_path.is_some()
    }

    pub(super) fn preferred_startup_path(&self) -> Option<&str> {
        self.startup_entry_path
            .as_deref()
            .or(self.detected_jar_path.as_deref())
    }

    pub(super) fn describe(&self) -> String {
        format!(
            "attachable={} startup_mode={} startup_entry={} jar={} core={} mc={}",
            self.is_attachable(),
            self.startup_mode.as_deref().unwrap_or("none"),
            self.startup_entry_path.as_deref().unwrap_or("none"),
            self.detected_jar_path.as_deref().unwrap_or("none"),
            self.inferred_core_type.as_deref().unwrap_or("unknown"),
            self.inferred_mc_version.as_deref().unwrap_or("unknown")
        )
    }
}

pub(super) fn inspect_local_folder(folder: &Path) -> LocalFolderInspection {
    if !folder.exists() || !folder.is_dir() {
        return LocalFolderInspection::default();
    }

    let startup_entry_path =
        resolve_attach_executable_path(folder).map(|path| path.to_string_lossy().to_string());
    let detected_jar_path = find_server_jar(folder).ok();
    let startup_mode = startup_entry_path
        .as_deref()
        .map(detect_startup_mode_from_path_like)
        .or_else(|| detected_jar_path.as_ref().map(|_| "jar".to_string()));

    let folder_display = folder.to_string_lossy();
    let inferred_core_type = startup_entry_path
        .as_deref()
        .map(detect_core_type)
        .or_else(|| detected_jar_path.as_deref().map(detect_core_type))
        .or_else(|| Some(detect_core_type(folder_display.as_ref())))
        .filter(|value| !value.eq_ignore_ascii_case("unknown"));
    let inferred_mc_version = startup_entry_path
        .as_deref()
        .and_then(|value| infer_mc_version_hint(&[value]))
        .or_else(|| {
            detected_jar_path
                .as_deref()
                .and_then(|value| infer_mc_version_hint(&[value]))
        })
        .or_else(|| infer_mc_version_hint(&[folder_display.as_ref()]));

    LocalFolderInspection {
        startup_entry_path,
        startup_mode,
        detected_jar_path,
        inferred_core_type,
        inferred_mc_version,
    }
}

pub(super) fn resolve_attach_executable_path(folder: &Path) -> Option<PathBuf> {
    let preferred_scripts = [
        "start.bat",
        "run.bat",
        "launch.bat",
        "start.sh",
        "run.sh",
        "launch.sh",
        "start.ps1",
        "run.ps1",
        "launch.ps1",
    ];

    preferred_scripts
        .iter()
        .find_map(|script| {
            let script_path = folder.join(script);
            if script_path.exists() {
                Some(script_path)
            } else {
                None
            }
        })
        .or_else(|| {
            find_root_startup_file(folder).and_then(|path| {
                if is_script_path(&path) {
                    Some(path)
                } else {
                    None
                }
            })
        })
}

fn is_script_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("bat" | "sh" | "ps1")
    )
}

#[cfg(test)]
mod tests {
    use super::inspect_local_folder;
    use tempfile::tempdir;

    #[test]
    fn inspect_local_folder_detects_attachable_script_and_metadata() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let inspection = inspect_local_folder(&folder);
        assert!(inspection.is_attachable());
        assert_eq!(inspection.startup_mode.as_deref(), Some("sh"));
        assert_eq!(inspection.inferred_core_type.as_deref(), Some("Paper"));
        assert_eq!(inspection.inferred_mc_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn inspect_local_folder_detects_attachable_jar_without_script() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("fabric-server.jar"), b"placeholder").expect("jar should write");

        let inspection = inspect_local_folder(&folder);
        assert!(inspection.is_attachable());
        assert_eq!(inspection.startup_mode.as_deref(), Some("jar"));
        assert!(inspection.startup_entry_path.is_none());
        assert!(inspection.detected_jar_path.is_some());
        assert_eq!(inspection.inferred_core_type.as_deref(), Some("Fabric"));
        assert_eq!(inspection.inferred_mc_version.as_deref(), Some("1.20.1"));
    }

    #[test]
    fn inspect_local_folder_rejects_empty_folder_shape() {
        let temp_dir = tempdir().expect("temp dir should exist");

        let inspection = inspect_local_folder(temp_dir.path());
        assert!(!inspection.is_attachable());
        assert!(inspection.startup_mode.is_none());
    }
}
