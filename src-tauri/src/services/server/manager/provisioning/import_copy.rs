use std::path::Path;

use crate::models::server::{ImportServerRequest, ServerInstance};

use super::super::common::{current_timestamp_secs, normalize_startup_mode, validate_server_name};
use super::super::fs::copy_dir_recursive;
use super::super::ServerManager;
use super::shared::{create_server_properties_if_missing, read_server_port};
use crate::services::server::installer;

pub(super) fn import_server(
    manager: &ServerManager,
    req: ImportServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let startup_mode = normalize_startup_mode(&req.startup_mode).to_string();
    let source_startup_file = Path::new(&req.jar_path);
    if !source_startup_file.exists() {
        return Err(format!("启动文件不存在: {}", req.jar_path));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let data_dir = manager.data_dir_value()?;
    let servers_dir = Path::new(&data_dir).join("servers");
    let server_dir = servers_dir.join(&id);

    std::fs::create_dir_all(&server_dir).map_err(|e| format!("无法创建服务器目录: {}", e))?;

    let startup_filename = source_startup_file
        .file_name()
        .ok_or_else(|| "无法获取启动文件名".to_string())?;
    let source_server_dir = source_startup_file
        .parent()
        .ok_or_else(|| "无法获取启动文件所在目录".to_string())?;

    println!(
        "导入服务器：复制源目录 {} -> {}",
        source_server_dir.display(),
        server_dir.display()
    );
    copy_dir_recursive(source_server_dir, &server_dir)
        .map_err(|e| format!("复制服务端目录失败: {}", e))?;

    let dest_startup = server_dir.join(startup_filename);
    if !dest_startup.exists() {
        return Err(format!("复制后的启动文件不存在: {}", dest_startup.display()));
    }

    create_server_properties_if_missing(&server_dir, req.port, req.online_mode)?;
    let port = read_server_port(&server_dir, req.port);

    let now = current_timestamp_secs();
    let core_type = installer::detect_core_type(&dest_startup.to_string_lossy());
    println!("检测到核心类型: {}", core_type);

    let server = ServerInstance {
        id: id.clone(),
        name: server_name,
        core_type,
        core_version: String::new(),
        mc_version: "unknown".into(),
        path: server_dir.to_string_lossy().to_string(),
        jar_path: dest_startup.to_string_lossy().to_string(),
        startup_mode,
        custom_command: req.custom_command,
        java_path: req.java_path,
        max_memory: req.max_memory,
        min_memory: req.min_memory,
        jvm_args: Vec::new(),
        port,
        created_at: now,
        last_started_at: None,
    };

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}
