//! 前端形态的请求/响应 DTO。
//!
//! 对齐 `src/types/server.ts` 与 `src/api/system.ts` 的字段名与形状。
//! 请求 DTO 用 `#[serde(rename_all = "camelCase")]` 以吃下前端 camelCase 参数；
//! 响应 DTO 保持前端期待的 snake_case 字段名（前端 TS 接口即 snake_case）。

use serde::{Deserialize, Serialize};

// ── 响应模型（对齐 src/types/server.ts）──────────────────────────────

/// 前端 `ServerInstance` 形态（扁平 snake_case）。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendServerInstance {
    pub id: String,
    pub name: String,
    pub core_type: String,
    pub core_version: String,
    pub mc_version: String,
    pub path: String,
    pub jar_path: String,
    pub startup_mode: String,
    pub custom_command: Option<String>,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub jvm_args: Vec<String>,
    pub port: u16,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
}

/// 前端 `ServerStatusInfo` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendServerStatusInfo {
    pub id: String,
    pub status: String,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

// ── 请求模型（对齐 src/api/server.ts，camelCase 反序列化）────────────

/// `create_server` 请求参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServerParams {
    pub name: String,
    pub core_type: String,
    pub mc_version: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub java_path: String,
    pub jar_path: String,
    /// 前端默认传 `"jar"`。
    pub startup_mode: String,
}

/// `add_existing_server` 请求参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddExistingServerParams {
    pub name: String,
    pub server_path: String,
    pub java_path: String,
    pub max_memory: u32,
    pub min_memory: u32,
    pub port: u16,
    pub startup_mode: String,
    pub executable_path: Option<String>,
}

/// `update_server_path` 请求参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServerPathParams {
    pub id: String,
    pub new_path: String,
    /// 第一阶段忽略，待 Phase 2 扩展 InstanceService。
    pub new_jar_path: Option<String>,
    /// 第一阶段忽略，待 Phase 2 扩展 InstanceService。
    pub new_startup_mode: Option<String>,
}

/// `force_stop_server` 请求参数（token 被丢弃）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceStopServerParams {
    pub id: String,
    pub confirmation_token: String,
}

/// `get_server_resource_usage` 请求参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetServerResourceUsageParams {
    pub server_id: String,
}

// ── 系统资源响应模型（对齐 src/api/system.ts）────────────────────────

/// 前端 `SystemInfo` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendSystemInfo {
    pub os: String,
    pub arch: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub host_name: String,
    pub cpu: FrontendCpu,
    pub memory: FrontendMemory,
    pub swap: FrontendSwap,
    pub disk: FrontendDisk,
    pub network: FrontendNetwork,
    pub uptime: u64,
    pub process_count: usize,
}

/// 前端 `CpuInfo` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendCpu {
    pub name: String,
    pub count: usize,
    pub usage: f32,
}

/// 前端 `MemoryInfo` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendMemory {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage: f32,
}

/// 前端 `SwapInfo` 形态（比 MemoryInfo 少 available）。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendSwap {
    pub total: u64,
    pub used: u64,
    pub usage: f32,
}

/// 前端 `DiskInfo` 形态（汇总 + 分区明细）。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendDisk {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage: f32,
    pub disks: Vec<FrontendDiskDetail>,
}

/// 前端 `DiskDetail` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendDiskDetail {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage: f32,
    pub is_removable: bool,
}

/// 前端 `NetworkInfo` 形态（汇总 + 接口明细）。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendNetwork {
    pub total_received: u64,
    pub total_transmitted: u64,
    pub interfaces: Vec<FrontendNetworkInterface>,
}

/// 前端 `NetworkInterface` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendNetworkInterface {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
}

/// 前端 `ServerResourceUsage` 形态。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendServerResourceUsage {
    pub server_id: String,
    pub server_name: String,
    pub status: String,
    pub pid: Option<u32>,
    pub cpu: FrontendCpu,
    pub memory: FrontendMemory,
    pub disk: FrontendDisk,
}
