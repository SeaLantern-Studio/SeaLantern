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
