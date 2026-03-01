// 入口：检测 --cli 参数决定启动 CLI 或 GUI 模式
// clap 解析参数，CLI 模式直接调用服务层，GUI 模式启动 Tauri

#![cfg_attr(all(not(debug_assertions), not(feature = "cli")), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--cli" {
        if let Err(e) = sea_lantern_lib::cli::run_cli() {
            eprintln!("\x1b[31m致命错误:\x1b[0m {}", e);
            std::process::exit(1);
        }
    } else {
        sea_lantern_lib::run();
    }
}
