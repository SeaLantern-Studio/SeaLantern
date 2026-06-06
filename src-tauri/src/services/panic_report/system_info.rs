use sysinfo::{Components, ProcessRefreshKind, ProcessesToUpdate, System};

/// 获取操作系统名称及版本信息
pub(super) fn get_os_info() -> String {
    let name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let version = System::os_version().unwrap_or_default();
    if version.is_empty() {
        name
    } else {
        format!("{name} {version}")
    }
}

/// 获取 CPU 温度
pub(super) fn get_cpu_temperature() -> String {
    let components = Components::new_with_refreshed_list();
    let temp = components
        .iter()
        .find(|component| component.label().to_lowercase().contains("cpu"))
        .or_else(|| components.iter().next())
        .map(|component| component.temperature());

    match temp {
        Some(temp) => format!("{temp:.2} C"),
        None => "N/A".to_string(),
    }
}

/// 获取当前内存占用百分比
pub(super) fn get_memory_load() -> f64 {
    let mut system = System::new();
    system.refresh_memory();

    let total = system.total_memory();
    if total == 0 {
        return 0.0;
    }

    (system.used_memory() as f64 / total as f64) * 100.0
}

/// 获取当前进程打开的句柄数量
pub(super) fn get_handle_count() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(dir) = std::fs::read_dir("/proc/self/fd") {
            return dir.count();
        }
    }

    let pid = match sysinfo::get_current_pid() {
        Ok(pid) => pid,
        Err(_) => return 0,
    };

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::new(),
    );

    system
        .process(pid)
        .and_then(|process| process.tasks())
        .map(|tasks| tasks.len())
        .unwrap_or(0)
}
