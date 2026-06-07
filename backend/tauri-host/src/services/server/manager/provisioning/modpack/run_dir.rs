use std::path::PathBuf;

use crate::services::server::manager::provisioning::i18n::{provisioning_t, provisioning_t1};

pub(super) fn resolve_modpack_run_dir(base_path: &str) -> Result<PathBuf, String> {
    let trimmed = base_path.trim();
    if trimmed.is_empty() {
        return Err(provisioning_t("server.provisioning.run_dir_empty"));
    }

    let run_dir = PathBuf::from(trimmed);

    if run_dir.exists() {
        if !run_dir.is_dir() {
            return Err(provisioning_t1(
                "server.provisioning.run_dir_exists",
                run_dir.to_string_lossy().to_string(),
            ));
        }

        let mut entries =
            std::fs::read_dir(&run_dir).map_err(|e| format!("读取运行目录失败: {}", e))?;
        if entries.next().is_none() {
            return Ok(run_dir);
        }

        return Err(provisioning_t1(
            "server.provisioning.run_dir_exists",
            run_dir.to_string_lossy().to_string(),
        ));
    }

    Ok(run_dir)
}

#[cfg(test)]
mod tests {
    use super::resolve_modpack_run_dir;
    use tempfile::tempdir;

    #[test]
    fn resolve_modpack_run_dir_keeps_explicit_user_selected_directory() {
        let dir = tempdir().expect("temp dir should exist");
        let target = dir.path().join("neoforgeserver");

        let resolved = resolve_modpack_run_dir(target.to_string_lossy().as_ref())
            .expect("explicit run dir should resolve");

        assert_eq!(resolved, target);
    }

    #[test]
    fn resolve_modpack_run_dir_allows_existing_empty_directory() {
        let dir = tempdir().expect("temp dir should exist");
        let target = dir.path().join("empty-run-dir");
        std::fs::create_dir_all(&target).expect("run dir should create");

        let resolved = resolve_modpack_run_dir(target.to_string_lossy().as_ref())
            .expect("existing empty directory should be accepted");

        assert_eq!(resolved, target);
    }

    #[test]
    fn resolve_modpack_run_dir_rejects_existing_non_empty_directory() {
        let dir = tempdir().expect("temp dir should exist");
        let target = dir.path().join("non-empty-run-dir");
        std::fs::create_dir_all(&target).expect("run dir should create");
        std::fs::write(target.join("server.jar"), b"jar").expect("fixture should write");

        let error = resolve_modpack_run_dir(target.to_string_lossy().as_ref())
            .expect_err("non-empty directory should still be rejected");

        assert!(error.contains("已存在") || error.contains("exists"));
    }
}
