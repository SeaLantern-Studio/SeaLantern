//! 服务器管理命令入口
//!
//! 这里按建服导入、启动扫描、文件操作和运行时控制拆到子模块，对外保留 Tauri 命令入口

mod common;
mod file_ops;
mod provisioning;
mod runtime;
mod startup_scan;

/// 强制停止前给前端展示的确认信息
pub use common::ForceStopPreparationResponse;

#[tauri::command]
/// 准备强制停止服务器
pub fn prepare_force_stop_server(
    id: String,
) -> Result<ForceStopPreparationResponse, String> {
    runtime::prepare_force_stop_server(id)
}

#[tauri::command]
/// 执行强制停止服务器
pub fn force_stop_server(id: String, confirmation_token: String) -> Result<(), String> {
    runtime::force_stop_server(id, confirmation_token)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
/// 新建服务器实例
pub fn create_server(
    name: String,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
    startup_mode: String,
) -> Result<crate::models::server::ServerInstance, String> {
    provisioning::create_server(
        name,
        core_type,
        mc_version,
        max_memory,
        min_memory,
        port,
        java_path,
        jar_path,
        startup_mode,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
/// 导入现有服务端核心
pub fn import_server(
    name: String,
    jar_path: String,
    startup_mode: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    online_mode: bool,
) -> Result<crate::models::server::ServerInstance, String> {
    provisioning::import_server(
        name,
        jar_path,
        startup_mode,
        java_path,
        max_memory,
        min_memory,
        port,
        online_mode,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
/// 接入已有服务器目录
pub fn add_existing_server(
    name: String,
    server_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    executable_path: Option<String>,
    custom_command: Option<String>,
) -> Result<crate::models::server::ServerInstance, String> {
    provisioning::add_existing_server(
        name,
        server_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        executable_path,
        custom_command,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
/// 导入整合包并创建服务器
pub fn import_modpack(
    name: String,
    modpack_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    online_mode: bool,
    custom_command: Option<String>,
    run_path: String,
    startup_file_path: Option<String>,
    core_type: Option<String>,
    mc_version: Option<String>,
) -> Result<crate::models::server::ServerInstance, String> {
    provisioning::import_modpack(
        name,
        modpack_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        online_mode,
        custom_command,
        run_path,
        startup_file_path,
        core_type,
        mc_version,
    )
}

#[tauri::command]
/// 解析服务端核心类型
pub async fn parse_server_core_type(
    source_path: String,
) -> Result<crate::models::server::ParsedServerCoreInfo, String> {
    provisioning::parse_server_core_type(source_path).await
}

#[tauri::command]
/// 扫描启动候选项
pub async fn scan_startup_candidates(
    source_path: String,
    source_type: String,
) -> Result<crate::models::server::StartupScanResult, String> {
    startup_scan::scan_startup_candidates(source_path, source_type).await
}

#[tauri::command]
pub fn collect_copy_conflicts(
    source_dir: String,
    target_dir: String,
) -> Result<Vec<String>, String> {
    file_ops::collect_copy_conflicts(source_dir, target_dir)
}

#[tauri::command]
pub fn copy_directory_contents(source_dir: String, target_dir: String) -> Result<(), String> {
    file_ops::copy_directory_contents(source_dir, target_dir)
}

#[tauri::command]
pub fn start_server(app: tauri::AppHandle, id: String) -> Result<(), String> {
    runtime::start_server(app, id)
}

#[tauri::command]
pub fn stop_server(id: String) -> Result<(), String> {
    runtime::stop_server(id)
}

#[tauri::command]
pub fn send_command(id: String, command: String) -> Result<(), String> {
    runtime::send_command(id, command)
}

#[tauri::command]
pub fn get_server_list() -> Vec<crate::models::server::ServerInstance> {
    runtime::get_server_list()
}

#[tauri::command]
pub fn get_server_status(app: tauri::AppHandle, id: String) -> crate::models::server::ServerStatusInfo {
    runtime::get_server_status(app, id)
}

#[tauri::command]
pub fn delete_server(id: String) -> Result<(), String> {
    runtime::delete_server(id)
}

#[tauri::command]
pub fn get_server_logs(id: String, since: usize, max_lines: Option<usize>) -> Vec<String> {
    runtime::get_server_logs(id, since, max_lines)
}

#[tauri::command]
pub fn update_server_name(id: String, name: String) -> Result<(), String> {
    runtime::update_server_name(id, name)
}

#[tauri::command]
pub fn validate_server_path(
    new_path: String,
) -> Result<crate::models::server::ValidateServerPathResult, String> {
    file_ops::validate_server_path(new_path)
}

#[tauri::command]
pub fn update_server_path(
    id: String,
    new_path: String,
    new_jar_path: Option<String>,
    new_startup_mode: Option<String>,
) -> Result<crate::models::server::ServerInstance, String> {
    runtime::update_server_path(id, new_path, new_jar_path, new_startup_mode)
}
