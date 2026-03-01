// CLI 命令实现
// 复用 services::global 中的单例服务，通过 clap 子命令路由到对应操作

use crate::services::global;
use crate::services::server_log_pipeline;
use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "sea-lantern")]
#[command(about = "Minecraft Server Manager - CLI Mode", long_about = None)]
#[command(version)]
pub struct Cli {
    /// 输出格式: text, json
    #[arg(short, long, global = true, default_value = "text")]
    format: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有服务器
    List {
        /// 只显示运行中的服务器
        #[arg(short, long)]
        running: bool,
    },
    /// 启动服务器
    Start {
        /// 服务器ID或名称
        id: String,
        /// 等待服务器完全启动后返回
        #[arg(short, long)]
        wait: bool,
    },
    /// 停止服务器
    Stop {
        /// 服务器ID或名称
        id: String,
        /// 强制终止（不等待优雅关闭）
        #[arg(short, long)]
        force: bool,
    },
    /// 重启服务器
    Restart {
        /// 服务器ID或名称
        id: String,
        #[arg(short, long, default_value = "3")]
        delay: u64,
    },
    /// 发送命令到服务器
    Send {
        /// 服务器ID或名称
        id: String,
        /// 要发送的命令
        command: String,
    },
    /// 查看服务器状态
    Status {
        /// 服务器ID或名称
        id: String,
    },
    /// 查看服务器详细信息
    Info {
        /// 服务器ID或名称
        id: String,
    },
    /// 查看服务器日志
    Logs {
        /// 服务器ID或名称
        id: String,
        /// 显示的行数
        #[arg(short, long, default_value = "50")]
        lines: usize,
        /// 只显示错误日志
        #[arg(short, long)]
        errors: bool,
    },
    /// 持续监控服务器日志
    Monitor {
        /// 服务器ID或名称
        id: String,
        /// 显示时间戳
        #[arg(short, long)]
        timestamp: bool,
    },
    /// 重命名服务器
    Rename {
        /// 服务器ID或名称
        id: String,
        /// 新名称
        new_name: String,
    },
    /// 删除服务器
    Delete {
        /// 服务器ID或名称
        id: String,
        /// 跳过确认
        #[arg(short, long)]
        yes: bool,
    },
    /// 停止所有运行中的服务器
    StopAll {
        /// 跳过确认
        #[arg(short, long)]
        yes: bool,
    },
    /// 列出所有运行中的服务器
    Running,
    /// 搜索服务器
    Search {
        /// 搜索关键词
        keyword: String,
    },
    /// 批量执行命令
    Exec {
        /// 目标服务器ID（多个用逗号分隔，或使用 "all" 表示所有运行中的服务器）
        targets: String,
        /// 要执行的命令
        command: String,
    },
    /// 导出服务器列表
    Export {
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 监控服务器状态变化
    Watch {
        /// 刷新间隔（秒）
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },
}

fn resolve_server_id(id_or_name: &str) -> Result<String, String> {
    let manager = global::server_manager();
    let servers = manager.get_server_list();
    
    if servers.iter().any(|s| s.id == id_or_name) {
        return Ok(id_or_name.to_string());
    }
    
    if let Some(server) = servers.iter().find(|s| s.name == id_or_name) {
        return Ok(server.id.clone());
    }
    
    let matches: Vec<_> = servers
        .iter()
        .filter(|s| s.name.to_lowercase().contains(&id_or_name.to_lowercase()))
        .collect();
    
    match matches.len() {
        0 => Err(format!("未找到服务器: {}", id_or_name)),
        1 => Ok(matches[0].id.clone()),
        _ => {
            let names: Vec<&str> = matches.iter().map(|s| s.name.as_str()).collect();
            Err(format!("找到多个匹配的服务器，请使用精确ID或名称: {}", names.join(", ")))
        }
    }
}

fn format_timestamp(timestamp: Option<u64>) -> String {
    match timestamp {
        Some(ts) => {
            let dt = chrono::DateTime::from_timestamp(ts as i64, 0)
                .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH);
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        None => "从未".to_string(),
    }
}

fn format_uptime(started_at: Option<u64>) -> String {
    match started_at {
        Some(ts) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let elapsed = now.saturating_sub(ts);
            let hours = elapsed / 3600;
            let minutes = (elapsed % 3600) / 60;
            let seconds = elapsed % 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }
        None => "-".to_string(),
    }
}

fn print_separator() {
    println!("{}", "─".repeat(60));
}

fn status_to_str(status: &crate::models::server::ServerStatus) -> &'static str {
    match status {
        crate::models::server::ServerStatus::Running => "[运行中]",
        crate::models::server::ServerStatus::Starting => "[启动中]",
        crate::models::server::ServerStatus::Stopping => "[停止中]",
        crate::models::server::ServerStatus::Stopped => "[已停止]",
        crate::models::server::ServerStatus::Error => "[错误]",
    }
}

pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    // 过滤掉 --cli 参数，让 clap 正确解析剩余参数
    let args: Vec<String> = std::env::args()
        .enumerate()
        .filter(|(i, arg)| *i == 0 || arg != "--cli")
        .map(|(_, v)| v)
        .collect();
    
    let cli = Cli::parse_from(args);
    let manager = global::server_manager();

    let result: Result<(), String> = match cli.command {
        Commands::List { running } => {
            let servers = manager.get_server_list();
            let filtered: Vec<_> = if running {
                servers.into_iter().filter(|s| {
                    let status = manager.get_server_status(&s.id);
                    matches!(status.status, crate::models::server::ServerStatus::Running)
                }).collect()
            } else {
                servers
            };
            
            if filtered.is_empty() {
                if running {
                    println!("没有运行中的服务器");
                } else {
                    println!("没有找到服务器");
                }
            } else {
                match cli.format {
                    OutputFormat::Json => {
                        let json = serde_json::to_string_pretty(&filtered)
                            .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
                        println!("{}", json);
                    }
                    OutputFormat::Text => {
                        println!("服务器列表 (共 {} 个):", filtered.len());
                        print_separator();
                        for s in filtered {
                            let status = manager.get_server_status(&s.id);
                            println!("  [{}] {} {}", s.id, s.name, status_to_str(&status.status));
                            println!("      核心: {} | 端口: {} | MC版本: {}", s.core_type, s.port, s.mc_version);
                        }
                        print_separator();
                    }
                }
            }
            Ok(())
        }
        
        Commands::Start { id, wait } => {
            let server_id = resolve_server_id(&id)?;
            manager.start_server(&server_id)?;
            println!("服务器 {} 已启动", id);
            
            if wait {
                println!("等待服务器完全启动...");
                let mut attempts = 0;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let status = manager.get_server_status(&server_id);
                    if matches!(status.status, crate::models::server::ServerStatus::Running) {
                        println!("服务器已就绪");
                        break;
                    }
                    attempts += 1;
                    if attempts > 60 {
                        return Err("等待服务器启动超时".into());
                    }
                }
            }
            Ok(())
        }
        
        Commands::Stop { id, force } => {
            let server_id = resolve_server_id(&id)?;
            if force {
                manager.stop_server(&server_id)?;
                println!("服务器 {} 已强制停止", id);
            } else {
                manager.request_stop_server(&server_id)?;
                println!("服务器 {} 正在停止...", id);
            }
            Ok(())
        }
        
        Commands::Restart { id, delay } => {
            let server_id = resolve_server_id(&id)?;
            println!("正在停止服务器 {}...", id);
            manager.request_stop_server(&server_id)?;
            
            let mut attempts = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let status = manager.get_server_status(&server_id);
                if matches!(status.status, crate::models::server::ServerStatus::Stopped) {
                    break;
                }
                attempts += 1;
                if attempts > 40 {
                    return Err("等待服务器停止超时".into());
                }
            }
            
            println!("等待 {} 秒后重启...", delay);
            std::thread::sleep(std::time::Duration::from_secs(delay));
            
            manager.start_server(&server_id)?;
            println!("服务器 {} 已重启", id);
            Ok(())
        }
        
        Commands::Send { id, command } => {
            let server_id = resolve_server_id(&id)?;
            manager.send_command(&server_id, &command)?;
            println!("命令已发送到服务器 {}: {}", id, command);
            Ok(())
        }
        
        Commands::Status { id } => {
            let server_id = resolve_server_id(&id)?;
            let status = manager.get_server_status(&server_id);
            let servers = manager.get_server_list();
            let server = servers.iter().find(|s| s.id == server_id);
            
            match cli.format {
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&status)
                        .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
                    println!("{}", json);
                }
                OutputFormat::Text => {
                    if let Some(s) = server {
                        println!("服务器状态:");
                        print_separator();
                        println!("  名称: {}", s.name);
                        println!("  ID: {}", s.id);
                        println!("  状态: {}", status_to_str(&status.status));
                        if matches!(status.status, crate::models::server::ServerStatus::Running) {
                            println!("  运行时间: {}", format_uptime(s.last_started_at));
                        }
                        println!("  端口: {}", s.port);
                        print_separator();
                    } else {
                        return Err("服务器信息不存在".into());
                    }
                }
            }
            Ok(())
        }
        
        Commands::Info { id } => {
            let server_id = resolve_server_id(&id)?;
            let servers = manager.get_server_list();
            let server = servers
                .iter()
                .find(|s| s.id == server_id)
                .ok_or_else(|| "服务器不存在".to_string())?;
            
            match cli.format {
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&server)
                        .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e));
                    println!("{}", json);
                }
                OutputFormat::Text => {
                    println!("服务器详细信息:");
                    print_separator();
                    println!("  ID: {}", server.id);
                    println!("  名称: {}", server.name);
                    println!("  核心类型: {}", server.core_type);
                    println!("  核心版本: {}", server.core_version);
                    println!("  MC版本: {}", server.mc_version);
                    println!("  路径: {}", server.path);
                    println!("  启动文件: {}", server.jar_path);
                    println!("  启动模式: {}", server.startup_mode);
                    if let Some(ref cmd) = server.custom_command {
                        println!("  自定义命令: {}", cmd);
                    }
                    println!("  Java路径: {}", server.java_path);
                    println!("  最大内存: {} MB", server.max_memory);
                    println!("  最小内存: {} MB", server.min_memory);
                    println!("  端口: {}", server.port);
                    println!("  创建时间: {}", format_timestamp(Some(server.created_at)));
                    println!("  最后启动: {}", format_timestamp(server.last_started_at));
                    print_separator();
                }
            }
            Ok(())
        }
        
        Commands::Logs { id, lines, errors } => {
            let server_id = resolve_server_id(&id)?;
            let logs = server_log_pipeline::get_logs(&server_id, 0, Some(lines));
            
            if logs.is_empty() {
                println!("暂无日志");
            } else {
                for line in logs {
                    if errors {
                        let lower = line.to_lowercase();
                        if lower.contains("error") || lower.contains("exception") 
                           || lower.contains("warn") || lower.contains("failed") {
                            println!("{}", line);
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
            Ok(())
        }
        
        Commands::Monitor { id, timestamp } => {
            let server_id = resolve_server_id(&id)?;
            println!("监控服务器 {} 日志 (Ctrl+C 退出)...", id);
            print_separator();
            
            let mut offset = 0usize;
            loop {
                let logs = server_log_pipeline::get_logs(&server_id, offset, Some(100));
                for line in &logs {
                    if timestamp {
                        let now = chrono::Local::now();
                        print!("[{}] ", now.format("%H:%M:%S"));
                    }
                    println!("{}", line);
                }
                offset += logs.len();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
        
        Commands::Rename { id, new_name } => {
            let server_id = resolve_server_id(&id)?;
            manager.update_server_name(&server_id, &new_name)?;
            println!("服务器 {} 已重命名为: {}", id, new_name);
            Ok(())
        }
        
        Commands::Delete { id, yes } => {
            let server_id = resolve_server_id(&id)?;
            let servers = manager.get_server_list();
            let server = servers
                .iter()
                .find(|s| s.id == server_id)
                .ok_or_else(|| "服务器不存在".to_string())?;
            
            if !yes {
                println!("即将删除服务器: {} ({})", server.name, server.id);
                println!("路径: {}", server.path);
                print!("确认删除? (y/N): ");
                use std::io::{self, BufRead, Write};
                let stdin = io::stdin();
                let mut input = String::new();
                stdin.lock().read_line(&mut input).ok();
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("已取消");
                    return Ok(());
                }
            }
            
            manager.delete_server(&server_id)?;
            println!("服务器 {} 已删除", server.name);
            Ok(())
        }
        
        Commands::StopAll { yes } => {
            let running_ids = manager.get_running_server_ids();
            if running_ids.is_empty() {
                println!("没有运行中的服务器");
                return Ok(());
            }
            
            if !yes {
                println!("即将停止 {} 个运行中的服务器:", running_ids.len());
                for id in &running_ids {
                    let servers = manager.get_server_list();
                    if let Some(s) = servers.iter().find(|s| &s.id == id) {
                        println!("  - {} ({})", s.name, s.id);
                    }
                }
                print!("确认停止所有? (y/N): ");
                use std::io::{self, BufRead, Write};
                let stdin = io::stdin();
                let mut input = String::new();
                stdin.lock().read_line(&mut input).ok();
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("已取消");
                    return Ok(());
                }
            }
            
            manager.stop_all_servers();
            println!("所有服务器已停止");
            Ok(())
        }
        
        Commands::Running => {
            let running_ids = manager.get_running_server_ids();
            if running_ids.is_empty() {
                println!("没有运行中的服务器");
            } else {
                println!("运行中的服务器 (共 {} 个):", running_ids.len());
                print_separator();
                let servers = manager.get_server_list();
                for id in &running_ids {
                    if let Some(s) = servers.iter().find(|s| &s.id == id) {
                        let uptime = format_uptime(s.last_started_at);
                        println!("  [{}] {} - 运行时间: {} | 端口: {}", s.id, s.name, uptime, s.port);
                    }
                }
                print_separator();
            }
            Ok(())
        }
        
        Commands::Search { keyword } => {
            let servers = manager.get_server_list();
            let matches: Vec<_> = servers
                .iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&keyword.to_lowercase())
                        || s.core_type.to_lowercase().contains(&keyword.to_lowercase())
                        || s.mc_version.to_lowercase().contains(&keyword.to_lowercase())
                })
                .collect();
            
            if matches.is_empty() {
                println!("未找到匹配的服务器: {}", keyword);
            } else {
                println!("搜索结果 (共 {} 个):", matches.len());
                print_separator();
                for s in matches {
                    let status = manager.get_server_status(&s.id);
                    println!("  [{}] {} - {} | {} | 端口: {}", 
                        s.id, s.name, status_to_str(&status.status), s.core_type, s.port);
                }
                print_separator();
            }
            Ok(())
        }
        
        Commands::Exec { targets, command } => {
            let target_ids: Vec<String> = if targets == "all" {
                manager.get_running_server_ids()
            } else {
                targets.split(',')
                    .map(|t| resolve_server_id(t.trim()))
                    .collect::<Result<Vec<_>, _>>()?
            };
            
            if target_ids.is_empty() {
                println!("没有目标服务器");
                return Ok(());
            }
            
            println!("向 {} 个服务器发送命令: {}", target_ids.len(), command);
            let mut success = 0;
            let mut failed = 0;
            
            for id in &target_ids {
                match manager.send_command(id, &command) {
                    Ok(_) => {
                        let servers = manager.get_server_list();
                        if let Some(s) = servers.iter().find(|s| &s.id == id) {
                            println!("  [OK] {} ({})", s.name, id);
                        }
                        success += 1;
                    }
                    Err(e) => {
                        println!("  [FAIL] {} - {}", id, e);
                        failed += 1;
                    }
                }
            }
            
            println!("完成: {} 成功, {} 失败", success, failed);
            Ok(())
        }
        
        Commands::Export { output } => {
            let servers = manager.get_server_list();
            let json = serde_json::to_string_pretty(&servers)
                .map_err(|e| format!("序列化失败: {}", e))?;
            
            match output {
                Some(path) => {
                    std::fs::write(&path, &json)
                        .map_err(|e| format!("写入文件失败: {}", e))?;
                    println!("已导出 {} 个服务器到: {}", servers.len(), path);
                }
                None => {
                    println!("{}", json);
                }
            }
            Ok(())
        }
        
        Commands::Watch { interval } => {
            println!("监控服务器状态 (Ctrl+C 退出)...");
            println!("刷新间隔: {} 秒", interval);
            print_separator();
            
            loop {
                print!("\x1B[2J\x1B[1;1H");
                
                let servers = manager.get_server_list();
                let running_ids = manager.get_running_server_ids();
                
                println!("服务器监控 - {} | 运行中: {} / 总计: {}", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    running_ids.len(), 
                    servers.len()
                );
                print_separator();
                
                for s in &servers {
                    let status = manager.get_server_status(&s.id);
                    let status_str = match status.status {
                        crate::models::server::ServerStatus::Running => {
                            let uptime = format_uptime(s.last_started_at);
                            format!("[运行中] ({})", uptime)
                        }
                        crate::models::server::ServerStatus::Starting => "[启动中]".to_string(),
                        crate::models::server::ServerStatus::Stopping => "[停止中]".to_string(),
                        crate::models::server::ServerStatus::Stopped => "[已停止]".to_string(),
                        crate::models::server::ServerStatus::Error => "[错误]".to_string(),
                    };
                    println!("{:<20} | {} | 端口: {:<5} | {}", s.name, status_str, s.port, s.core_type);
                }
                
                print_separator();
                std::thread::sleep(std::time::Duration::from_secs(interval));
            }
        }
    };

    if let Err(e) = result {
        eprintln!("\x1b[31m错误:\x1b[0m {}", e);
        std::process::exit(1);
    }
    Ok(())
}