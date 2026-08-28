//! 系统资源信息契约模型。
//!
//! 提供整机资源快照、进程资源占用与目录磁盘占用的宿主能力端口，
//! 供 tauri / server 等宿主统一消费，不依赖任何具体系统采集实现。

mod models;

pub use models::{
    CpuInfo, DirectoryUsage, DiskInfo, DiskSummary, MemoryInfo, NetworkInfo, ProcessResourceUsage,
    ServerResourceUsage, SystemSnapshot,
};
