//! 系统资源信息服务实现。
//!
//! 实现 [`sealantern_interface::SystemService`] 能力端口，组合 `infra` 的
//! 平台系统采集能力（CPU / 内存 / 磁盘 / 网络 / 进程 / 目录占用），
//! 向宿主提供整机快照、进程资源与目录磁盘占用。
//!
//! 错误分层：内部以应用层主错误 [`SystemError`] 为源头，暴露
//! [`SystemService`] 时统一转为接口契约错误 [`SystemServiceError`]。

use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use sealantern_infra::platform::{
    collect_disks, collect_networks, collect_process_usage, collect_resource_snapshot,
    collect_system_info, cpu_brand_name, directory_size, get_default_run_path, path_disk_capacity,
    process_count,
};
use sealantern_interface::system::{
    CpuInfo, DirectoryUsage, DiskInfo, DiskSummary, MemoryInfo, NetworkInfo, ProcessResourceUsage,
    SystemSnapshot,
};
use sealantern_interface::{SystemService, SystemServiceError};

use crate::error::SystemError;

/// CPU 采样间隔：`sysinfo` 的 CPU 使用率是增量值，需间隔两次采样取后一次。
const CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(200);

/// 基于 `infra` 平台采集能力的系统资源信息服务实现。
#[derive(Debug, Default)]
pub struct CoreSystemService;

impl CoreSystemService {
    /// 采集整机资源快照，返回应用层主错误。
    ///
    /// 同步的 sysinfo 采集经 `spawn_blocking` 调度到阻塞线程池，CPU 采样
    /// 间隔等待留在异步侧，避免阻塞运行时的核心线程。
    async fn snapshot_inner() -> Result<SystemSnapshot, SystemError> {
        // 静态系统信息与首次 CPU 采样（同步采集，阻塞线程池执行）。
        let info = tokio::task::spawn_blocking(collect_system_info)
            .await
            .map_err(SystemError::from)?;
        let _ = tokio::task::spawn_blocking(collect_resource_snapshot)
            .await
            .map_err(SystemError::from)?;

        // CPU 间隔两次采样，取后一次的平滑使用率（异步等待，不阻塞 runtime）。
        tokio::time::sleep(CPU_SAMPLE_INTERVAL).await;
        let second = tokio::task::spawn_blocking(collect_resource_snapshot)
            .await
            .map_err(SystemError::from)?;

        let memory = MemoryInfo {
            total: second.total_memory_bytes,
            used: second.used_memory_bytes,
            available: second.available_memory_bytes,
            usage: percent(second.used_memory_bytes, second.total_memory_bytes),
        };
        let swap = MemoryInfo {
            total: second.total_swap_bytes,
            used: second.used_swap_bytes,
            available: second
                .total_swap_bytes
                .saturating_sub(second.used_swap_bytes),
            usage: percent(second.used_swap_bytes, second.total_swap_bytes),
        };

        let (disks, networks, cpu_brand) = tokio::task::spawn_blocking(|| {
            let disks: Vec<DiskInfo> = collect_disks()
                .into_iter()
                .map(|disk| DiskInfo {
                    name: disk.name,
                    mount_point: disk.mount_point,
                    file_system: disk.file_system,
                    total: disk.total_bytes,
                    used: disk.used_bytes,
                    available: disk.available_bytes,
                    is_removable: disk.is_removable,
                })
                .collect();
            let networks: Vec<NetworkInfo> = collect_networks()
                .into_iter()
                .map(|network| NetworkInfo {
                    interface: network.interface,
                    received: network.received_bytes,
                    transmitted: network.transmitted_bytes,
                })
                .collect();
            let cpu_brand = cpu_brand_name();
            (disks, networks, cpu_brand)
        })
        .await
        .map_err(SystemError::from)?;

        let disk_total: u64 = disks.iter().map(|d| d.total).sum();
        let disk_used: u64 = disks.iter().map(|d| d.used).sum();
        let disk_available: u64 = disks.iter().map(|d| d.available).sum();
        let disk = DiskSummary {
            total: disk_total,
            used: disk_used,
            available: disk_available,
            usage: percent(disk_used, disk_total),
            disks,
        };

        let process_count = tokio::task::spawn_blocking(process_count)
            .await
            .map_err(SystemError::from)?;

        Ok(SystemSnapshot {
            os: info.operating_system,
            arch: info.architecture,
            os_name: info.name.unwrap_or_else(|| "Unknown".into()),
            os_version: info.version.unwrap_or_else(|| "Unknown".into()),
            kernel_version: info.kernel_version.unwrap_or_else(|| "Unknown".into()),
            host_name: info.host_name.unwrap_or_else(|| "Unknown".into()),
            cpu: CpuInfo {
                name: cpu_brand,
                count: info.logical_cpu_count,
                usage: second.cpu_usage.clamp(0.0, 100.0),
            },
            memory,
            swap,
            disk,
            networks,
            uptime: info.uptime_seconds,
            process_count,
        })
    }

    /// 采集指定进程资源使用，返回应用层主错误。
    async fn process_usage_inner(pid: u32) -> Result<ProcessResourceUsage, SystemError> {
        // CPU 间隔两次采样，取后一次的平滑使用率。采集为同步 sysinfo 调用，
        // 经 spawn_blocking 调度；间隔等待留在异步侧。
        let _ = tokio::task::spawn_blocking(move || collect_process_usage(pid))
            .await
            .map_err(SystemError::from)?;
        tokio::time::sleep(CPU_SAMPLE_INTERVAL).await;
        let usage = tokio::task::spawn_blocking(move || collect_process_usage(pid))
            .await
            .map_err(SystemError::from)?;

        let Some(usage) = usage else {
            return Ok(ProcessResourceUsage {
                pid: None,
                cpu_usage: 0.0,
                memory_used: 0,
                memory_total: 0,
                memory_usage: 0.0,
            });
        };

        let snapshot = tokio::task::spawn_blocking(collect_resource_snapshot)
            .await
            .map_err(SystemError::from)?;
        let memory_total = snapshot.total_memory_bytes;
        Ok(ProcessResourceUsage {
            pid: Some(usage.pid),
            cpu_usage: usage.cpu_usage.clamp(0.0, 100.0),
            memory_used: usage.memory_bytes,
            memory_total,
            memory_usage: percent(usage.memory_bytes, memory_total),
        })
    }

    /// 计算目录磁盘占用，返回应用层主错误。
    ///
    /// 目录遍历与容量查询是同步且可能耗时的操作，经 `spawn_blocking` 调度到
    /// 阻塞线程池，避免阻塞异步运行时的核心线程。
    async fn directory_usage_inner(path: &Path) -> Result<DirectoryUsage, SystemError> {
        let path_owned = path.to_path_buf();
        let (path, used, total, available) =
            tokio::task::spawn_blocking(move || -> Result<_, SystemError> {
                if !path_owned.exists() {
                    return Err(SystemError::PathNotFound);
                }

                let used = directory_size(&path_owned);
                let (total, available) = path_disk_capacity(&path_owned);
                Ok((path_owned, used, total, available))
            })
            .await??;

        let total_effective = if total > 0 { total } else { used.max(1) };
        Ok(DirectoryUsage {
            path,
            used,
            total: total_effective,
            available,
            usage: percent(used, total_effective),
        })
    }

    /// 解析默认运行路径，返回应用层主错误。
    async fn default_run_path_inner() -> Result<String, SystemError> {
        get_default_run_path()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|error| match error {
                sealantern_infra::platform::PlatformError::ResolveDefaultRunPath { source } => {
                    SystemError::DefaultRunPathUnresolved { source }
                }
                _ => SystemError::Unsupported,
            })
    }
}

#[async_trait]
impl SystemService for CoreSystemService {
    async fn system_snapshot(&self) -> Result<SystemSnapshot, SystemServiceError> {
        Self::snapshot_inner().await.map_err(Into::into)
    }

    async fn process_usage(&self, pid: u32) -> Result<ProcessResourceUsage, SystemServiceError> {
        Self::process_usage_inner(pid).await.map_err(Into::into)
    }

    async fn directory_usage(&self, path: &Path) -> Result<DirectoryUsage, SystemServiceError> {
        Self::directory_usage_inner(path).await.map_err(Into::into)
    }

    async fn default_run_path(&self) -> Result<String, SystemServiceError> {
        Self::default_run_path_inner().await.map_err(Into::into)
    }
}

/// 计算使用率（百分比，0.0 - 100.0）。
fn percent(used: u64, total: u64) -> f32 {
    if total > 0 {
        (used as f64 / total as f64 * 100.0) as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn snapshot_reports_realistic_values() {
        let service = CoreSystemService;
        let snapshot = service.system_snapshot().await.expect("snapshot");

        assert!(!snapshot.os.is_empty());
        assert!(snapshot.cpu.count > 0);
        assert!(snapshot.cpu.usage >= 0.0 && snapshot.cpu.usage <= 100.0);
        assert!(snapshot.memory.total > 0);
        assert!(snapshot.memory.used <= snapshot.memory.total);
    }

    #[tokio::test]
    async fn current_process_usage_is_reported() {
        let service = CoreSystemService;
        let usage = service
            .process_usage(std::process::id())
            .await
            .expect("process usage");

        assert!(usage.pid.is_some());
        assert!(usage.cpu_usage >= 0.0 && usage.cpu_usage <= 100.0);
    }

    #[tokio::test]
    async fn missing_process_returns_none_pid() {
        let service = CoreSystemService;
        let usage = service.process_usage(u32::MAX).await.expect("usage");

        assert_eq!(usage.pid, None);
    }

    #[tokio::test]
    async fn missing_directory_reports_path_not_found() {
        let service = CoreSystemService;
        let result = service
            .directory_usage(Path::new("/nonexistent/sl-path"))
            .await;

        assert_eq!(result, Err(SystemServiceError::PathNotFound));
    }

    #[tokio::test]
    async fn default_run_path_resolves_to_sea_lantern_dir() {
        let service = CoreSystemService;
        let path = service.default_run_path().await.expect("default run path");

        let name = std::path::Path::new(&path)
            .file_name()
            .expect("path should have a file name")
            .to_string_lossy();
        assert_eq!(name, "SeaLantern", "unexpected default run dir: {name}");
    }
}
