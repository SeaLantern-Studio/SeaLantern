// CLI
// 通过 clap 解析命令行参数，复用现有服务层逻辑，实现无 GUI 的服务器管理

mod commands;
pub use commands::run_cli;