//! 服务器管理命令入口
//!
//! 这里按建服导入、启动扫描、文件操作和运行时控制拆到子模块，对外保留 Tauri 命令入口

mod common;
mod file_ops;
mod provisioning;
mod runtime;
mod startup_scan;

use serde::Serialize;

/// 强制停止前给前端展示的确认信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceStopPreparationResponse {
    pub token: String,
    pub expires_at: u64,
}

#[tauri::command]
/// 准备强制停止服务器
pub fn prepare_force_stop_server(id: String) -> Result<ForceStopPreparationResponse, String> {
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
    aliases: Option<Vec<String>>,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
    server_path: Option<String>,
    startup_mode: String,
    custom_command: Option<String>,
    jvm_args: Vec<String>,
    terminal_mode: crate::models::server::LocalTerminalMode,
    cpu_policy: crate::models::server::CpuPolicyConfig,
    jvm_preset: crate::models::server::JvmPresetConfig,
) -> Result<crate::models::server::ServerInstance, String> {
    provisioning::create_server(
        name,
        aliases,
        core_type,
        mc_version,
        max_memory,
        min_memory,
        port,
        java_path,
        jar_path,
        server_path,
        startup_mode,
        custom_command,
        jvm_args,
        terminal_mode,
        cpu_policy,
        jvm_preset,
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
    jvm_args: Vec<String>,
    terminal_mode: crate::models::server::LocalTerminalMode,
    cpu_policy: crate::models::server::CpuPolicyConfig,
    jvm_preset: crate::models::server::JvmPresetConfig,
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
        jvm_args,
        terminal_mode,
        cpu_policy,
        jvm_preset,
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
    core_type: Option<String>,
    mc_version: Option<String>,
    jvm_args: Vec<String>,
    terminal_mode: crate::models::server::LocalTerminalMode,
    cpu_policy: crate::models::server::CpuPolicyConfig,
    jvm_preset: crate::models::server::JvmPresetConfig,
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
        core_type,
        mc_version,
        jvm_args,
        terminal_mode,
        cpu_policy,
        jvm_preset,
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
    jvm_args: Vec<String>,
    terminal_mode: crate::models::server::LocalTerminalMode,
    cpu_policy: crate::models::server::CpuPolicyConfig,
    jvm_preset: crate::models::server::JvmPresetConfig,
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
        jvm_args,
        terminal_mode,
        cpu_policy,
        jvm_preset,
    )
}

#[tauri::command]
/// 解析服务端核心 key，返回的 `core_type` 为 canonical key
pub async fn parse_server_core_key(
    source_path: String,
) -> Result<crate::models::server::ParsedServerCoreInfo, String> {
    provisioning::parse_server_core_key(source_path).await
}

#[tauri::command]
/// 兼容旧命令名，返回 legacy display-style `core_type`
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
pub async fn start_server(app: tauri::AppHandle, id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || runtime::start_server(app, id))
        .await
        .map_err(|e| format!("start server task failed: {}", e))?
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
pub fn get_terminal_transcript(
    id: String,
    cursor: u64,
    max_bytes: Option<usize>,
) -> Result<crate::services::server::terminal_transcript::TerminalTranscriptChunk, String> {
    runtime::get_terminal_transcript(id, cursor, max_bytes)
}

#[tauri::command]
pub fn send_terminal_input(id: String, input: String) -> Result<(), String> {
    runtime::send_terminal_input(id, input)
}

#[tauri::command]
pub fn resize_terminal(id: String, cols: u16, rows: u16) -> Result<(), String> {
    runtime::resize_terminal(id, cols, rows)
}

#[tauri::command]
pub fn get_server_list() -> Result<Vec<crate::models::server::ServerInstance>, String> {
    runtime::get_server_list_checked()
}

pub(crate) fn get_server_list_checked() -> Result<Vec<crate::models::server::ServerInstance>, String>
{
    runtime::get_server_list_checked()
}

#[tauri::command]
pub fn get_server_status(
    app: tauri::AppHandle,
    id: String,
) -> crate::models::server::ServerStatusInfo {
    runtime::get_server_status(app, id)
}

#[tauri::command]
pub fn delete_server(id: String) -> Result<(), String> {
    runtime::delete_server(id)
}

#[tauri::command]
pub fn get_server_logs(
    id: String,
    since: usize,
    max_lines: Option<usize>,
) -> Result<Vec<String>, String> {
    runtime::get_server_logs(id, since, max_lines)
}

#[tauri::command]
pub fn clear_server_logs(id: String) -> Result<(), String> {
    runtime::clear_server_logs(id)
}

#[tauri::command]
pub fn get_local_launch_detail(
    id: String,
) -> Result<crate::services::server::manager::LocalLaunchDetail, String> {
    runtime::get_local_launch_detail(id)
}

#[cfg(test)]
mod tests {
    use super::get_server_list;
    use crate::services::global;

    #[test]
    fn get_server_list_surfaces_server_list_lock_failures() {
        let manager = global::server_manager();

        let poison_thread = std::thread::spawn(move || {
            let _guard = manager
                .servers
                .lock()
                .expect("servers lock should be acquired");
            panic!("poison server list lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let error = get_server_list()
            .expect_err("lock failure should not be flattened into an empty server list");

        assert_eq!(error, "servers lock poisoned");
    }
}

#[tauri::command]
pub fn get_docker_launch_detail(
    id: String,
) -> Result<crate::services::server::runtime::docker_itzg::DockerLaunchDetail, String> {
    runtime::get_docker_launch_detail(id)
}

#[tauri::command]
pub fn update_server_name(id: String, name: String) -> Result<(), String> {
    runtime::update_server_name(id, name)
}

#[tauri::command]
pub fn update_server_java_path(
    id: String,
    java_path: String,
) -> Result<crate::models::server::ServerInstance, String> {
    runtime::update_server_java_path(id, java_path)
}

#[tauri::command]
pub fn update_server_terminal_mode(
    id: String,
    terminal_mode: crate::models::server::LocalTerminalMode,
) -> Result<crate::models::server::ServerInstance, String> {
    runtime::update_server_terminal_mode(id, terminal_mode)
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
