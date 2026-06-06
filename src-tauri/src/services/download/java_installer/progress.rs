use tauri::{Emitter, Window};

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    state: String,
    progress: u64,
    total: u64,
    message: String,
}

fn emit_progress_event<R: tauri::Runtime>(window: &Window<R>, payload: DownloadProgress) {
    let _ = window.emit("java-install-progress", payload);
}

/// 发送下载开始事件
pub(super) fn emit_downloading_started<R: tauri::Runtime>(window: &Window<R>) {
    emit_progress_event(
        window,
        DownloadProgress {
            state: "downloading".to_string(),
            progress: 0,
            total: 0,
            message: "开始下载...".to_string(),
        },
    );
}

/// 发送下载进度事件
pub(super) fn emit_progress<R: tauri::Runtime>(
    window: &Window<R>,
    progress: u64,
    total: u64,
    message: String,
) {
    emit_progress_event(
        window,
        DownloadProgress {
            state: "downloading".to_string(),
            progress,
            total,
            message,
        },
    );
}

/// 发送下载完成事件
pub(super) fn emit_download_finished<R: tauri::Runtime>(
    window: &Window<R>,
    progress: u64,
    total: u64,
) {
    emit_progress_event(
        window,
        DownloadProgress {
            state: "downloading".to_string(),
            progress,
            total,
            message: "下载完成，准备解压...".to_string(),
        },
    );
}

/// 发送解压开始事件
pub(super) fn emit_extracting_started<R: tauri::Runtime>(window: &Window<R>) {
    emit_progress_event(
        window,
        DownloadProgress {
            state: "extracting".to_string(),
            progress: 0,
            total: 100,
            message: "正在解压...".to_string(),
        },
    );
}

/// 发送安装完成事件
pub(super) fn emit_finished<R: tauri::Runtime>(window: &Window<R>) {
    emit_progress_event(
        window,
        DownloadProgress {
            state: "finished".to_string(),
            progress: 100,
            total: 100,
            message: "安装完成".to_string(),
        },
    );
}
