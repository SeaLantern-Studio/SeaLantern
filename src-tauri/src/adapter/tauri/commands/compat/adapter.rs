//! 纯转换函数：后端模型 ↔ 前端形态。
//!
//! 所有函数无副作用、可单测。命令层只做 `AppServices::get()` 取服务后调用本模块转换。
//! 字段映射规则见各函数注释，对齐 `src/types/server.ts` 与 `src/api/system.ts`。

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec, LocalLaunch, StartupMode};
use sealantern_core::server::{ServerProcessState, ServerStatus};
use sealantern_infra::platform::get_app_data_dir;
use sealantern_interface::system::{
    CpuInfo, DiskInfo, DiskSummary, MemoryInfo, NetworkInfo, ProcessResourceUsage, SystemSnapshot,
};
use sealantern_interface::InstanceServiceError;

use super::models::{
    AddExistingServerParams, CreateServerParams, FrontendCpu, FrontendDisk, FrontendDiskDetail,
    FrontendMemory, FrontendNetwork, FrontendNetworkInterface, FrontendServerInstance,
    FrontendServerResourceUsage, FrontendServerStatusInfo, FrontendSwap, FrontendSystemInfo,
};

// ── Instance ↔ 前端 ──────────────────────────────────────────────────

/// 后端 `Instance` → 前端 `ServerInstance` 形态。
///
/// - `game_version → mc_version`
/// - `directory → path`
/// - `max_memory_mib → max_memory` / `min_memory_mib → min_memory`
/// - `launch.startup_target → jar_path`（None → 空串）
/// - `launch.startup_mode → startup_mode`（用 `as_str()`，与前端 `"jar"/"bat"/...` 一致）
/// - `launch.java_executable → java_path`（None → 空串）
/// - `launch.jvm_arguments → jvm_args`
/// - 丢弃 `aliases / server_metadata / custom_executable / custom_arguments`
pub(crate) fn instance_to_frontend(instance: &Instance) -> FrontendServerInstance {
    FrontendServerInstance {
        id: instance.id.as_str().to_string(),
        name: instance.name.clone(),
        core_type: instance.core_type.clone(),
        core_version: instance.core_version.clone(),
        mc_version: instance.game_version.clone(),
        path: instance.directory.to_string_lossy().to_string(),
        jar_path: instance
            .launch
            .startup_target
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        startup_mode: instance.launch.startup_mode.as_str().to_string(),
        custom_command: instance.launch.custom_command.clone(),
        java_path: instance
            .launch
            .java_executable
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        max_memory: instance.max_memory_mib,
        min_memory: instance.min_memory_mib,
        jvm_args: instance.launch.jvm_arguments.clone(),
        port: instance.port,
        created_at: instance.created_at_unix_secs,
        last_started_at: instance.last_started_at_unix_secs,
    }
}

/// 前端 `create_server` 参数 → 后端 `InstanceSpec`。
///
/// 生成缺失字段：
/// - `id`：UUID v4
/// - `directory`：`{app_data_dir}/servers/{id}`
/// - `created_at_unix_secs`：当前时间
/// - `core_version`：留空（Phase 2 由 provisioning 补）
pub(crate) fn create_params_to_spec(
    params: CreateServerParams,
) -> Result<InstanceSpec, InstanceServiceError> {
    let id_str = uuid::Uuid::new_v4().to_string();
    let id = InstanceId::new(id_str).map_err(|_| InstanceServiceError::InvalidInput)?;
    let directory = get_app_data_dir().join("servers").join(id.as_str());
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let startup_mode = StartupMode::parse(&params.startup_mode)
        .map_err(|_| InstanceServiceError::InvalidInput)?;

    Ok(InstanceSpec {
        id,
        name: params.name,
        aliases: Vec::new(),
        core_type: params.core_type,
        core_version: String::new(),
        game_version: params.mc_version,
        directory,
        port: params.port,
        max_memory_mib: params.max_memory,
        min_memory_mib: params.min_memory,
        created_at_unix_secs: created_at,
        last_started_at_unix_secs: None,
        server_metadata: None,
        launch: LocalLaunch {
            startup_mode,
            startup_target: Some(PathBuf::from(params.jar_path)),
            custom_command: None,
            custom_executable: None,
            custom_arguments: Vec::new(),
            java_executable: if params.java_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(params.java_path))
            },
            jvm_arguments: Vec::new(),
        },
    })
}

/// 前端 `add_existing_server` 参数 → 后端 `InstanceSpec`。
///
/// 与 [`create_params_to_spec`] 类似，但：
/// - `directory = server_path`（已存在服务端）
/// - `startup_target = executable_path`（若提供）
/// - `core_type / core_version / game_version` 留空（Phase 2 由 server_inspection 补）
pub(crate) fn add_existing_params_to_spec(
    params: AddExistingServerParams,
) -> Result<InstanceSpec, InstanceServiceError> {
    let id_str = uuid::Uuid::new_v4().to_string();
    let id = InstanceId::new(id_str).map_err(|_| InstanceServiceError::InvalidInput)?;
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let startup_mode = StartupMode::parse(&params.startup_mode)
        .map_err(|_| InstanceServiceError::InvalidInput)?;

    Ok(InstanceSpec {
        id,
        name: params.name,
        aliases: Vec::new(),
        core_type: String::new(),
        core_version: String::new(),
        game_version: String::new(),
        directory: PathBuf::from(params.server_path),
        port: params.port,
        max_memory_mib: params.max_memory,
        min_memory_mib: params.min_memory,
        created_at_unix_secs: created_at,
        last_started_at_unix_secs: None,
        server_metadata: None,
        launch: LocalLaunch {
            startup_mode,
            startup_target: params.executable_path.map(PathBuf::from),
            custom_command: None,
            custom_executable: None,
            custom_arguments: Vec::new(),
            java_executable: if params.java_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(params.java_path))
            },
            jvm_arguments: Vec::new(),
        },
    })
}

// ── SystemSnapshot ↔ 前端 ────────────────────────────────────────────

/// 后端 `SystemSnapshot` → 前端 `SystemInfo` 形态。
///
/// 关键整形：
/// - `networks: Vec<NetworkInfo>` → `network: { total_received: Σ, total_transmitted: Σ, interfaces }`
/// - `swap: MemoryInfo` → `FrontendSwap`（丢弃 available）
/// - `disk: DiskSummary` → `FrontendDisk`（含 disks 明细）
pub(crate) fn system_snapshot_to_frontend(snapshot: SystemSnapshot) -> FrontendSystemInfo {
    let network = networks_to_frontend(&snapshot.networks);
    let swap = memory_to_swap(&snapshot.swap);
    let disk = disk_summary_to_frontend(&snapshot.disk);
    let cpu = cpu_to_frontend(&snapshot.cpu);
    let memory = memory_to_frontend(&snapshot.memory);

    FrontendSystemInfo {
        os: snapshot.os.to_string(),
        arch: snapshot.arch.to_string(),
        os_name: snapshot.os_name,
        os_version: snapshot.os_version,
        kernel_version: snapshot.kernel_version,
        host_name: snapshot.host_name,
        cpu,
        memory,
        swap,
        disk,
        network,
        uptime: snapshot.uptime,
        process_count: snapshot.process_count,
    }
}

/// `ServerStatus` → 前端 `ServerStatusInfo`。
///
/// - `Running → "Running"`、`Exited(0) → "Stopped"`、`Exited(非0) → "Error"`、`Exited(无码) → "Stopped"`
/// - `pid`：Running 时 `Some(process_id)`，Exited 时 `None`
/// - `uptime`：后端无此信息，恒 `None`
pub(crate) fn server_status_to_frontend(
    id: String,
    status: ServerStatus,
) -> FrontendServerStatusInfo {
    let (status_str, pid) = match status.state {
        ServerProcessState::Running => ("Running".to_string(), Some(status.process_id)),
        ServerProcessState::Exited(exit_status) => {
            let mapped = exit_status
                .code()
                .map(|code| if code == 0 { "Stopped" } else { "Error" })
                .unwrap_or("Stopped");
            (mapped.to_string(), None)
        }
    };

    FrontendServerStatusInfo {
        id,
        status: status_str,
        pid,
        uptime: None,
    }
}

/// `ProcessResourceUsage` → 前端 `ServerResourceUsage`。
///
/// `ProcessResourceUsage` 无 cpu.name/count 与 disk，相关字段留空/零值占位（Phase 2 补）。
pub(crate) fn process_usage_to_resource_usage(
    server_id: String,
    server_name: String,
    status: String,
    usage: ProcessResourceUsage,
) -> FrontendServerResourceUsage {
    let cpu = FrontendCpu {
        name: String::new(),
        count: 0,
        usage: usage.cpu_usage,
    };
    let memory = FrontendMemory {
        total: usage.memory_total,
        used: usage.memory_used,
        available: usage.memory_total.saturating_sub(usage.memory_used),
        usage: usage.memory_usage,
    };
    let disk = FrontendDisk {
        total: 0,
        used: 0,
        available: 0,
        usage: 0.0,
        disks: Vec::new(),
    };

    FrontendServerResourceUsage {
        server_id,
        server_name,
        status,
        pid: usage.pid,
        cpu,
        memory,
        disk,
    }
}

// ── 内部子转换 ───────────────────────────────────────────────────────

fn cpu_to_frontend(cpu: &CpuInfo) -> FrontendCpu {
    FrontendCpu {
        name: cpu.name.clone(),
        count: cpu.count,
        usage: cpu.usage,
    }
}

fn memory_to_frontend(memory: &MemoryInfo) -> FrontendMemory {
    FrontendMemory {
        total: memory.total,
        used: memory.used,
        available: memory.available,
        usage: memory.usage,
    }
}

fn memory_to_swap(swap: &MemoryInfo) -> FrontendSwap {
    FrontendSwap {
        total: swap.total,
        used: swap.used,
        usage: swap.usage,
    }
}

fn disk_summary_to_frontend(disk: &DiskSummary) -> FrontendDisk {
    FrontendDisk {
        total: disk.total,
        used: disk.used,
        available: disk.available,
        usage: disk.usage,
        disks: disk.disks.iter().map(disk_detail_to_frontend).collect(),
    }
}

fn disk_detail_to_frontend(disk: &DiskInfo) -> FrontendDiskDetail {
    FrontendDiskDetail {
        name: disk.name.clone(),
        mount_point: disk.mount_point.to_string_lossy().to_string(),
        file_system: disk.file_system.clone(),
        total: disk.total,
        used: disk.used,
        available: disk.available,
        usage: if disk.total > 0 {
            disk.used as f32 / disk.total as f32 * 100.0
        } else {
            0.0
        },
        is_removable: disk.is_removable,
    }
}

fn networks_to_frontend(networks: &[NetworkInfo]) -> FrontendNetwork {
    let total_received = networks.iter().map(|n| n.received).sum();
    let total_transmitted = networks.iter().map(|n| n.transmitted).sum();
    let interfaces = networks
        .iter()
        .map(|n| FrontendNetworkInterface {
            name: n.interface.clone(),
            received: n.received,
            transmitted: n.transmitted,
        })
        .collect();
    FrontendNetwork {
        total_received,
        total_transmitted,
        interfaces,
    }
}

// ── 单测 ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_instance() -> Instance {
        let spec = InstanceSpec {
            id: InstanceId::new("server-42").expect("valid id"),
            name: "测试服".into(),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory: PathBuf::from("/tmp/server-42"),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 100,
            last_started_at_unix_secs: Some(200),
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("/tmp/server-42/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: Some(PathBuf::from("/usr/bin/java")),
                jvm_arguments: vec!["-Xmx2G".into()],
            },
        };
        Instance::new(spec).expect("valid instance")
    }

    #[test]
    fn instance_to_frontend_maps_all_fields() {
        let instance = sample_instance();
        let frontend = instance_to_frontend(&instance);

        assert_eq!(frontend.id, "server-42");
        assert_eq!(frontend.name, "测试服");
        assert_eq!(frontend.core_type, "paper");
        assert_eq!(frontend.core_version, "1.20.4");
        assert_eq!(frontend.mc_version, "1.20.4");
        assert_eq!(frontend.path, "/tmp/server-42");
        assert_eq!(frontend.jar_path, "/tmp/server-42/server.jar");
        assert_eq!(frontend.startup_mode, "jar");
        assert_eq!(frontend.java_path, "/usr/bin/java");
        assert_eq!(frontend.max_memory, 2048);
        assert_eq!(frontend.min_memory, 512);
        assert_eq!(frontend.jvm_args, vec!["-Xmx2G"]);
        assert_eq!(frontend.port, 25565);
        assert_eq!(frontend.created_at, 100);
        assert_eq!(frontend.last_started_at, Some(200));
    }

    #[test]
    fn instance_to_frontend_handles_missing_optional_paths() {
        let mut spec = InstanceSpec {
            id: InstanceId::new("bare").expect("valid id"),
            name: "裸实例".into(),
            aliases: Vec::new(),
            core_type: String::new(),
            core_version: String::new(),
            game_version: String::new(),
            directory: PathBuf::from("/tmp/bare"),
            port: 25565,
            max_memory_mib: 1024,
            min_memory_mib: 256,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("/tmp/bare/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        };
        let instance = Instance::new(spec.clone()).expect("valid");
        let frontend = instance_to_frontend(&instance);
        assert_eq!(frontend.java_path, "");
        assert_eq!(frontend.last_started_at, None);
        // 复用 spec 避免 move 警告
        spec.id = InstanceId::new("bare").unwrap();
        let _ = spec;
    }

    #[test]
    fn create_params_to_spec_builds_valid_spec() {
        let params = CreateServerParams {
            name: "新服".into(),
            core_type: "paper".into(),
            mc_version: "1.20.4".into(),
            max_memory: 2048,
            min_memory: 512,
            port: 25565,
            java_path: "/usr/bin/java".into(),
            jar_path: "/tmp/server.jar".into(),
            startup_mode: "jar".into(),
        };

        let spec = create_params_to_spec(params).expect("spec should build");
        // UUID 非空
        assert!(!spec.id.as_str().is_empty());
        // 目录以 servers/{id} 结尾
        assert!(spec.directory.ends_with(spec.id.as_str()));
        assert!(spec.directory.starts_with(get_app_data_dir()));
        // created_at > 0（除非系统时钟异常）
        assert!(spec.created_at_unix_secs > 0);
        // core_version 留空
        assert_eq!(spec.core_version, "");
        // launch 字段
        assert_eq!(spec.launch.startup_mode, StartupMode::Jar);
        assert_eq!(
            spec.launch.startup_target.as_ref().unwrap().to_string_lossy(),
            "/tmp/server.jar"
        );

        // Instance::new 必须成功（字段校验通过）
        Instance::new(spec).expect("instance should construct");
    }

    #[test]
    fn create_params_to_spec_rejects_invalid_startup_mode() {
        let params = CreateServerParams {
            name: "x".into(),
            core_type: "paper".into(),
            mc_version: "1.20.4".into(),
            max_memory: 2048,
            min_memory: 512,
            port: 25565,
            java_path: "/usr/bin/java".into(),
            jar_path: "/tmp/server.jar".into(),
            startup_mode: "docker".into(),
        };

        let result = create_params_to_spec(params);
        assert_eq!(result, Err(InstanceServiceError::InvalidInput));
    }

    #[test]
    fn add_existing_params_to_spec_uses_server_path() {
        let params = AddExistingServerParams {
            name: "已存在服".into(),
            server_path: "/existing/server".into(),
            java_path: "/usr/bin/java".into(),
            max_memory: 2048,
            min_memory: 512,
            port: 25565,
            startup_mode: "jar".into(),
            executable_path: Some("/existing/server.jar".into()),
        };

        let spec = add_existing_params_to_spec(params).expect("spec should build");
        assert_eq!(spec.directory, PathBuf::from("/existing/server"));
        assert_eq!(
            spec.launch.startup_target.as_ref().unwrap().to_string_lossy(),
            "/existing/server.jar"
        );
        // 版本字段留空
        assert_eq!(spec.core_type, "");
        assert_eq!(spec.game_version, "");
        assert_eq!(spec.core_version, "");

        Instance::new(spec).expect("instance should construct");
    }

    #[test]
    fn system_snapshot_to_frontend_sums_network_totals() {
        let snapshot = SystemSnapshot {
            os: "linux",
            arch: "x86_64",
            os_name: "Linux".into(),
            os_version: "6.1".into(),
            kernel_version: "6.1.0".into(),
            host_name: "host".into(),
            cpu: CpuInfo {
                name: "CPU".into(),
                count: 8,
                usage: 12.5,
            },
            memory: MemoryInfo {
                total: 100,
                used: 50,
                available: 50,
                usage: 50.0,
            },
            swap: MemoryInfo {
                total: 200,
                used: 100,
                available: 100,
                usage: 50.0,
            },
            disk: DiskSummary {
                total: 1000,
                used: 500,
                available: 500,
                usage: 50.0,
                disks: Vec::new(),
            },
            networks: vec![
                NetworkInfo {
                    interface: "eth0".into(),
                    received: 100,
                    transmitted: 200,
                },
                NetworkInfo {
                    interface: "eth1".into(),
                    received: 300,
                    transmitted: 400,
                },
            ],
            uptime: 999,
            process_count: 42,
        };

        let frontend = system_snapshot_to_frontend(snapshot);

        assert_eq!(frontend.network.total_received, 400);
        assert_eq!(frontend.network.total_transmitted, 600);
        assert_eq!(frontend.network.interfaces.len(), 2);
        assert_eq!(frontend.network.interfaces[0].name, "eth0");
        assert_eq!(frontend.network.interfaces[1].name, "eth1");
        // swap 无 available 字段
        assert_eq!(frontend.swap.total, 200);
        assert_eq!(frontend.swap.used, 100);
        // cpu/memory 直传
        assert_eq!(frontend.cpu.count, 8);
        assert_eq!(frontend.memory.available, 50);
        assert_eq!(frontend.process_count, 42);
        assert_eq!(frontend.uptime, 999);
    }

    #[test]
    fn disk_detail_usage_is_computed_when_total_nonzero() {
        let disk = DiskInfo {
            name: "sda".into(),
            mount_point: PathBuf::from("/"),
            file_system: "ext4".into(),
            total: 1000,
            used: 250,
            available: 750,
            is_removable: false,
        };
        let frontend = disk_detail_to_frontend(&disk);
        assert!((frontend.usage - 25.0).abs() < 0.01);
    }

    #[test]
    fn disk_detail_usage_is_zero_when_total_zero() {
        let disk = DiskInfo {
            name: "empty".into(),
            mount_point: PathBuf::from("/mnt"),
            file_system: "tmpfs".into(),
            total: 0,
            used: 0,
            available: 0,
            is_removable: true,
        };
        let frontend = disk_detail_to_frontend(&disk);
        assert_eq!(frontend.usage, 0.0);
        assert!(frontend.is_removable);
    }
}
