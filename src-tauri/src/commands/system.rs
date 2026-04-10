use crate::services;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::{Disks, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tauri_plugin_dialog::DialogExt;

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new_all()));
static SERVER_DISK_USAGE_CACHE: Lazy<Mutex<HashMap<String, CachedDirectorySize>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const PROCESS_CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(200);
const SERVER_DISK_USAGE_CACHE_TTL: Duration = Duration::from_secs(30);

struct CachedDirectorySize {
    used: u64,
    computed_at: Instant,
}

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
pub fn get_server_resource_usage(server_id: String) -> Result<serde_json::Value, String> {
    let manager = services::global::server_manager();
    let server = manager
        .get_server_list()
        .into_iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("未找到服务器: {}", server_id))?;

    let status = manager.get_server_status(&server.id);
    let mut cpu_usage = 0.0_f32;
    let mut memory_used = 0_u64;
    let mut memory_total = 0_u64;
    let mut pid: Option<u32> = None;

    if let Some(raw_pid) = status.pid {
        let process_pid = Pid::from_u32(raw_pid);
        pid = Some(raw_pid);

        {
            let mut sys = SYSTEM.lock().map_err(|e| e.to_string())?;
            sys.refresh_memory();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[process_pid]),
                true,
                ProcessRefreshKind::new().with_cpu().with_memory(),
            );
            memory_total = sys.total_memory();
        }

        std::thread::sleep(PROCESS_CPU_SAMPLE_INTERVAL);

        {
            let mut sys = SYSTEM.lock().map_err(|e| e.to_string())?;
            sys.refresh_memory();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[process_pid]),
                true,
                ProcessRefreshKind::new().with_cpu().with_memory(),
            );

            if let Some(process) = sys.process(process_pid) {
                cpu_usage = process.cpu_usage();
                memory_used = process.memory();
                memory_total = sys.total_memory();
            }
        }
    }

    let disk_path = Path::new(&server.path);
    let disk_used = get_cached_directory_size(disk_path);
    let (disk_total, disk_available) = get_path_disk_capacity(disk_path);
    let disk_total_effective = if disk_total > 0 {
        disk_total
    } else {
        disk_used.max(1)
    };
    let disk_usage = if disk_total_effective > 0 {
        (disk_used as f64 / disk_total_effective as f64 * 100.0) as f32
    } else {
        0.0
    };

    let cpu_clamped = cpu_usage.clamp(0.0, 100.0);
    let memory_usage = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64 * 100.0) as f32
    } else {
        0.0
    };

    Ok(serde_json::json!({
        "server_id": server.id,
        "server_name": server.name,
        "status": status.status.as_str(),
        "pid": pid,
        "cpu": {
            "name": "Server Process",
            "count": 1,
            "usage": cpu_clamped,
        },
        "memory": {
            "total": memory_total,
            "used": memory_used,
            "available": memory_total.saturating_sub(memory_used),
            "usage": memory_usage,
        },
        "disk": {
            "total": disk_total_effective,
            "used": disk_used,
            "available": disk_available,
            "usage": disk_usage.clamp(0.0, 100.0),
            "path": server.path,
        },
    }))
}

fn get_cached_directory_size(path: &Path) -> u64 {
    let cache_key = path.to_string_lossy().into_owned();
    let now = Instant::now();

    if let Ok(cache) = SERVER_DISK_USAGE_CACHE.lock() {
        if let Some(entry) = cache.get(&cache_key) {
            if now.duration_since(entry.computed_at) < SERVER_DISK_USAGE_CACHE_TTL {
                return entry.used;
            }
        }
    }

    let used = calculate_directory_size(path);

    if let Ok(mut cache) = SERVER_DISK_USAGE_CACHE.lock() {
        cache.insert(cache_key, CachedDirectorySize { used, computed_at: now });
    }

    used
}

fn calculate_directory_size(path: &Path) -> u64 {
    fn walk(path: &Path) -> u64 {
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return 0;
        };

        if metadata.is_file() {
            return metadata.len();
        }

        if !metadata.is_dir() {
            return 0;
        }

        let Ok(entries) = std::fs::read_dir(path) else {
            return 0;
        };

        entries
            .filter_map(Result::ok)
            .map(|entry| walk(&entry.path()))
            .sum()
    }

    if !path.exists() {
        return 0;
    }

    walk(path)
}

fn get_path_disk_capacity(path: &Path) -> (u64, u64) {
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let disks = Disks::new_with_refreshed_list();

    let mut best_match: Option<(usize, u64, u64)> = None;

    for disk in disks.iter() {
        let mount_point = disk.mount_point();
        if canonical_path.starts_with(mount_point) {
            let mount_len = mount_point.as_os_str().to_string_lossy().len();
            let candidate = (mount_len, disk.total_space(), disk.available_space());
            match best_match {
                Some((best_len, _, _)) if best_len >= mount_len => {}
                _ => best_match = Some(candidate),
            }
        }
    }

    best_match
        .map(|(_, total, available)| (total, available))
        .unwrap_or((0, 0))
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

#[tauri::command]
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

#[tauri::command]
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

#[tauri::command]
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

#[tauri::command]
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

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    opener::open(&path)
        .map(|_| ())
        .map_err(|e| format!("打开文件失败: {}", e))
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    opener::open(&path)
        .map(|_| ())
        .map_err(|e| format!("打开文件夹失败: {}", e))
}

#[tauri::command]
pub fn get_default_run_path() -> Result<String, String> {
    let base = dirs_next::data_dir()
        .or_else(dirs_next::document_dir)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| "无法确定默认运行路径".to_string())?;

    Ok(base.join("SeaLantern").to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_safe_mode_status() -> Result<bool, String> {
    Ok(std::env::args().any(|arg| arg == "--safe-mode"))
}

#[tauri::command]
pub fn frontend_heartbeat() -> Result<(), String> {
    Ok(())
}
