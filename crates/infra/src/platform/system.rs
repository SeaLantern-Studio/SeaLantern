use sysinfo::System;

/// 当前机器的稳定系统信息快照。
#[derive(Debug, Clone, PartialEq, Eq)]
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
}
