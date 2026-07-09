use std::path::PathBuf;

use tauri::{AppHandle, Emitter};

use sea_lantern_update_core::types::DownloadProgress;

/// 下载更新文件
pub async fn download_update_file(
    app: AppHandle,
    url: String,
    expected_hash: Option<String>,
    cache_dir: PathBuf,
) -> Result<String, String> {
    sea_lantern_update_core::download::download_update_file(
        url,
        expected_hash,
        cache_dir,
        |progress| {
            emit_progress(&app, progress);
        },
    )
    .await
}

fn emit_progress(app: &AppHandle, progress: DownloadProgress) {
    let _ = app.emit("update-download-progress", progress);
}
