use super::common::{
    CachedDirectorySize, PROCESS_CPU_SAMPLE_INTERVAL, SERVER_DISK_USAGE_CACHE,
    SERVER_DISK_USAGE_CACHE_TTL, SYSTEM,
};
use crate::services;
use std::path::Path;
use std::time::Instant;
use sysinfo::{Disks, Pid, ProcessRefreshKind, ProcessesToUpdate};

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
    let disk_total_effective = if disk_total > 0 { disk_total } else { disk_used.max(1) };
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
