use crate::commands::app::common::{app_t, app_t1};
use std::path::Path;
use tauri_plugin_dialog::DialogExt;

pub async fn pick_jar_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_select_server_jar"))
        .add_filter(app_t("app.dialog.filter_jar_files"), &["jar"])
        .add_filter(app_t("app.dialog.filter_all_files"), &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_archive_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_select_server_file"))
        .add_filter(
            app_t("app.dialog.filter_server_files"),
            &["jar", "zip", "tar", "tgz", "gz", "exe"],
        )
        .add_filter(app_t("app.dialog.filter_jar_files"), &["jar"])
        .add_filter(app_t("app.dialog.filter_zip_files"), &["zip"])
        .add_filter(app_t("app.dialog.filter_tar_files"), &["tar"])
        .add_filter(app_t("app.dialog.filter_compressed_tar"), &["tgz", "gz"])
        .add_filter(app_t("app.dialog.filter_server_executables"), &["exe"])
        .add_filter(app_t("app.dialog.filter_all_files"), &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_startup_file(
    app: tauri::AppHandle,
    mode: String,
) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mode = mode.to_ascii_lowercase();

    let mut dialog = app.dialog().file();
    match mode.as_str() {
        "bat" => {
            dialog = dialog
                .set_title(app_t("app.dialog.title_select_server_bat"))
                .add_filter(app_t("app.dialog.filter_bat_files"), &["bat"]);
        }
        "sh" => {
            dialog = dialog
                .set_title(app_t("app.dialog.title_select_server_sh"))
                .add_filter(app_t("app.dialog.filter_shell_scripts"), &["sh"]);
        }
        _ => {
            dialog = dialog
                .set_title(app_t("app.dialog.title_select_server_jar"))
                .add_filter(app_t("app.dialog.filter_jar_files"), &["jar"]);
        }
    }

    dialog
        .add_filter(app_t("app.dialog.filter_all_files"), &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_server_executable(
    app: tauri::AppHandle,
) -> Result<Option<(String, String)>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_select_server_executable"))
        .add_filter(app_t("app.dialog.filter_server_files_short"), &["jar", "bat", "sh"])
        .add_filter(app_t("app.dialog.filter_jar_files"), &["jar"])
        .add_filter(app_t("app.dialog.filter_batch_files"), &["bat"])
        .add_filter(app_t("app.dialog.filter_shell_scripts"), &["sh"])
        .add_filter(app_t("app.dialog.filter_all_files"), &["*"])
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

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_select_java_executable"))
        .add_filter(
            if cfg!(windows) {
                app_t("app.dialog.filter_java_executable")
            } else {
                app_t("app.dialog.filter_java_binary")
            },
            if cfg!(windows) { &["exe"] } else { &[""] },
        )
        .add_filter(app_t("app.dialog.filter_all_files"), &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_save_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_save_file"))
        .save_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_personalization_export_file(
    app: tauri::AppHandle,
    suggested_name: String,
) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_export_personalization"))
        .set_file_name(&suggested_name)
        .add_filter(app_t("app.dialog.filter_personalization_package"), &["zip"])
        .save_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_personalization_import_file(
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_import_personalization"))
        .add_filter(app_t("app.dialog.filter_personalization_package"), &["zip"])
        .add_filter(app_t("app.dialog.filter_zip_files"), &["zip"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_select_folder"))
        .pick_folder(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}

pub async fn pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title(app_t("app.dialog.title_select_image"))
        .add_filter(
            app_t("app.dialog.filter_images"),
            &["png", "jpg", "jpeg", "gif", "webp", "bmp"],
        )
        .add_filter(app_t("app.dialog.filter_all_files"), &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv()
        .map_err(|e| app_t1("app.dialog.error", e.to_string()))
}
