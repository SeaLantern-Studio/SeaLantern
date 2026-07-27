//! 系统相关命令

use serde::{Deserialize, Serialize};

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub used_memory: u64,
}

/// 获取系统信息
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    // TODO: 实现真正的系统信息获取
    Ok(SystemInfo {
        os_name: "Unknown".to_string(),
        os_version: "Unknown".to_string(),
        cpu_model: "Unknown".to_string(),
        cpu_cores: num_cpus::get(),
        total_memory: 0,
        used_memory: 0,
    })
}

/// 选择 JAR 文件
#[tauri::command]
pub async fn pick_jar_file() -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现文件选择
    Ok(None)
}

/// 选择压缩包文件
#[tauri::command]
pub async fn pick_archive_file() -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现文件选择
    Ok(None)
}

/// 选择 Java 文件
#[tauri::command]
pub async fn pick_java_file() -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现文件选择
    Ok(None)
}

/// 选择保存文件
#[tauri::command]
pub async fn pick_save_file() -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现文件选择
    Ok(None)
}

/// 获取系统字体列表
#[tauri::command]
pub async fn get_system_fonts() -> Result<Vec<String>, String> {
    // TODO: 实现真正的字体获取
    Ok(vec!["Consolas".to_string(), "Monaco".to_string()])
}

/// 检查开发者模式
#[tauri::command]
pub async fn check_developer_mode() -> Result<bool, String> {
    // TODO: 实现真正的开发者模式检查
    Ok(false)
}

/// 前端心跳
#[tauri::command]
pub async fn frontend_heartbeat() -> Result<(), String> {
    // 心跳命令，不需要做任何事情
    Ok(())
}