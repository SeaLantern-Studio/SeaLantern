use crate::services::global;
use std::io::{self, Write};

pub fn handle_cli() {
    let args: Vec<String> = std::env::args().collect();

    // 如果没有命令行参数，且不是以 --cli 启动，则返回，让 main 继续启动 GUI
    if args.len() <= 1 {
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "--cli" | "cli" => {
            run_interactive_cli();
            std::process::exit(0);
        }
        "list" => {
            list_servers();
            std::process::exit(0);
        }
        "start" => {
            if args.len() > 2 {
                start_server(&args[2]);
            } else {
                println!("用法: start <服务器ID>");
            }
            std::process::exit(0);
        }
        "stop" => {
            if args.len() > 2 {
                stop_server(&args[2]);
            } else {
                println!("用法: stop <服务器ID>");
            }
            std::process::exit(0);
        }
        "help" | "--help" | "-h" => {
            print_help();
            std::process::exit(0);
        }
        _ => {
            // 如果是不认识的参数，可能还是要启动 GUI，或者报错
            // 这里我们选择如果是特定命令则执行，否则让 GUI 启动
            if command.starts_with('-') {
                print_help();
                std::process::exit(0);
            }
        }
    }
}

fn print_help() {
    println!("Sea Lantern - Minecraft Server Manager (CLI Mode)");
    println!("\n用法:");
    println!("  cli / --cli      进入交互式命令行模式");
    println!("  list             列出所有服务器");
    println!("  start <ID>       启动指定服务器");
    println!("  stop <ID>        停止指定服务器");
    println!("  help             显示帮助信息");
}

fn list_servers() {
    let manager = global::server_manager();
    let servers = manager.get_server_list();
    if servers.is_empty() {
        println!("暂无服务器。");
        return;
    }
    println!("{:<36} {:<20} {:<10} {:<10}", "ID", "名称", "版本", "端口");
    println!("{}", "-".repeat(80));
    for s in servers {
        println!("{:<36} {:<20} {:<10} {:<10}", s.id, s.name, s.mc_version, s.port);
    }
}

fn start_server(id: &str) {
    let manager = global::server_manager();
    match manager.start_server(id) {
        Ok(_) => println!("服务器 {} 正在启动...", id),
        Err(e) => println!("启动失败: {}", e),
    }
}

fn stop_server(id: &str) {
    let manager = global::server_manager();
    match manager.stop_server(id) {
        Ok(_) => println!("服务器 {} 已停止。", id),
        Err(e) => println!("停止失败: {}", e),
    }
}

fn run_interactive_cli() {
    println!("欢迎使用 Sea Lantern 交互式命令行模式！输入 'help' 查看命令。");
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "list" => list_servers(),
            "start" => {
                if parts.len() > 1 {
                    start_server(parts[1]);
                } else {
                    println!("用法: start <服务器ID>");
                }
            }
            "stop" => {
                if parts.len() > 1 {
                    stop_server(parts[1]);
                } else {
                    println!("用法: stop <服务器ID>");
                }
            }
            "exit" | "quit" => break,
            "help" => {
                println!("可用命令: list, start <ID>, stop <ID>, help, exit");
            }
            _ => println!("未知命令: {}。输入 'help' 查看帮助。", parts[0]),
        }
    }
}
