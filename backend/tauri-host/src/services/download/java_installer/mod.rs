//! Java 下载与安装

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Window};

/// 下载并安装 Java 运行时
pub async fn download_and_install_java<R: tauri::Runtime>(
    url: String,
    version_name: String,
    window: Window<R>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<String, String> {
    let app_dir = crate::utils::path::get_app_data_dir();
    let java_bin = sea_lantern_java_installer_core::download_and_install_java(
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
        return Err("用户取消安装".to_string());
    }

    Ok(java_bin.to_string_lossy().to_string())
}
