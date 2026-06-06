use std::path::Path;
use tauri_plugin_dialog::DialogExt;

pub async fn pick_jar_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select server JAR file")
        .add_filter("JAR Files", &["jar"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

pub async fn pick_archive_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

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
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
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
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

pub async fn pick_server_executable(
    app: tauri::AppHandle,
) -> Result<Option<(String, String)>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

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

pub async fn pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select Java executable")
        .add_filter(
            if cfg!(windows) {
                "Java Executable"
            } else {
                "Java Binary"
            },
            if cfg!(windows) { &["exe"] } else { &[""] },
        )
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

pub async fn pick_save_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Save File")
        .save_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select folder")
        .pick_folder(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

pub async fn pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select image")
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}
