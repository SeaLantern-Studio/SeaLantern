//! Java 下载与安装

use crate::services::download::common::{download_t, download_t1};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Window};

fn resolve_java_install_app_dir() -> Result<std::path::PathBuf, String> {
    crate::utils::path::get_or_create_app_data_dir_checked()
        .map(std::path::PathBuf::from)
        .map_err(|e| download_t1("download.java.app_data_dir_resolve_failed", e.to_string()))
}

/// 下载并安装 Java 运行时
pub async fn download_and_install_java<R: tauri::Runtime>(
    url: String,
    version_name: String,
    window: Window<R>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<String, String> {
    let app_dir = resolve_java_install_app_dir()?;
    let java_bin = java_installer::download_and_install_java(
        &url,
        &version_name,
        &app_dir,
        cancel_flag.as_ref(),
        |payload| {
            let _ = window.emit("java-install-progress", payload);
        },
    )
    .await?;

    if cancel_flag.load(Ordering::Relaxed) {
        return Err(download_t("download.java.user_cancelled"));
    }

    Ok(java_bin.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::resolve_java_install_app_dir;
    use crate::services::download::common::download_t;
    use crate::test_support::{lock_env, EnvGuard};

    #[test]
    fn resolve_java_install_app_dir_surfaces_app_data_dir_creation_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let blocked_root = temp_dir.path().join("blocked-root");
        std::fs::write(&blocked_root, b"not a directory")
            .expect("file-backed app data root should exist");
        let blocked_path = blocked_root.join("nested");
        let _env_lock = lock_env();
        let _guard = EnvGuard::set("SEALANTERN_DATA_DIR", &blocked_path.to_string_lossy());

        let error = match resolve_java_install_app_dir() {
            Err(error) => error,
            Ok(path) => panic!(
                "app data dir failure should not be silently downgraded, got path: {}",
                path.display()
            ),
        };

        assert!(
            error.contains(&download_t("download.java.app_data_dir_resolve_failed_prefix")),
            "unexpected error: {}",
            error
        );
        assert!(error.contains("blocked-root"), "unexpected error: {}", error);
    }
}
