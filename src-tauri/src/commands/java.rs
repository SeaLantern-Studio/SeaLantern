use crate::services::java_detector;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// Global flag to control cancellation of Java installation
static JAVA_INSTALL_CANCEL_FLAG: Lazy<Mutex<Option<Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(None));

#[tauri::command]
pub async fn detect_java() -> Result<Vec<java_detector::JavaInfo>, String> {
    tauri::async_runtime::spawn_blocking(java_detector::detect_java_installations)
        .await
        .map_err(|e| format!("Java 检测任务失败: {}", e))
}

#[tauri::command]
pub async fn validate_java_path(path: String) -> Result<java_detector::JavaInfo, String> {
    tauri::async_runtime::spawn_blocking(move || java_detector::validate_java(path.as_str()))
        .await
        .map_err(|e| format!("Java 路径验证任务失败: {}", e))?
}

#[tauri::command]
pub async fn install_java<R: tauri::Runtime>(
    _app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
    url: String,
    version_name: String,
) -> Result<String, String> {
    use crate::services::java_installer;

    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Scoping the lock
    {
        let mut lock = JAVA_INSTALL_CANCEL_FLAG
            .lock()
            .map_err(|_| "无法获取锁".to_string())?;
        *lock = Some(cancel_flag.clone());
    }

    let result =
        java_installer::download_and_install_java(url, version_name, window, cancel_flag).await;

    // Clear flag after done
    if let Ok(mut lock) = JAVA_INSTALL_CANCEL_FLAG.lock() {
        *lock = None;
    }

    result
}

#[tauri::command]
pub async fn cancel_java_install() -> Result<(), String> {
    let lock = JAVA_INSTALL_CANCEL_FLAG
        .lock()
        .map_err(|_| "无法获取锁".to_string())?;

    if let Some(flag) = &*lock {
        flag.store(true, Ordering::Relaxed);
    }

    Ok(())
}
