use std::path::Path;

use crate::commands::server::config::SLStartupConfig;
use crate::models::server::{CreateServerRequest, ServerInstance};

use super::super::common::{current_timestamp_secs, normalize_startup_mode, validate_server_name};
use super::ServerManager;

pub(super) fn create_server(
    manager: &ServerManager,
    req: CreateServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = current_timestamp_secs();
    let jar_path_obj = Path::new(&req.jar_path);
    let server_dir = jar_path_obj
        .parent()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let server = ServerInstance {
        id: id.clone(),
        name: server_name,
        core_type: req.core_type,
        core_version: String::new(),
        mc_version: req.mc_version,
        path: server_dir.clone(),
        jar_path: req.jar_path,
        startup_mode: normalize_startup_mode(&req.startup_mode).to_string(),
        custom_command: req.custom_command,
        java_path: req.java_path,
        max_memory: req.max_memory,
        min_memory: req.min_memory,
        jvm_args: Vec::new(),
        port: req.port,
        created_at: now,
        last_started_at: None,
    };

    // 自动生成 SL.json 启动配置文件
    let sl_config = SLStartupConfig {
        max_memory: Some(req.max_memory),
        min_memory: Some(req.min_memory),
    };
    let sl_path = Path::new(&server_dir).join("SL.json");
    match serde_json::to_string_pretty(&sl_config) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&sl_path, content) {
                eprintln!("[WARN] 无法创建 SL.json 启动配置文件: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[WARN] 无法序列化 SL.json 配置: {}", e);
        }
    }

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}
