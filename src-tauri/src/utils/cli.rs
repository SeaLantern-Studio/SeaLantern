use crate::services::global;
use std::io::{self, Write};

pub fn handle_cli() {
    let args: Vec<String> = std::env::args().collect();

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
        "search-mods" => {
            if args.len() > 3 {
                search_mods(&args[2], &args[3], args.get(4).unwrap_or(&"Fabric".to_string()));
            } else {
                println!("用法: search-mods <关键词> <游戏版本> [加载器(默认Fabric)]");
            }
            std::process::exit(0);
        }
        "join" => {
            if args.len() > 2 {
                join_server(&args[2]);
            } else {
                println!("用法: join <服务器ID>");
            }
            std::process::exit(0);
        }
        "create-id" => {
            if args.len() > 4 {
                create_server_id(
                    &args[2],
                    &args[3],
                    &args[4],
                    args.get(5).map(|s| s.as_str()).unwrap_or("25565"),
                );
            } else {
                println!("用法: create-id <ID> <名称> <地址> [端口]");
            }
            std::process::exit(0);
        }
        "list-ids" => {
            list_server_ids();
            std::process::exit(0);
        }
        "resolve-id" => {
            if args.len() > 2 {
                resolve_server_id(&args[2]);
            } else {
                println!("用法: resolve-id <服务器ID>");
            }
            std::process::exit(0);
        }
        "help" | "--help" | "-h" => {
            print_help();
            std::process::exit(0);
        }
        _ => {
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
    println!("  search-mods <关键词> <版本> [加载器]  搜索模组");
    println!("  join <ID>        通过 ID 加入服务器");
    println!("  create-id <ID> <名称> <地址> [端口]  创建服务器 ID");
    println!("  list-ids         列出所有服务器 ID");
    println!("  resolve-id <ID>  解析服务器 ID 到地址");
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

fn search_mods(query: &str, version: &str, loader: &str) {
    println!("正在搜索 Modrinth: {} (版本: {}, 加载器: {})...", query, version, loader);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mod_manager = global::mod_manager();
        match mod_manager.search_modrinth(query, version, loader).await {
            Ok(mods) => {
                if mods.is_empty() {
                    println!("未找到匹配的模组。");
                } else {
                    println!("{:<20} {:<15} {:<50}", "名称", "来源", "下载链接");
                    println!("{}", "-".repeat(85));
                    for m in mods {
                        println!("{:<20} {:<15} {:<50}", m.name, m.source, m.download_url);
                    }
                }
            }
            Err(e) => println!("搜索失败: {}", e),
        }
    });
}

fn join_server(id: &str) {
    println!("正在解析服务器 ID: {}...", id);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let join_manager = global::join_manager();
        match join_manager.resolve_id(id).await {
            Ok(addr) => {
                println!("成功解析！服务器地址: {}:{}", addr.host, addr.port);
                println!("正在启动 Minecraft 并连接...");
                println!("请在 Minecraft 中连接到: {}:{}", addr.host, addr.port);
            }
            Err(e) => println!("解析失败: {}", e),
        }
    });
}

fn create_server_id(id: &str, name: &str, address: &str, port: &str) {
    println!("正在创建服务器 ID: {}...", id);
    let port_num: u16 = port.parse().unwrap_or(25565);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let id_manager = global::server_id_manager();
        let req = crate::services::server_id_manager::CreateServerIdRequest {
            id: Some(id.to_string()),
            name: name.to_string(),
            address: address.to_string(),
            port: port_num,
            description: None,
            tags: None,
        };
        match id_manager.create_id(req).await {
            Ok(entry) => {
                println!("成功创建服务器 ID!");
                println!("ID: {}", entry.id);
                println!("名称: {}", entry.name);
                println!("地址: {}:{}", entry.address, entry.port);
            }
            Err(e) => println!("创建失败: {}", e),
        }
    });
}

fn list_server_ids() {
    println!("正在列出所有服务器 ID...");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let id_manager = global::server_id_manager();
        let ids = id_manager.list_ids().await;
        if ids.is_empty() {
            println!("暂无服务器 ID。");
            return;
        }
        println!("{:<20} {:<20} {:<20} {:<10}", "ID", "名称", "地址", "端口");
        println!("{}", "-".repeat(70));
        for entry in ids {
            println!(
                "{:<20} {:<20} {:<20} {:<10}",
                entry.id, entry.name, entry.address, entry.port
            );
        }
    });
}

fn resolve_server_id(id: &str) {
    println!("正在解析服务器 ID: {}...", id);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let id_manager = global::server_id_manager();
        match id_manager.resolve_id(id).await {
            Ok((addr, port)) => {
                println!("成功解析!");
                println!("地址: {}:{}", addr, port);
            }
            Err(e) => println!("解析失败: {}", e),
        }
    });
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
            "create-id" => {
                if parts.len() > 3 {
                    create_server_id(
                        parts[1],
                        parts[2],
                        parts[3],
                        parts.get(4).unwrap_or(&"25565"),
                    );
                } else {
                    println!("用法: create-id <ID> <名称> <地址> [端口]");
                }
            }
            "list-ids" => list_server_ids(),
            "resolve-id" => {
                if parts.len() > 1 {
                    resolve_server_id(parts[1]);
                } else {
                    println!("用法: resolve-id <ID>");
                }
            }
            "exit" | "quit" => break,
            "help" => {
                println!("可用命令: list, start <ID>, stop <ID>, create-id, list-ids, resolve-id, help, exit");
            }
            _ => println!("未知命令: {}。输入 'help' 查看帮助。", parts[0]),
        }
    }
}
