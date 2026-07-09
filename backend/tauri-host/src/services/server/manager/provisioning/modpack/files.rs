use std::path::Path;

use super::super::super::fs::copy_dir_recursive;
use crate::services::server::manager::provisioning::i18n::{provisioning_t, provisioning_t1};
use sea_lantern_server_installer_core::{
    extract_modpack_archive, should_copy_modpack_source_as_native_server_binary,
};
use sea_lantern_server_local_setup_core::{path_is_child_of, paths_equal};

pub(super) fn prepare_modpack_files(source_path: &Path, run_dir: &Path) -> Result<(), String> {
    let source_file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("server.jar");
    let source_extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    if source_path.is_file() {
        std::fs::create_dir_all(run_dir).map_err(|e| {
            provisioning_t1("server.provisioning.run_dir_create_failed", e.to_string())
        })?;
        if source_extension == "jar" {
            let target_jar = run_dir.join(source_file_name);
            std::fs::copy(source_path, &target_jar).map_err(|e| {
                provisioning_t1("server.provisioning.copy_jar_failed", e.to_string())
            })?;
        } else if should_copy_modpack_source_as_native_server_binary(source_path) {
            let target_file = run_dir.join(source_file_name);
            std::fs::copy(source_path, &target_file).map_err(|e| {
                provisioning_t1("server.provisioning.copy_jar_failed", e.to_string())
            })?;
        } else {
            extract_modpack_archive(source_path, run_dir)?;
        }
        return Ok(());
    }

    if source_path.is_dir() {
        if !paths_equal(source_path, run_dir) {
            if path_is_child_of(run_dir, source_path) {
                return Err(provisioning_t("server.provisioning.run_dir_inside_source"));
            }
            std::fs::create_dir_all(run_dir).map_err(|e| {
                provisioning_t1("server.provisioning.run_dir_create_failed", e.to_string())
            })?;
            copy_dir_recursive(source_path, run_dir).map_err(|e| {
                provisioning_t1("server.provisioning.copy_modpack_files_failed", e.to_string())
            })?;
        }
        return Ok(());
    }

    Err(provisioning_t("server.provisioning.invalid_modpack_path"))
}

#[cfg(test)]
mod tests {
    use super::prepare_modpack_files;

    #[test]
    fn prepare_modpack_files_copies_native_server_binary_without_extracting() {
        let source_dir = tempfile::tempdir().expect("temp dir should exist");
        let run_dir = tempfile::tempdir().expect("temp dir should exist");
        let exe_path = source_dir.path().join("pumpkin-X64-Windows.exe");
        std::fs::write(&exe_path, b"pumpkin").expect("pumpkin executable should write");
        let target_dir = run_dir.path().join("server-root");

        prepare_modpack_files(&exe_path, &target_dir).expect("pumpkin executable should copy");

        assert!(target_dir.join("pumpkin-X64-Windows.exe").exists());
    }
}
