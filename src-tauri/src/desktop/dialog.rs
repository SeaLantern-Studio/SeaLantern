//! 文件对话框命令。
//!
//! 实现 system 模块的文件选择、保存与文件夹选择接口：
//! `desktop_pick_jar_file` / `desktop_pick_archive_file` / `desktop_pick_startup_file` /
//! `desktop_pick_server_executable` / `desktop_pick_java_file` / `desktop_pick_save_file` /
//! `desktop_pick_folder` / `desktop_pick_image_file` / `desktop_open_folder`。
//!
//! 对话框由 `tauri-plugin-dialog` 提供，采用回调 + 通道转发结果：
//! 选中返回路径字符串，取消返回 `null`。

use std::path::Path;
use std::sync::mpsc;

use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// 打开系统文件选择器选择 JAR 文件。
///
/// 返回选中文件的路径，取消则返回 `null`。
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_jar_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_archive_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_startup_file(
    app: tauri::AppHandle,
    mode: String,
) -> Result<Option<String>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_server_executable(
    app: tauri::AppHandle,
) -> Result<Option<(String, String)>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_save_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
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

/// 在系统文件管理器中打开指定文件夹路径。
///
/// 使用 `tauri-plugin-opener` 打开文件夹，成功返回 `true`，失败返回错误信息。
#[tauri::command(rename_all = "snake_case")]
pub fn desktop_open_folder(app: tauri::AppHandle, path: String) -> Result<bool, String> {
    let path_buf = Path::new(&path);

    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !path_buf.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    // open_path 需要实现 Into<String> 的类型，将 PathBuf 转为字符串
    let path_str = path_buf
        .to_str()
        .ok_or_else(|| "Invalid path: contains non-UTF-8 characters".to_string())?
        .to_string();

    app.opener()
        .open_path(path_str, None::<&str>)
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    Ok(true)
}
