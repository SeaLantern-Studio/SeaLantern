use once_cell::sync::Lazy;
use std::sync::Mutex;
use sysinfo::{Disks, Networks, System};
use tauri_plugin_dialog::DialogExt;

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new_all()));

#[tauri::command]
pub fn get_system_info() -> Result<serde_json::Value, String> {
    let mut sys = SYSTEM.lock().map_err(|e| e.to_string())?;

    sys.refresh_all();

    let cpu_usage = sys.global_cpu_usage();
    let cpu_count = sys.cpus().len();
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();
    let memory_usage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64 * 100.0) as f32
    } else {
        0.0
    };

    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    let swap_usage = if total_swap > 0 {
        (used_swap as f64 / total_swap as f64 * 100.0) as f32
    } else {
        0.0
    };

    let disks = Disks::new_with_refreshed_list();
    let disk_info: Vec<serde_json::Value> = disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage = if total > 0 {
                (used as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            };
            serde_json::json!({
                "name": disk.name().to_string_lossy(),
                "mount_point": disk.mount_point().to_string_lossy(),
                "file_system": disk.file_system().to_string_lossy().to_string(),
                "total": total,
                "used": used,
                "available": available,
                "usage": usage,
                "is_removable": disk.is_removable(),
            })
        })
        .collect();

    let total_disk_space: u64 = disks.iter().map(|d| d.total_space()).sum();
    let total_disk_available: u64 = disks.iter().map(|d| d.available_space()).sum();
    let total_disk_used = total_disk_space.saturating_sub(total_disk_available);
    let total_disk_usage = if total_disk_space > 0 {
        (total_disk_used as f64 / total_disk_space as f64 * 100.0) as f32
    } else {
        0.0
    };

    let networks = Networks::new_with_refreshed_list();
    let network_info: Vec<serde_json::Value> = networks
        .iter()
        .map(|(name, data)| {
            serde_json::json!({
                "name": name,
                "received": data.total_received(),
                "transmitted": data.total_transmitted(),
            })
        })
        .collect();

    let total_received: u64 = networks.values().map(|d| d.total_received()).sum();
    let total_transmitted: u64 = networks.values().map(|d| d.total_transmitted()).sum();

    let uptime = System::uptime();

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let host_name = System::host_name().unwrap_or_else(|| "Unknown".to_string());

    let process_count = sys.processes().len();

    Ok(serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "os_name": os_name,
        "os_version": os_version,
        "kernel_version": kernel_version,
        "host_name": host_name,
        "cpu": {
            "name": cpu_name,
            "count": cpu_count,
            "usage": cpu_usage,
        },
        "memory": {
            "total": total_memory,
            "used": used_memory,
            "available": available_memory,
            "usage": memory_usage,
        },
        "swap": {
            "total": total_swap,
            "used": used_swap,
            "usage": swap_usage,
        },
        "disk": {
            "total": total_disk_space,
            "used": total_disk_used,
            "available": total_disk_available,
            "usage": total_disk_usage,
            "disks": disk_info,
        },
        "network": {
            "total_received": total_received,
            "total_transmitted": total_transmitted,
            "interfaces": network_info,
        },
        "uptime": uptime,
        "process_count": process_count,
    }))
}

#[tauri::command]
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

#[tauri::command]
pub async fn pick_archive_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select modpack archive")
        .add_filter("Archive Files", &["zip", "tar", "tgz", "gz"])
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

#[tauri::command]
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

#[tauri::command]
pub async fn pick_server_executable(
    app: tauri::AppHandle,
) -> Result<Option<(String, String)>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select server executable file")
        .add_filter("Server Files", &["jar", "bat", "sh"])
        .add_filter("JAR Files", &["jar"])
        .add_filter("BAT Files", &["bat"])
        .add_filter("Shell Scripts", &["sh"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| {
                let path_str = p.to_string();
                let ext = std::path::Path::new(&path_str)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_ascii_lowercase())
                    .unwrap_or_default();
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

#[tauri::command]
pub async fn pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog()
        .file()
        .set_title("Select Java executable")
        .add_filter("Executable", &["exe", ""])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.await.map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub async fn pick_save_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .set_title("Save")
        .save_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog()
        .file()
        .set_title("Select folder")
        .pick_folder(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.await.map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub async fn pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog()
        .file()
        .set_title("Select background image")
        .add_filter("Image Files", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.await.map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", path.display()));
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer.exe")
            .arg("/select,")
            .arg(path)
            .spawn()
            .map_err(|e| format!("无法打开文件: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("无法打开文件: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("无法打开文件: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Err(format!("文件夹不存在: {}", path.display()));
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    Ok(())
}
