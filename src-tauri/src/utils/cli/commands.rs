use crate::services::global;
use crate::services::server::id_manager::CreateServerIdRequest;

use super::runtime::run_async_cli_task;

pub(super) fn list_servers() {
    let manager = global::server_manager();
    let servers = manager.get_server_list();
    if servers.is_empty() {
        println!("暂无服务器。");
        return;
    }

    println!("{:<36} {:<20} {:<10} {:<10}", "ID", "名称", "版本", "端口");
    println!("{}", "-".repeat(80));
    for server in servers {
        println!(
            "{:<36} {:<20} {:<10} {:<10}",
            server.id, server.name, server.mc_version, server.port
        );
    }
}

pub(super) fn start_server(id: &str) {
    let manager = global::server_manager();
    match manager.start_server(id) {
        Ok(report) => {
            println!("服务器 {} 正在启动...", id);
            if let Some(fallback) = report.fallback {
                println!(
                    "已触发启动回退: {} -> {} ({})",
                    fallback.from_mode, fallback.to_mode, fallback.reason
                );
            }
        }
        Err(err) => println!("启动失败: {}", err),
    }
}

pub(super) fn stop_server(id: &str) {
    let manager = global::server_manager();
    match manager.stop_server(id) {
        Ok(_) => println!("服务器 {} 已停止。", id),
        Err(err) => println!("停止失败: {}", err),
    }
}

pub(super) fn search_mods(query: &str, version: &str, loader: &str) {
    println!("正在搜索 Modrinth: {} (版本: {}, 加载器: {})...", query, version, loader);
    run_async_cli_task(async {
        let mod_manager = global::mod_manager();
        match mod_manager.search_modrinth(query, version, loader).await {
            Ok(mods) => {
                if mods.is_empty() {
                    println!("未找到匹配的模组。");
                } else {
                    println!("{:<20} {:<15} {:<50}", "名称", "来源", "下载链接");
                    println!("{}", "-".repeat(85));
                    for item in mods {
                        println!("{:<20} {:<15} {:<50}", item.name, item.source, item.download_url);
                    }
                }
            }
            Err(err) => println!("搜索失败: {}", err),
        }
    });
}

pub(super) fn join_server(id: &str) {
    println!("正在解析服务器 ID: {}...", id);
    run_async_cli_task(async {
        let join_manager = global::join_manager();
        match join_manager.resolve_id(id).await {
            Ok(addr) => {
                println!("成功解析！服务器地址: {}:{}", addr.host, addr.port);
                println!("正在启动 Minecraft 并连接...");
                println!("请在 Minecraft 中连接到: {}:{}", addr.host, addr.port);
            }
            Err(err) => println!("解析失败: {}", err),
        }
    });
}

pub(super) fn create_server_id(id: &str, name: &str, address: &str, port: &str) {
    println!("正在创建服务器 ID: {}...", id);
    let port_num: u16 = port.parse().unwrap_or(25565);

    run_async_cli_task(async {
        let id_manager = global::server_id_manager();
        let req = CreateServerIdRequest {
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
            Err(err) => println!("创建失败: {}", err),
        }
    });
}

pub(super) fn list_server_ids() {
    println!("正在列出所有服务器 ID...");
    run_async_cli_task(async {
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

pub(super) fn resolve_server_id(id: &str) {
    println!("正在解析服务器 ID: {}...", id);
    run_async_cli_task(async {
        let id_manager = global::server_id_manager();
        match id_manager.resolve_id(id).await {
            Ok((addr, port)) => {
                println!("成功解析!");
                println!("地址: {}:{}", addr, port);
            }
            Err(err) => println!("解析失败: {}", err),
        }
    });
}
