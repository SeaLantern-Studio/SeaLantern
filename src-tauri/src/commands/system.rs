use sysinfo::{Disks, Networks, System};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn get_system_info() -> Result<serde_json::Value, String> {
    let mut sys = System::new_all();
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

    // 针对不同平台计算磁盘空间
    let (total_disk_space, total_disk_available): (u64, u64) = {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            // 在 macOS 和 Linux 上，只计算根目录 (/) 的磁盘空间
            if let Some(root_disk) = disks
                .iter()
                .find(|d| d.mount_point().to_string_lossy() == "/")
            {
                (root_disk.total_space(), root_disk.available_space())
            } else {
                // 如果找不到根目录，回退到计算所有磁盘
                (
                    disks.iter().map(|d| d.total_space()).sum(),
                    disks.iter().map(|d| d.available_space()).sum(),
                )
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            // 在其他平台，继续计算所有磁盘空间
            (
                disks.iter().map(|d| d.total_space()).sum(),
                disks.iter().map(|d| d.available_space()).sum(),
            )
        }
    };
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
pub async fn pick_java_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select Java executable")
        .add_filter("Executable", &["exe", ""])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select modpack folder")
        .pick_folder(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub async fn pick_image_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("Select background image")
        .add_filter("Image Files", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let result = path.map(|p| p.to_string());
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("Dialog error: {}", e))
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("explorer")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("Failed to open folder".to_string())
        }
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("Failed to open folder".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = Command::new("xdg-open")
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("Failed to open folder".to_string())
        }
    }
}
