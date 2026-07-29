mod commands;
mod interactive;
mod runtime;

pub fn handle_cli() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() <= 1 {
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "--cli" | "cli" => {
            interactive::run_interactive_cli();
            std::process::exit(0);
        }
        "list" => {
            commands::list_servers();
            std::process::exit(0);
        }
        "start" => {
            if args.len() > 2 {
                commands::start_server(&args[2]);
            } else {
                println!("用法: start <服务器ID>");
            }
            std::process::exit(0);
        }
        "stop" => {
            if args.len() > 2 {
                commands::stop_server(&args[2]);
            } else {
                println!("用法: stop <服务器ID>");
            }
            std::process::exit(0);
        }
        "search-mods" => {
            if args.len() > 3 {
                commands::search_mods(
                    &args[2],
                    &args[3],
                    args.get(4).map(|value| value.as_str()).unwrap_or("Fabric"),
                );
            } else {
                println!("用法: search-mods <关键词> <游戏版本> [加载器(默认Fabric)]");
            }
            std::process::exit(0);
        }
        "join" => {
            if args.len() > 2 {
                commands::join_server(&args[2]);
            } else {
                println!("用法: join <服务器ID>");
            }
            std::process::exit(0);
        }
        "create-id" => {
            if args.len() > 4 {
                commands::create_server_id(
                    &args[2],
                    &args[3],
                    &args[4],
                    args.get(5).map(|value| value.as_str()).unwrap_or("25565"),
                );
            } else {
                println!("用法: create-id <ID> <名称> <地址> [端口]");
            }
            std::process::exit(0);
        }
        "list-ids" => {
            commands::list_server_ids();
            std::process::exit(0);
        }
        "resolve-id" => {
            if args.len() > 2 {
                commands::resolve_server_id(&args[2]);
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

pub(super) fn print_help() {
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
