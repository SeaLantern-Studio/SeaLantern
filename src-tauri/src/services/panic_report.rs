//! panic_report.rs
//! 负责在程序崩溃（panic）时收集系统信息并生成崩溃日志。
//!
//! 通过 Rust 标准库的 `std::panic::set_hook` 注册全局 panic 回调，
//! 无需汇编或 unsafe 代码，全平台兼容（Linux / macOS / Windows）。
//! 系统信息（内存、CPU 温度、句柄数）通过已有依赖 `sysinfo` 跨平台获取，
//! 不再依赖 Linux 专有的 /proc、/sys 虚拟文件系统。
//!
//! 日志输出目录：项目根目录（dev 模式）或可执行文件同级的 `panic-log/` 文件夹。
//! 日志文件名格式：`panic_<YYYYMMDD_HHMMSS_mmm>.log`，以崩溃时间戳命名，不会覆盖旧日志。

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use chrono::Utc;
use sysinfo::{Components, ProcessRefreshKind, ProcessesToUpdate, System};

/// 记录程序启动时间，用于在崩溃日志中展示运行时长。
/// 使用 OnceLock 保证只初始化一次，线程安全。
static START_TIME: OnceLock<chrono::DateTime<Utc>> = OnceLock::new();

/// 防止 panic hook 重入的标志。
/// 若 panic hook 自身触发了新的 panic，此标志可避免无限递归。
static PANIC_HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

/// 注册全局 panic hook。
///
/// 应在程序启动时尽早调用（早于任何可能 panic 的代码），
/// 以确保所有 panic 都能被捕获并生成日志。
///
/// hook 触发时会：
/// 1. 收集崩溃时刻、启动时刻、OS 信息、CPU 温度、内存占用、文件句柄数、CPU 核心数；
/// 2. 获取 panic 发生的源码位置（文件名、行号、列号）及错误消息；
/// 3. 将日志写入 `panic-log/` 目录下，文件名为 `panic_<时间戳>.log`；
/// 4. 同时将日志内容输出到 stderr；
/// 5. 以退出码 0xFFFF 终止进程。
pub fn init_panic_hook() {
    // 记录程序启动时间；若已初始化则忽略（幂等）
    START_TIME.get_or_init(Utc::now);

    std::panic::set_hook(Box::new(|panic_info| {
        // 使用原子交换防止 hook 重入：若已有一个 hook 正在运行则直接返回
        if PANIC_HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        let crash_time = Utc::now();
        let start_time = *START_TIME.get().expect("start time not set");

        // 用 chrono 格式化时间，输出为易于阅读与解析的格式
        let crash_time_str = crash_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();
        let start_time_str = start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();

        // 获取 panic 的完整消息字符串
        let panic_message = panic_info.to_string();

        // 获取 panic 发生的源码位置（文件:行:列），若无则标记为 unknown
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        // 收集系统环境信息（全平台，通过 sysinfo 获取）
        let os_info = get_os_info();
        let cpu_temp = get_cpu_temperature();
        let mem_load = get_memory_load();
        let handle_count = get_handle_count();
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // 拼装崩溃日志正文
        let report = format!(
            "============!Panicked!============\n\
             ===============Info===============\n\
             Panic Time  : {crash_time_str}\n\
             Start Time  : {start_time_str}\n\
             OS          : {os_info}\n\
             CPU Temp    : {cpu_temp}\n\
             Loaded Mem  : {mem_load:.2}%\n\
             Handle Count: {handle_count}\n\
             CPU Cores   : {cpu_cores}\n\
             ============Panic Info============\n\
             Location    : {location}\n\
             Message     : {panic_message}\n\
             ============ReportEnds============\n",
        );

        // 将日志写入 panic-log/ 目录下以时间戳命名的文件
        let report_path = build_report_path(&crash_time);
        match report_path {
            Ok(path) => {
                if let Err(e) = fs::write(&path, &report) {
                    eprintln!("Failed to write panic log to '{}': {e}", path.display());
                } else {
                    println!("Panic log written to '{}'", path.display());
                }
            }
            Err(e) => {
                // 目录创建失败时降级：直接输出到 stderr，不中断后续流程
                eprintln!("Failed to prepare panic-log directory: {e}");
            }
        }

        // 同时将日志输出到 stderr，方便终端或日志系统捕获
        eprintln!("{report}");
        eprintln!("Sea Lantern PANICKED!!");

        // 重置标志（进程即将退出，保持语义完整性）
        PANIC_HOOK_RUNNING.store(false, Ordering::SeqCst);

        // 以异常退出码终止进程，告知外部监控程序发生了崩溃
        std::process::exit(0xFFFF);
    }));
}

/// 构造崩溃日志的完整输出路径。
///
/// 目标路径：`<基准目录>/panic-log/panic_<YYYYMMDD_HHMMSS_mmm>.log`
///
/// 基准目录的选取策略（按优先级）：
/// 1. **dev 模式**：Cargo 编译时注入的 `CARGO_MANIFEST_DIR`（即 `src-tauri/`）的父目录，
///    也就是项目根目录，日志会落在仓库根的 `panic-log/` 下；
/// 2. **发布模式**：可执行文件所在目录（安装目录）旁的 `panic-log/`；
/// 3. **兜底**：当前工作目录下的 `panic-log/`。
///
/// - 若 `panic-log/` 目录不存在则自动创建（含所有父目录）；
/// - 返回 `Err` 仅在目录创建失败时出现。
fn build_report_path(now: &chrono::DateTime<Utc>) -> std::io::Result<PathBuf> {
    // dev 模式：CARGO_MANIFEST_DIR 指向 src-tauri/，取其父目录即项目根
    // 发布模式：该环境变量不存在，回退到可执行文件所在目录，再回退到当前工作目录
    let base_dir = option_env!("CARGO_MANIFEST_DIR")
        .and_then(|manifest| PathBuf::from(manifest).parent().map(|p| p.to_path_buf()))
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        })
        .unwrap_or_else(|| PathBuf::from("."));

    let log_dir = base_dir.join("panic-log");

    // 目录不存在时递归创建，已存在则忽略错误
    fs::create_dir_all(&log_dir)?;

    // 使用 chrono 格式化时间戳作为文件名，避免手写日历逻辑
    // 格式：panic_YYYYMMDD_HHMMSS_mmm.log，其中 mmm 为毫秒
    let file_name = now.format("panic_%Y%m%d_%H%M%S_%3f.log").to_string();

    Ok(log_dir.join(file_name))
}

/// 获取操作系统名称及版本信息。
///
/// 使用 `sysinfo::System` 跨平台获取，格式为 `<OS名> <版本号>`，
/// 例如 `"Ubuntu 22.04"` / `"Windows 11"` / `"macOS 14.4"`。
/// 若信息不可用则返回 `"Unknown"`。
fn get_os_info() -> String {
    let name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let version = System::os_version().unwrap_or_default();
    if version.is_empty() {
        name
    } else {
        format!("{name} {version}")
    }
}

/// 获取 CPU 温度（摄氏度）。
///
/// 使用 `sysinfo::Components` 跨平台读取，优先选取标签含 "cpu"（大小写不敏感）的传感器，
/// 若无匹配则取第一个可用传感器。
/// 不支持读取温度的平台（如部分虚拟机）返回 `"N/A"`。
fn get_cpu_temperature() -> String {
    // Components::new_with_refreshed_list 会自动刷新并列出所有温度传感器
    let components = Components::new_with_refreshed_list();

    // 优先选取标签含 "cpu" 的传感器，回退到第一个可用传感器
    let temp = components
        .iter()
        .find(|c| c.label().to_lowercase().contains("cpu"))
        .or_else(|| components.iter().next())
        .map(|c| c.temperature());

    match temp {
        Some(t) => format!("{t:.2} C"),
        None => "N/A".to_string(),
    }
}

/// 获取当前内存占用百分比（0.0 ~ 100.0）。
///
/// 使用 `sysinfo::System` 跨平台读取物理内存的总量与已用量，
/// 计算 `used / total * 100`。
/// 若系统不支持或内存总量为 0，返回 `0.0`。
fn get_memory_load() -> f64 {
    // 用 new() 构造空实例后仅刷新内存，避免不必要的开销
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory();
    if total == 0 {
        return 0.0;
    }
    (sys.used_memory() as f64 / total as f64) * 100.0
}

/// 获取当前进程打开的文件句柄（文件描述符）数量。
///
/// - Linux：通过统计 `/proc/self/fd` 目录条目数获取精确值；
/// - 其他平台：通过 `sysinfo` 刷新进程信息后读取其子任务（线程）数作为近似参考，
///   若均不支持则返回 `0`。
fn get_handle_count() -> usize {
    // Linux 上直接统计 /proc/self/fd，最准确
    #[cfg(target_os = "linux")]
    {
        if let Ok(dir) = std::fs::read_dir("/proc/self/fd") {
            return dir.count();
        }
    }

    // 其他平台：用 sysinfo 读取当前进程的子任务数作为句柄数的近似值
    let pid = match sysinfo::get_current_pid() {
        Ok(p) => p,
        Err(_) => return 0,
    };

    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        // sysinfo 0.32 的 ProcessRefreshKind::new() 为空集，不刷新任何额外字段；
        // 基础进程信息（含 tasks）在 refresh_processes_specifics 调用时默认填充
        ProcessRefreshKind::new(),
    );

    sys.process(pid)
        .and_then(|p| p.tasks())
        .map(|t| t.len())
        .unwrap_or(0)
}
