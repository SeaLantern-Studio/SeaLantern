use std::io::{self, Write};

use super::commands;
use super::print_help;

pub(super) fn run_interactive_cli() {
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
            "list" => commands::list_servers(),
            "start" => {
                if parts.len() > 1 {
                    commands::start_server(parts[1]);
                } else {
                    println!("用法: start <服务器ID>");
                }
            }
            "stop" => {
                if parts.len() > 1 {
                    commands::stop_server(parts[1]);
                } else {
                    println!("用法: stop <服务器ID>");
                }
            }
            "create-id" => {
                if parts.len() > 3 {
                    commands::create_server_id(
                        parts[1],
                        parts[2],
                        parts[3],
                        parts.get(4).copied().unwrap_or("25565"),
                    );
                } else {
                    println!("用法: create-id <ID> <名称> <地址> [端口]");
                }
            }
            "list-ids" => commands::list_server_ids(),
            "resolve-id" => {
                if parts.len() > 1 {
                    commands::resolve_server_id(parts[1]);
                } else {
                    println!("用法: resolve-id <ID>");
                }
            }
            "exit" | "quit" => break,
            "help" => print_help(),
            _ => println!("未知命令: {}。输入 'help' 查看帮助。", parts[0]),
        }
    }
}
