//! 文件对话框命令。
//!
//! 实现 system 模块的文件选择、保存与文件夹选择接口：
//! `pick_jar_file` / `pick_archive_file` / `pick_startup_file` /
//! `pick_server_executable` / `pick_java_file` / `pick_save_file` /
//! `pick_folder` / `pick_image_file`。
//!
//! 对话框由 `tauri-plugin-dialog` 提供，采用回调 + 通道转发结果：
//! 选中返回路径字符串，取消返回 `null`。

use std::path::Path;
use std::sync::mpsc;

use tauri_plugin_dialog::DialogExt;

/// 打开系统文件选择器选择 JAR 文件。
///
/// 返回选中文件的路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_jar_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select server JAR file")
        .add_filter("JAR Files", &["jar"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件选择器选择压缩包文件（.zip/.tar/.tar.gz/.tgz/.jar）。
///
/// 返回选中文件的路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_archive_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select server file")
        .add_filter("Server Files", &["jar", "zip", "tar", "tgz", "gz"])
        .add_filter("JAR Files", &["jar"])
        .add_filter("ZIP Files", &["zip"])
        .add_filter("TAR Files", &["tar"])
        .add_filter("Compressed TAR", &["tgz", "gz"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件选择器选择启动文件。
///
/// `mode` 决定过滤器：`"jar"` / `"bat"` / `"sh"`，未知模式按 JAR 处理。
/// 返回选中文件的路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_startup_file(app: tauri::AppHandle, mode: String) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();
    let mode = mode.to_ascii_lowercase();

    let mut dialog = app.dialog().file();
    match mode.as_str() {
        "bat" => {
            dialog = dialog
                .set_title("Select server BAT file")
                .add_filter("BAT Files", &["bat"]);
        }
        "sh" => {
            dialog = dialog
                .set_title("Select server SH file")
                .add_filter("Shell Scripts", &["sh"]);
        }
        _ => {
            dialog = dialog
                .set_title("Select server JAR file")
                .add_filter("JAR Files", &["jar"]);
        }
    }

    dialog
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件选择器选择服务端可执行文件，同时返回启动模式。
///
/// 返回 `[路径, 启动模式]`，模式为 `"jar" | "bat" | "sh"`；取消则返回 `null`。
#[tauri::command]
pub fn pick_server_executable(app: tauri::AppHandle) -> Result<Option<(String, String)>, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select server executable")
        .add_filter("Server Files", &["jar", "bat", "sh"])
        .add_filter("JAR Files", &["jar"])
        .add_filter("Batch Files", &["bat"])
        .add_filter("Shell Scripts", &["sh"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| {
                let path_str = p.to_string();
                let ext = Path::new(&path_str)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let mode = match ext.as_str() {
                    "bat" => "bat",
                    "sh" => "sh",
                    _ => "jar",
                };
                (path_str, mode.to_string())
            });
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件选择器选择 Java 可执行文件。
///
/// Windows 上按 `.exe` 过滤，其他平台不设扩展名过滤器（Java 二进制无扩展名）。
/// 返回选中文件的路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();

    #[cfg(target_os = "windows")]
    let dialog = app.dialog().file().add_filter("Java Executable", &["exe"]);
    #[cfg(not(target_os = "windows"))]
    let dialog = app.dialog().file();

    dialog
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件保存对话框。
///
/// 返回保存路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_save_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Save File")
        .save_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件夹选择器。
///
/// 返回选中的文件夹路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select folder")
        .pick_folder(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

/// 打开系统文件选择器选择图片文件。
///
/// 返回选中文件的路径，取消则返回 `null`。
#[tauri::command]
pub fn pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select image")
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}
