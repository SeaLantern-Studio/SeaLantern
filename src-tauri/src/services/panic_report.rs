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

mod pathing;
mod system_info;

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use chrono::Utc;

/// 记录程序启动时间，用于在崩溃日志中展示运行时长
/// 使用 OnceLock 保证只初始化一次，线程安全
static START_TIME: OnceLock<chrono::DateTime<Utc>> = OnceLock::new();

/// 防止 panic hook 重入的标志
/// 若 panic hook 自身触发了新的 panic，此标志可避免无限递归
static PANIC_HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

/// 注册全局 panic hook
///
/// 应在程序启动时尽早调用
/// 以确保所有 panic 都能被捕获并生成日志
pub fn init_panic_hook() {
    START_TIME.get_or_init(Utc::now);

    std::panic::set_hook(Box::new(|panic_info| {
        if PANIC_HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        let crash_time = Utc::now();
        let start_time = *START_TIME.get().unwrap_or(&crash_time);
        let crash_time_str = crash_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();
        let start_time_str = start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();
        let panic_message = panic_info.to_string();
        let location = panic_info
            .location()
            .map(|location| format!("{}:{}:{}", location.file(), location.line(), location.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let os_info = system_info::get_os_info();
        let cpu_temp = system_info::get_cpu_temperature();
        let mem_load = system_info::get_memory_load();
        let handle_count = system_info::get_handle_count();
        let cpu_cores = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1);

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

        match pathing::build_report_path(&crash_time) {
            Ok(path) => {
                if let Err(err) = fs::write(&path, &report) {
                    eprintln!("Failed to write panic log to '{}': {err}", path.display());
                } else {
                    println!("Panic log written to '{}'", path.display());
                }
            }
            Err(err) => {
                eprintln!("Failed to prepare panic-log directory: {err}");
            }
        }

        eprintln!("{report}");
        eprintln!("Sea Lantern PANICKED!!");

        PANIC_HOOK_RUNNING.store(false, Ordering::SeqCst);
        std::process::exit(0xFFFF);
    }));
}
