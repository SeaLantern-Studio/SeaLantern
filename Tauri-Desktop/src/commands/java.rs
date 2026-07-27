//! Java 相关命令

use crate::models::JavaInfo;
use std::sync::Mutex;
use tauri::State;

/// Java 状态管理
pub struct JavaState {
    pub detected: Mutex<Vec<JavaInfo>>,
}

impl Default for JavaState {
    fn default() -> Self {
        Self {
            detected: Mutex::new(Vec::new()),
        }
    }
}

/// 检测系统中的 Java 安装
#[tauri::command]
pub async fn detect_java(state: State<'_, JavaState>) -> Result<Vec<JavaInfo>, String> {
    // TODO: 实现真正的 Java 检测逻辑
    // 目前返回空列表
    let detected = vec![];

    let mut current = state.detected.lock().map_err(|e| e.to_string())?;
    *current = detected.clone();

    Ok(detected)
}

/// 验证 Java 路径
#[tauri::command]
pub async fn validate_java_path(path: String) -> Result<JavaInfo, String> {
    // TODO: 实现真正的 Java 路径验证
    Ok(JavaInfo {
        version: "unknown".to_string(),
        path,
        major_version: Some(17),
    })
}

/// 安装 Java
#[tauri::command]
pub async fn install_java(url: String, version_name: String) -> Result<(), String> {
    // TODO: 实现真正的 Java 安装逻辑
    tracing::info!("Installing Java {} from {}", version_name, url);
    Ok(())
}

/// 取消 Java 安装
#[tauri::command]
pub async fn cancel_java_install() -> Result<(), String> {
    // TODO: 实现取消安装逻辑
    Ok(())
}