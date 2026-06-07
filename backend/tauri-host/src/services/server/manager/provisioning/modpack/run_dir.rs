use std::path::PathBuf;

use crate::services::server::manager::provisioning::i18n::{provisioning_t, provisioning_t1};

pub(super) fn resolve_modpack_run_dir(base_path: &str) -> Result<PathBuf, String> {
    let trimmed = base_path.trim();
    if trimmed.is_empty() {
        return Err(provisioning_t("server.provisioning.run_dir_empty"));
    }

    let run_dir = PathBuf::from(trimmed);

    if run_dir.exists() {
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
}
