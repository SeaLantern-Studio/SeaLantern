#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        std::println!("{}", message);
        $crate::utils::logger::capture_stdout(&message);
    }};
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        std::eprintln!("{}", message);
        $crate::utils::logger::capture_stderr(&message);
    }};
}

mod adapters;
mod commands;
mod hardcode_data;
mod models;
pub mod plugins;
mod runtime;
mod services;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// 程序入口。
pub fn run() {
    runtime::run();
}
