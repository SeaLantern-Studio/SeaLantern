mod cli_env;
mod commands;
mod compose_cli;
mod docker_doctor;
mod interactive;
mod runtime;
mod server;
mod server_args;
mod server_control;
mod server_create_entry;
mod server_create_flow;
#[cfg(test)]
mod server_create_flow_tests;
mod server_docker;
mod server_endpoint;
mod server_entry;
mod server_execute;
mod server_feedback;
mod server_feedback_preflight;
mod server_flow;
mod server_help;
mod server_manage;
mod server_manage_logs;
mod server_manage_render;
mod server_manage_start;
mod server_ports;
mod server_ref;
mod server_session;
mod server_setup;
mod server_shared;
#[cfg(test)]
mod server_test_support;
mod server_transport;
#[cfg(test)]
mod server_transport_tests;

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
            if let Err(err) = commands::list_servers() {
                eprintln!("列出服务器失败: {}", err);
            }
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
        "server" => {
            server::handle_server_command(&args[2..]);
            std::process::exit(0);
        }
        "docker" => {
            let exit_code = docker_doctor::handle_docker_command(&args[2..]);
            std::process::exit(exit_code);
        }
        "compose" => {
            let exit_code = compose_cli::handle_compose_command(&args[2..]);
            std::process::exit(exit_code);
        }
        "__local-runtime-helper" => {
            let exit_code =
                crate::services::server::runtime::local_helper::handle_helper_command(&args[2..]);
            std::process::exit(exit_code);
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
    println!("  server [名称] [选项]  创建/接管通用服务器记录，并可拉起 web/cli");
    println!("                   可运行 `sealantern server help` 查看完整参数说明");
    println!("  docker doctor    检查 Docker CLI、守护进程与容器化部署前置条件");
    println!("  docker pull <image[:tag]>  拉取 Docker 镜像，便于后续创建或重试启动");
    println!("  compose generate 为 docker_itzg 服务器导出 compose YAML / 完整容器模板");
    println!("  search-mods <关键词> <版本> [加载器]  搜索模组");
    println!("  join <ID>        通过 ID 加入服务器");
    println!("  create-id <ID> <名称> <地址> [端口]  创建服务器 ID");
    println!("  list-ids         列出所有服务器 ID");
    println!("  resolve-id <ID>  解析服务器 ID 到地址");
    println!("  help             显示帮助信息");
    println!("\nserver 子命令示例:");
    println!("  server fabric-1.20.1 --mc 1.20.1 --core fabric --jar E:/srv/server.jar --java C:/Java/bin/java.exe");
    println!("  server paper-docker --runtime docker --mc 1.21.1 --core paper --image itzg/minecraft-server --image-tag latest --web 8000");
    println!("\n更多说明:");
    println!("  docs/cli-server-runtime-guide.md");
}
