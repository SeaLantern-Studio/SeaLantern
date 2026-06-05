use super::common::SYSTEM;
use std::io::ErrorKind;
use sysinfo::{Disks, Networks, System};

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

pub async fn test_ipv6_connectivity() -> Result<serde_json::Value, String> {
    use tokio::net::TcpStream;
    use tokio::task::JoinSet;
    use tokio::time::Duration;

    let test_targets = [
        ("[2606:4700:4700::1111]:53", "Cloudflare DNS"),
        ("[2001:4860:4860::8888]:53", "Google DNS"),
        ("[2606:4700:4700::64]:443", "Cloudflare HTTPS"),
    ];

    let timeout_duration = Duration::from_secs(3);
    let mut last_error = String::new();
    let mut last_error_kind: Option<ErrorKind> = None;
    let mut tested_targets: Vec<serde_json::Value> = Vec::new();

    let mut tasks = JoinSet::new();

    for (addr, name) in test_targets {
        tasks.spawn(async move {
            let result = tokio::time::timeout(timeout_duration, TcpStream::connect(addr)).await;
            (addr, name, result)
        });
    }

    while let Some(join_result) = tasks.join_next().await {
        let (addr, name, result) =
            join_result.map_err(|error| format!("IPv6 测试任务失败: {}", error))?;

        match result {
            Ok(Ok(_stream)) => {
                tasks.abort_all();
                return Ok(serde_json::json!({
                    "supported": true,
                    "message": format!("IPv6 连接成功（通过 {}）", name),
                    "detail": format!("成功连接到 {} ({})", name, addr)
                }));
            }
            Ok(Err(error)) => {
                tested_targets.push(serde_json::json!({
                    "target": name,
                    "address": addr,
                    "error": format!("{}", error),
                    "kind": format!("{:?}", error.kind())
                }));
                last_error = format!("{}", error);
                last_error_kind = Some(error.kind());
            }
            Err(_) => {
                tested_targets.push(serde_json::json!({
                    "target": name,
                    "address": addr,
                    "error": "连接超时",
                    "kind": "TimedOut"
                }));
                last_error = "连接超时".to_string();
                last_error_kind = Some(ErrorKind::TimedOut);
            }
        }
    }

    let summary = match last_error_kind {
        Some(ErrorKind::AddrNotAvailable) => {
            "系统未分配 IPv6 地址，请检查网络适配器是否启用了 IPv6".to_string()
        }
        Some(ErrorKind::NetworkUnreachable) => {
            "IPv6 网络不可达，可能未启用 IPv6 或 ISP 不支持".to_string()
        }
        Some(ErrorKind::TimedOut) => {
            "连接超时，您的网络可能不支持 IPv6 或防火墙阻止了连接".to_string()
        }
        Some(ErrorKind::ConnectionRefused) => {
            "目标服务器拒绝连接，但 IPv6 网络可能可用".to_string()
        }
        _ => format!("IPv6 连接失败: {}", last_error),
    };

    let error_kind = last_error_kind.map(|kind| format!("{:?}", kind));

    Ok(serde_json::json!({
        "supported": false,
        "message": summary,
        "detail": last_error,
        "error_kind": error_kind,
        "targets": tested_targets
    }))
}
