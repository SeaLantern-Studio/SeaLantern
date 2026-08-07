use std::path::Path;

use sysinfo::{Disks, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// 当前机器的稳定系统信息快照。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SystemInfo {
    pub operating_system: &'static str,
    pub architecture: &'static str,
    pub family: &'static str,
    pub name: Option<String>,
    pub version: Option<String>,
    pub kernel_version: Option<String>,
    pub host_name: Option<String>,
    pub logical_cpu_count: usize,
    pub physical_cpu_count: Option<usize>,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub uptime_seconds: u64,
}

/// 采集当前机器的系统信息。
///
/// `sysinfo` 在无法取得某项操作系统元数据时返回 `None`，因此采集不会因单个
/// 信息缺失而失败。
pub fn collect_system_info() -> SystemInfo {
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();

    SystemInfo {
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        family: std::env::consts::FAMILY,
        name: System::name(),
        version: System::os_version(),
        kernel_version: System::kernel_version(),
        host_name: System::host_name(),
        logical_cpu_count: system.cpus().len(),
        physical_cpu_count: system.physical_core_count(),
        total_memory_bytes: system.total_memory(),
        available_memory_bytes: system.available_memory(),
        uptime_seconds: System::uptime(),
    }
}

/// 单次采样的 CPU / 内存 / 交换用量快照。
///
/// `cpu_usage` 为采样时刻的瞬时值：`sysinfo` 的 CPU 使用率是「上次刷新到本次
/// 刷新」之间的增量，因此需要调用方在两次采样后自行取差值（通常间隔数百毫秒）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceSnapshot {
    /// CPU 使用率（0.0 - 100.0，瞬时采样值）。
    pub cpu_usage: f32,
    /// 物理内存总量（字节）。
    pub total_memory_bytes: u64,
    /// 已用物理内存（字节）。
    pub used_memory_bytes: u64,
    /// 可用物理内存（字节）。
    pub available_memory_bytes: u64,
    /// 交换分区总量（字节）。
    pub total_swap_bytes: u64,
    /// 已用交换分区（字节）。
    pub used_swap_bytes: u64,
}

/// 采集一次 CPU / 内存 / 交换用量快照。
///
/// 调用方如需平滑的 CPU 使用率，应间隔一段时间连续调用两次并取后一次的
/// `cpu_usage`（或自行做差值计算）。
pub fn collect_resource_snapshot() -> ResourceSnapshot {
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_usage();

    ResourceSnapshot {
        cpu_usage: system.global_cpu_usage(),
        total_memory_bytes: system.total_memory(),
        used_memory_bytes: system.used_memory(),
        available_memory_bytes: system.available_memory(),
        total_swap_bytes: system.total_swap(),
        used_swap_bytes: system.used_swap(),
    }
}

/// 单个磁盘分区的用量信息。
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiskUsage {
    /// 磁盘名称（如 `C:`、`/dev/sda1`）。
    pub name: String,
    /// 挂载点路径。
    pub mount_point: std::path::PathBuf,
    /// 文件系统类型（如 `NTFS`、`ext4`）。
    pub file_system: String,
    /// 总容量（字节）。
    pub total_bytes: u64,
    /// 已用容量（字节）。
    pub used_bytes: u64,
    /// 可用容量（字节）。
    pub available_bytes: u64,
    /// 是否为可移动磁盘。
    pub is_removable: bool,
}

/// 采集全部磁盘分区的用量信息。
pub fn collect_disks() -> Vec<DiskUsage> {
    Disks::new_with_refreshed_list()
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            DiskUsage {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_path_buf(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
                total_bytes: total,
                used_bytes: total.saturating_sub(available),
                available_bytes: available,
                is_removable: disk.is_removable(),
            }
        })
        .collect()
}

/// 单个网络接口的累计收发字节数。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkUsage {
    /// 接口名（如 `eth0`、`Wi-Fi`）。
    pub interface: String,
    /// 累计接收字节数。
    pub received_bytes: u64,
    /// 累计发送字节数。
    pub transmitted_bytes: u64,
}

/// 采集全部网络接口的累计收发字节数。
///
/// 返回值为接口上电以来的累计值；如需速率，调用方应间隔采样后做差值。
pub fn collect_networks() -> Vec<NetworkUsage> {
    Networks::new_with_refreshed_list()
        .iter()
        .map(|(interface, data)| NetworkUsage {
            interface: interface.clone(),
            received_bytes: data.total_received(),
            transmitted_bytes: data.total_transmitted(),
        })
        .collect()
}

/// 单个进程的资源使用（瞬时采样值）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessUsage {
    /// 进程 ID。
    pub pid: u32,
    /// CPU 使用率（0.0 - 100.0，瞬时采样值，需二次采样取差值）。
    pub cpu_usage: f32,
    /// 进程占用的物理内存（字节）。
    pub memory_bytes: u64,
}

/// 采集指定进程的瞬时资源使用。
///
/// `cpu_usage` 是「上次刷新到本次刷新」的增量值，单独一次采样意义有限；
/// 调用方应间隔一段时间连续调用两次并取后一次结果（与
/// [`collect_resource_snapshot`] 的 CPU 语义一致）。
///
/// 进程不存在或无权访问时返回 `None`。
pub fn collect_process_usage(pid: u32) -> Option<ProcessUsage> {
    let process_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[process_pid]),
        true,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    system.process(process_pid).map(|process| ProcessUsage {
        pid,
        cpu_usage: process.cpu_usage(),
        memory_bytes: process.memory(),
    })
}

/// 计算目录（含子目录）占用的总字节数。
///
/// 递归遍历目录，累加所有常规文件的长度；符号链接与特殊文件按 0 计。
/// 路径不存在或不可读时返回 0。目录较大时遍历可能较慢，调用方应自行缓存。
pub fn directory_size(path: &Path) -> u64 {
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

/// 返回指定路径所在挂载点的磁盘容量 `(总量, 可用量)`。
///
/// 路径不匹配任何已知挂载点时返回 `(0, 0)`。
pub fn path_disk_capacity(path: &Path) -> (u64, u64) {
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

/// 获取 CPU 型号名称（如 `Intel(R) Core(TM) i7-9700K`）。
///
/// 无法获取时返回 `"Unknown"`。
pub fn cpu_brand_name() -> String {
    let mut system = System::new();
    system.refresh_cpu_all();
    system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".into())
}

/// 获取当前运行的进程数。
pub fn process_count() -> usize {
    System::new_all().processes().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_compile_target_and_runtime_memory() {
        let info = collect_system_info();

        assert_eq!(info.operating_system, std::env::consts::OS);
        assert_eq!(info.architecture, std::env::consts::ARCH);
        assert!(!info.family.is_empty());
        assert!(info.logical_cpu_count > 0);
        assert!(info.total_memory_bytes >= info.available_memory_bytes);
    }

    #[test]
    fn resource_snapshot_reports_usable_memory_values() {
        let snapshot = collect_resource_snapshot();

        assert!(snapshot.cpu_usage >= 0.0 && snapshot.cpu_usage <= 100.0);
        assert!(snapshot.total_memory_bytes > 0);
        assert!(snapshot.used_memory_bytes <= snapshot.total_memory_bytes);
        assert!(snapshot.available_memory_bytes <= snapshot.total_memory_bytes);
        assert!(snapshot.used_swap_bytes <= snapshot.total_swap_bytes);
    }

    #[test]
    fn disks_and_networks_are_serde_serializable() {
        let disks = collect_disks();
        let networks = collect_networks();

        // 结构与 serde 序列化兼容性验证：能序列化即满足契约层要求。
        for disk in &disks {
            serde_json::to_string(disk).expect("disk must serialize");
        }
        for network in &networks {
            serde_json::to_string(network).expect("network must serialize");
        }
    }

    #[test]
    fn current_process_usage_is_collectable() {
        let pid = std::process::id();
        let usage = collect_process_usage(pid);

        assert!(usage.is_some());
        let usage = usage.expect("current process must exist");
        assert_eq!(usage.pid, pid);
        assert!(usage.cpu_usage >= 0.0 && usage.cpu_usage <= 100.0);
        assert!(usage.memory_bytes > 0);
    }

    #[test]
    fn directory_size_counts_files_recursively() {
        let dir = std::env::temp_dir().join(format!("sl-dirsize-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).expect("create temp dir");
        std::fs::write(dir.join("a.txt"), vec![0u8; 10]).expect("write a");
        std::fs::write(dir.join("sub/b.txt"), vec![0u8; 20]).expect("write b");

        assert_eq!(directory_size(&dir), 30);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_path_has_zero_size_and_capacity() {
        let missing = Path::new("/nonexistent/sl-path");
        assert_eq!(directory_size(missing), 0);
        assert_eq!(path_disk_capacity(missing), (0, 0));
    }
}
