//! panic_report.rs
//! 负责在程序崩溃（panic）时收集系统信息并生成崩溃日志。
//!
//! 通过 Rust 标准库的 `std::panic::set_hook` 注册全局 panic 回调，
//! 无需汇编或 unsafe 代码，跨平台兼容。
//! Linux 平台可读取 /proc、/sys 等虚拟文件系统获取更详细的硬件信息；
//! 其他平台则使用通用回退值。
//!
//! 报告输出目录：项目根目录（dev 模式）或可执行文件同级的 `panic-log/` 文件夹。
//! 报告文件名格式：`panic_<YYYYMMDD_HHMMSS_mmm>.log`，以崩溃时间戳命名，不会覆盖旧报告。

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use chrono::Utc;

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

        // 用 chrono 格式化时间，输出为 ISO 8601 / RFC 3339 格式，易于阅读与解析
        let crash_time_str = crash_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();
        let start_time_str = start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();

        // 获取 panic 的完整消息字符串
        let panic_message = panic_info.to_string();

        // 获取 panic 发生的源码位置（文件:行:列），若无则标记为 unknown
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        // 收集系统环境信息
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

/// 获取操作系统信息字符串。
///
/// - Linux：读取 `/proc/version`，包含内核版本及编译信息；
/// - 其他平台：返回 `std::env::consts::OS`（如 "windows"、"macos"）。
fn get_os_info() -> String {
    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/version")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string()
    }
    #[cfg(not(target_os = "linux"))]
    {
        // 非 Linux 平台无法读取 /proc/version，返回平台名称作为回退
        std::env::consts::OS.to_string()
    }
}

/// 获取 CPU 温度（摄氏度）。
///
/// - Linux：遍历 `/sys/class/thermal/thermal_zone*` 读取第一个有效温度值；
///   内核以毫摄氏度为单位存储，需除以 1000 转换；
/// - 其他平台或读取失败：返回 `"N/A"`。
fn get_cpu_temperature() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let path = entry.path();
                // 只处理 thermal_zone* 目录
                if path.to_string_lossy().contains("thermal_zone") {
                    let temp_path = path.join("temp");
                    if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                        if let Ok(millideg) = temp_str.trim().parse::<f64>() {
                            // 内核单位为毫摄氏度，除以 1000 得到摄氏度
                            return format!("{:.2} C", millideg / 1000.0);
                        }
                    }
                }
            }
        }
        "N/A".to_string()
    }
    #[cfg(not(target_os = "linux"))]
    {
        // 非 Linux 平台暂不支持读取 CPU 温度
        "N/A".to_string()
    }
}

/// 获取当前内存占用百分比（0.0 ~ 100.0）。
///
/// - Linux：解析 `/proc/meminfo` 中的 `MemTotal` 与 `MemAvailable` 字段，
///   计算 `(total - available) / total * 100`；
/// - 其他平台或读取失败：返回 `0.0`。
fn get_memory_load() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64;
            let mut available = 0u64;
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    // 格式：MemTotal:    <数值> kB
                    if let Some(val) = line.split_whitespace().nth(1) {
                        total = val.parse().unwrap_or(0);
                    }
                } else if line.starts_with("MemAvailable:") {
                    // 格式：MemAvailable: <数值> kB
                    if let Some(val) = line.split_whitespace().nth(1) {
                        available = val.parse().unwrap_or(0);
                    }
                }
            }
            if total > 0 {
                return (total.saturating_sub(available) as f64 / total as f64) * 100.0;
            }
        }
        0.0
    }
    #[cfg(not(target_os = "linux"))]
    {
        // 非 Linux 平台暂不支持读取内存信息
        0.0
    }
}

/// 获取当前进程打开的文件句柄数量。
///
/// - Linux：统计 `/proc/self/fd` 目录下的条目数，每个条目对应一个打开的文件描述符；
/// - 其他平台：返回 `0`。
fn get_handle_count() -> usize {
    #[cfg(target_os = "linux")]
    {
        fs::read_dir("/proc/self/fd")
            .map(|e| e.count())
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        // 非 Linux 平台暂不支持读取文件句柄数
        0
    }
}
