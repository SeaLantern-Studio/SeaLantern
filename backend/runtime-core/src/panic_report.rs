use chrono::Utc;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static START_TIME: OnceLock<chrono::DateTime<Utc>> = OnceLock::new();
static PANIC_HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

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
            .map(|location| {
                format!("{}:{}:{}", location.file(), location.line(), location.column())
            })
            .unwrap_or_else(|| "unknown location".to_string());

        let os_info = crate::panic_report_system_info::get_os_info();
        let cpu_temp = crate::panic_report_system_info::get_cpu_temperature();
        let mem_load = crate::panic_report_system_info::get_memory_load();
        let handle_count = crate::panic_report_system_info::get_handle_count();
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

        match crate::panic_report_pathing::build_report_path(&crash_time) {
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
