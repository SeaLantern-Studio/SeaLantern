use std::path::Path;

use crate::models::server::{AddExistingServerRequest, ServerInstance};

use super::super::common::{
    current_timestamp_secs, detect_startup_mode_from_path, normalize_startup_mode,
    validate_server_name,
};
use super::super::fs::find_server_executable;
use super::super::ServerManager;
use super::shared::{ensure_server_path_writable, read_server_port};
use crate::services::server::installer;

pub(super) fn add_existing_server(
    manager: &ServerManager,
    req: AddExistingServerRequest,
) -> Result<ServerInstance, String> {
    let server_name = validate_server_name(&req.name)?;
    let server_path = Path::new(&req.server_path);
    if !server_path.exists() {
        return Err(format!("服务器目录不存在: {}", req.server_path));
    }
    if !server_path.is_dir() {
        return Err("所选路径不是文件夹".to_string());
    }

    ensure_server_path_writable(server_path)?;

    let requested_mode = normalize_startup_mode(&req.startup_mode).to_string();
    let (jar_path, startup_mode, custom_command) = if requested_mode == "custom" {
        let command = req
            .custom_command
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "自定义启动命令不能为空".to_string())?;
        (String::new(), requested_mode, Some(command))
    } else if let Some(ref exec_path) = req.executable_path {
        let path = Path::new(exec_path);
        if !path.exists() {
            return Err(format!("选择的可执行文件不存在: {}", exec_path));
        }
        (exec_path.clone(), detect_startup_mode_from_path(path), None)
    } else {
        let (path, mode) = find_server_executable(server_path)?;
        (path, mode, None)
    };

    let port = read_server_port(server_path, req.port);
    let core_type = if startup_mode == "custom" {
        "Unknown".to_string()
    } else {
        installer::detect_core_type(&jar_path)
    };
    println!("检测到核心类型: {}", core_type);

    let now = current_timestamp_secs();
    let id = uuid::Uuid::new_v4().to_string();

    let server = ServerInstance {
        id: id.clone(),
        name: server_name,
        core_type,
        core_version: String::new(),
        mc_version: "unknown".into(),
        path: req.server_path,
        jar_path,
        startup_mode,
        custom_command,
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
