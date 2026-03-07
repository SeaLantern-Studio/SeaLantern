//! panic_report.rs
//! 负责在程序崩溃（panic）时收集系统信息并生成崩溃报告。
//!
//! 通过 Rust 标准库的 `std::panic::set_hook` 注册全局 panic 回调，
//! 无需汇编或 unsafe 代码，跨平台兼容。
//! Linux 平台可读取 /proc、/sys 等虚拟文件系统获取更详细的硬件信息；
//! 其他平台则使用通用回退值。
//!
//! 报告输出目录：可执行文件同级的 `panic-log/` 文件夹。
//! 报告文件名格式：`panic_<YYYYMMDD_HHMMSS_mmm>.log`，以崩溃时间戳命名，不会覆盖旧报告。

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 记录程序启动时间，用于在崩溃报告中计算运行时长。
/// 使用 OnceLock 保证只初始化一次，线程安全。
static START_TIME: OnceLock<SystemTime> = OnceLock::new();

/// 防止 panic hook 重入的标志。
/// 若 panic hook 自身触发了新的 panic，此标志可避免无限递归。
static PANIC_HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

/// 注册全局 panic hook。
///
/// 应在程序启动时尽早调用（早于任何可能 panic 的代码），
/// 以确保所有 panic 都能被捕获并生成报告。
///
/// hook 触发时会：
/// 1. 收集崩溃时刻、启动时刻、OS 信息、CPU 温度、内存占用、文件句柄数、CPU 核心数；
/// 2. 获取 panic 发生的源码位置（文件名、行号、列号）及错误消息；
/// 3. 将报告写入可执行文件同级 `panic-log/` 目录下，文件名为 `panic_<时间戳>.log`；
/// 4. 同时将报告输出到 stderr；
/// 5. 以退出码 0xFFFF 终止进程。
pub fn panic_report() {
    // 记录程序启动时间；若已初始化则忽略（幂等）
    START_TIME.get_or_init(SystemTime::now);

    std::panic::set_hook(Box::new(|panic_info| {
        // 使用原子交换防止 hook 重入：若已有一个 hook 正在运行则直接返回
        if PANIC_HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        // 格式化启动时间与崩溃时间
        let start_time = format_time(*START_TIME.get().expect("start time not set"));
        let crash_time = format_time(SystemTime::now());

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

        // 拼装崩溃报告正文
        let report = format!(
            "============!Panicked!============\n\
             ===============Info===============\n\
             Panic Time  : {crash_time}\n\
             Start Time  : {start_time}\n\
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

        // 将报告写入 panic-log/ 目录下以时间戳命名的文件
        // 时间戳取自 UNIX 毫秒数，同时格式化为 YYYYMMDD_HHMMSS_mmm 便于排序与阅读
        let report_path = build_report_path();
        match report_path {
            Ok(path) => {
                if let Err(e) = fs::write(&path, &report) {
                    eprintln!("Failed to write panic report to '{}': {e}", path.display());
                } else {
                    println!("Panic report written to '{}'", path.display());
                }
            }
            Err(e) => {
                // 目录创建失败时降级：直接输出到 stderr，不中断后续流程
                eprintln!("Failed to prepare panic-log directory: {e}");
            }
        }

        // 同时将报告输出到 stderr，方便终端或日志系统捕获
        eprintln!("{report}");
        eprintln!("Sea Lantern PANICKED!!");

        // 重置标志（进程即将退出，保持语义完整性）
        PANIC_HOOK_RUNNING.store(false, Ordering::SeqCst);

        // 以异常退出码终止进程，告知外部监控程序发生了崩溃
        std::process::exit(0xFFFF);
    }));
}

/// 构造崩溃报告的完整输出路径。
///
/// 目标路径：`<项目根目录>/panic-log/panic_<YYYYMMDD_HHMMSS_mmm>.log`
///
/// 基准目录的选取策略（按优先级）：
/// 1. **dev 模式**：Cargo 编译时注入的 `CARGO_MANIFEST_DIR`（即 `src-tauri/`）的父目录，
///    也就是项目根目录，报告会落在仓库根的 `panic-log/` 下；
/// 2. **发布模式**：可执行文件所在目录（安装目录）旁的 `panic-log/`；
/// 3. **兜底**：当前工作目录下的 `panic-log/`。
///
/// - 若 `panic-log/` 目录不存在则自动创建（含所有父目录）；
/// - 返回 `Err` 仅在目录创建失败时出现。
fn build_report_path() -> std::io::Result<PathBuf> {
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

    // 用崩溃时刻的 UNIX 毫秒时间戳构造唯一文件名
    let ts_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // 同时将毫秒时间戳转换为 YYYYMMDD_HHMMSS_mmm 格式，便于人工识别
    let ts_secs = (ts_millis / 1000) as u64;
    let ms_part = (ts_millis % 1000) as u32;
    let ss = ts_secs % 60;
    let mm = (ts_secs / 60) % 60;
    let hh = (ts_secs / 3600) % 24;
    // 计算日期（以 1970-01-01 为基准的简单推算）
    let days_since_epoch = ts_secs / 86400;
    let (year, month, day) = days_to_ymd(days_since_epoch);

    let file_name = format!(
        "panic_{:04}{:02}{:02}_{:02}{:02}{:02}_{:03}.log",
        year, month, day, hh, mm, ss, ms_part
    );

    Ok(log_dir.join(file_name))
}

/// 将从 UNIX epoch 起的天数转换为 (年, 月, 日)。
///
/// 使用 Gregorian 历法推算，无需外部 crate。
fn days_to_ymd(mut days: u64) -> (u32, u32, u32) {
    // 以 400 年为一个循环（146097 天）
    let year_400 = days / 146097;
    days %= 146097;

    let year_100 = (days / 36524).min(3);
    days -= year_100 * 36524;

    let year_4 = days / 1461;
    days %= 1461;

    let year_1 = (days / 365).min(3);
    days -= year_1 * 365;

    let year = (year_400 * 400 + year_100 * 100 + year_4 * 4 + year_1 + 1970) as u32;

    // 判断是否为闰年
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days_in_month = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut month = 1u32;
    let mut remaining = days as u32;
    for dim in &days_in_month {
        if remaining < *dim {
            break;
        }
        remaining -= dim;
        month += 1;
    }

    (year, month, remaining + 1)
}

/// 将 `SystemTime` 格式化为人类可读的字符串。
///
/// 输出格式：`epoch+<天>d HH:MM:SS.mmm (<毫秒数> ms since epoch)`
fn format_time(t: SystemTime) -> String {
    let since_epoch = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since_epoch.as_secs();
    let millis = since_epoch.subsec_millis();

    // 将总秒数拆分为天、时、分、秒
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;

    format!(
        "epoch+{}d {:02}:{:02}:{:02}.{:03} ({} ms since epoch)",
        days,
        h,
        m,
        s,
        millis,
        since_epoch.as_millis()
    )
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
