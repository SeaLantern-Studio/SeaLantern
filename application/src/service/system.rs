//! 系统资源信息服务实现。
//!
//! 实现 [`crate::port::SystemService`] 能力端口，组合 `infra` 的
//! 平台系统采集能力（CPU / 内存 / 磁盘 / 网络 / 进程 / 目录占用），
//! 向宿主提供整机快照、进程资源与目录磁盘占用。
//!
//! 错误分层：内部以应用层主错误 [`SystemError`] 为源头，暴露
//! [`SystemService`] 时统一转为接口契约错误 [`SystemServiceError`]。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use sealantern_contract::SystemServiceError;
use sealantern_contract::server::ServerState;
use sealantern_contract::system::{
    CpuInfo, DirectoryUsage, DiskInfo, DiskSummary, MemoryInfo, NetworkInfo, ProcessResourceUsage,
    ServerResourceUsage, SystemSnapshot,
};
use sealantern_infra::platform::{
    collect_cpu_info, collect_disks, collect_networks, collect_process_usage,
    collect_resource_snapshot, collect_system_info, cpu_brand_name, directory_size,
    get_default_run_path, path_disk_capacity, process_count,
};

use crate::error::SystemError;
use crate::port::{InstanceService, ServerService, SystemService};
use crate::service::{CoreInstanceService, CoreServerService};

/// CPU 采样间隔：`sysinfo` 的 CPU 使用率是增量值，需间隔两次采样取后一次。
const CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(200);

/// 基于 `infra` 平台采集能力的系统资源信息服务实现。
pub struct CoreSystemService {
    instance_service: Arc<CoreInstanceService>,
    server_service: Arc<CoreServerService>,
}

impl CoreSystemService {
    /// 构造系统资源信息服务。
    pub fn new(
        instance_service: Arc<CoreInstanceService>,
        server_service: Arc<CoreServerService>,
    ) -> Self {
        Self { instance_service, server_service }
    }
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

    async fn default_run_path(&self) -> Result<String, SystemServiceError> {
        Self::default_run_path_inner().await.map_err(Into::into)
    }

    async fn server_resource_usage(
        &self,
        instance_id: &str,
    ) -> Result<ServerResourceUsage, SystemServiceError> {
        let instance_id = sealantern_core::instance::InstanceId::new(instance_id)
            .map_err(|_| SystemServiceError::PathNotFound)?;

        let instance = self
            .instance_service
            .find(&instance_id)
            .await
            .map_err(|_| SystemServiceError::OperationFailed)?
            .ok_or(SystemServiceError::PathNotFound)?;

        let snapshot = self
            .server_service
            .status(&instance_id)
            .await
            .map_err(|_| SystemServiceError::OperationFailed)?;

        let status = state_to_string(snapshot.state);

        // 未运行或无进程时返回空资源，不报错。
        let usage = match snapshot.pid {
            Some(pid) => Self::process_usage_inner(pid).await?,
            None => ProcessResourceUsage {
                pid: None,
                cpu_usage: 0.0,
                memory_used: 0,
                memory_total: 0,
                memory_usage: 0.0,
            },
        };
        // CPU 品牌与核心数是静态信息，轻量采集即可；不再调用整机快照，避免
        // 每次服务器资源查询都顺带扫一遍整机磁盘/网络/进程。
        let (cpu_name, cpu_count) = tokio::task::spawn_blocking(collect_cpu_info)
            .await
            .map_err(SystemError::from)?;

        // 磁盘占用按实例目录统计，而不是整机分区求和：服务器页面的磁盘指标
        // 应反映该实例实际占用的空间与其所在挂载点容量。
        let directory = instance.directory.clone();
        let disk = tokio::task::spawn_blocking(move || {
            let used = directory_size(&directory);
            let (total, available) = path_disk_capacity(&directory);
            DirectoryUsage {
                path: directory,
                used,
                total,
                available,
                usage: percent(used, total),
            }
        })
        .await
        .map_err(SystemError::from)?;

        Ok(ServerResourceUsage {
            server_id: instance.id.as_str().to_string(),
            server_name: instance.name,
            status,
            pid: snapshot.pid,
            cpu: CpuInfo {
                name: cpu_name,
                count: cpu_count,
                usage: usage.cpu_usage.clamp(0.0, 100.0),
            },
            memory: MemoryInfo {
                total: usage.memory_total,
                used: usage.memory_used,
                available: usage.memory_total.saturating_sub(usage.memory_used),
                usage: usage.memory_usage.clamp(0.0, 100.0),
            },
            disk,
        })
    }
}

/// 将服务器运行状态转为小写字符串（对齐前端 `status` 字段）。
fn state_to_string(state: ServerState) -> String {
    match state {
        ServerState::Starting => "starting".to_string(),
        ServerState::Running => "running".to_string(),
        ServerState::Stopping => "stopping".to_string(),
        ServerState::Stopped => "stopped".to_string(),
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
    use std::sync::Arc;

    use tempfile::tempdir;

    use super::*;

    /// 构造测试用系统服务（临时实例目录 + 独立 server 服务）。
    async fn test_service() -> CoreSystemService {
        let dir = tempdir().expect("create temp dir");
        let instance = Arc::new(
            CoreInstanceService::with_path(dir.path().join("instances.json"))
                .await
                .expect("create instance service"),
        );
        let server = Arc::new(CoreServerService::new(
            instance.clone(),
            Arc::new(crate::service::CoreSettingsService::new()),
        ));
        CoreSystemService::new(instance, server)
    }

    #[tokio::test]
    async fn snapshot_reports_realistic_values() {
        let service = test_service().await;
        let snapshot = service.system_snapshot().await.expect("snapshot");

        assert!(!snapshot.os.is_empty());
        assert!(snapshot.cpu.count > 0);
        assert!(snapshot.cpu.usage >= 0.0 && snapshot.cpu.usage <= 100.0);
        assert!(snapshot.memory.total > 0);
        assert!(snapshot.memory.used <= snapshot.memory.total);
    }

    #[tokio::test]
    async fn default_run_path_resolves_to_sea_lantern_dir() {
        let service = test_service().await;
        let path = service.default_run_path().await.expect("default run path");

        let name = std::path::Path::new(&path)
            .file_name()
            .expect("path should have a file name")
            .to_string_lossy();
        assert_eq!(name, "SeaLantern", "unexpected default run dir: {name}");
    }

    /// 服务器页面的磁盘指标应为实例目录占用，而非整机磁盘汇总。
    #[tokio::test]
    async fn server_resource_usage_reports_directory_disk_usage() {
        use sealantern_core::instance::{InstanceId, LocalLaunch, StartupMode};

        let dir = tempdir().expect("create temp dir");
        let instance_dir = dir.path().join("server");
        std::fs::create_dir_all(&instance_dir).expect("create instance dir");
        // 写入已知大小的文件，验证磁盘占用按目录统计。
        let payload = vec![0x5A; 4096];
        std::fs::write(instance_dir.join("level.dat"), &payload).expect("write payload");

        let instance_service = Arc::new(
            CoreInstanceService::with_path(dir.path().join("instances.json"))
                .await
                .expect("create instance service"),
        );
        let spec = sealantern_core::instance::InstanceSpec {
            id: InstanceId::new("disk-test").expect("valid id"),
            name: "磁盘测试".into(),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory: instance_dir.clone(),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(instance_dir.join("server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        };
        let instance = instance_service
            .create(spec)
            .await
            .expect("create instance");
        let server = Arc::new(CoreServerService::new(
            instance_service.clone(),
            Arc::new(crate::service::CoreSettingsService::new()),
        ));
        let service = CoreSystemService::new(instance_service, server);

        let usage = service
            .server_resource_usage(instance.id.as_str())
            .await
            .expect("server resource usage");

        // 目录占用应反映写入的文件（≈4096 字节），远小于整机磁盘量级。
        assert!(
            usage.disk.used >= payload.len() as u64,
            "disk used too small: {}",
            usage.disk.used
        );
        assert!(
            usage.disk.used < 1024 * 1024,
            "disk used looks like whole-machine usage: {}",
            usage.disk.used
        );
        assert_eq!(usage.disk.path, instance.directory);
    }
}
