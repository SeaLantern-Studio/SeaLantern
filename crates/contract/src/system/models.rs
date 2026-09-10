//! 系统资源信息契约模型。
//!
//! 定义宿主消费的系统资源快照、进程资源与目录磁盘占用等模型，
//! 全部可序列化，供跨传输面传递。

use std::path::PathBuf;

use serde::Serialize;

/// CPU 资源信息。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CpuInfo {
    /// CPU 型号名称。
    pub name: String,
    /// 逻辑核心数。
    pub count: usize,
    /// CPU 使用率（0.0 - 100.0，调用方间隔采样后的值）。
    pub usage: f32,
}

/// 内存资源信息。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MemoryInfo {
    /// 总量（字节）。
    pub total: u64,
    /// 已用（字节）。
    pub used: u64,
    /// 可用（字节）。
    pub available: u64,
    /// 使用率（0.0 - 100.0）。
    pub usage: f32,
}

/// 单个磁盘分区信息。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiskInfo {
    /// 磁盘名称。
    pub name: String,
    /// 挂载点。
    pub mount_point: PathBuf,
    /// 文件系统类型。
    pub file_system: String,
    /// 总容量（字节）。
    pub total: u64,
    /// 已用（字节）。
    pub used: u64,
    /// 可用（字节）。
    pub available: u64,
    /// 是否为可移动磁盘。
    pub is_removable: bool,
}

/// 单个网络接口信息。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NetworkInfo {
    /// 接口名。
    pub interface: String,
    /// 累计接收字节。
    pub received: u64,
    /// 累计发送字节。
    pub transmitted: u64,
}

/// 磁盘汇总信息。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiskSummary {
    /// 全部分区总容量（字节）。
    pub total: u64,
    /// 全部分区已用（字节）。
    pub used: u64,
    /// 全部分区可用（字节）。
    pub available: u64,
    /// 整体使用率（0.0 - 100.0）。
    pub usage: f32,
    /// 分区明细。
    pub disks: Vec<DiskInfo>,
}

/// 整机系统资源快照（宿主消费的契约模型）。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SystemSnapshot {
    /// 操作系统（`std::env::consts::OS`，如 `windows` / `linux` / `macos`）。
    pub os: &'static str,
    /// 架构（`std::env::consts::ARCH`，如 `x86_64`）。
    pub arch: &'static str,
    /// 操作系统名称（如 `Windows 11`）。
    pub os_name: String,
    /// 系统版本。
    pub os_version: String,
    /// 内核版本。
    pub kernel_version: String,
    /// 主机名。
    pub host_name: String,
    /// CPU 资源。
    pub cpu: CpuInfo,
    /// 内存资源。
    pub memory: MemoryInfo,
    /// 交换分区。
    pub swap: MemoryInfo,
    /// 磁盘汇总（总量/已用/可用为全部分区求和）。
    pub disk: DiskSummary,
    /// 网络接口列表。
    pub networks: Vec<NetworkInfo>,
    /// 系统运行时长（秒）。
    pub uptime: u64,
    /// 当前进程数。
    pub process_count: usize,
}

/// 单进程资源使用（宿主消费的契约模型）。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessResourceUsage {
    /// 进程 ID；进程不存在时为 `None`。
    pub pid: Option<u32>,
    /// CPU 使用率（0.0 - 100.0，调用方间隔采样后的值）。
    pub cpu_usage: f32,
    /// 进程占用内存（字节）。
    pub memory_used: u64,
    /// 系统内存总量（字节），用于计算使用率。
    pub memory_total: u64,
    /// 内存使用率（0.0 - 100.0）。
    pub memory_usage: f32,
}

/// 目录磁盘占用（宿主消费的契约模型）。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DirectoryUsage {
    /// 目录路径。
    pub path: PathBuf,
    /// 目录占用字节数。
    pub used: u64,
    /// 所在挂载点总容量（字节）。
    pub total: u64,
    /// 所在挂载点可用容量（字节）。
    pub available: u64,
    /// 使用率（0.0 - 100.0）。
    pub usage: f32,
}

/// 单服务器资源占用（宿主消费的契约模型，按实例标识查询）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ServerResourceUsage {
    /// 实例标识。
    pub server_id: String,
    /// 实例名称。
    pub server_name: String,
    /// 运行状态（`ServerState` 的小写字符串，如 `running` / `stopped`）。
    pub status: String,
    /// 进程 ID；未运行时为 `None`。
    pub pid: Option<u32>,
    /// CPU 资源。
    pub cpu: CpuInfo,
    /// 内存资源。
    pub memory: MemoryInfo,
    /// 实例目录磁盘占用（目录本身占用 + 所在挂载点容量）。
    pub disk: DirectoryUsage,
}
