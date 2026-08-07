//! `src/api/system.ts` 对应的兼容命令。
//!
//! 前端使用旧命令名（如 `get_system_info`、`get_default_run_path`），本模块把这些
//! 旧名注册为 Tauri 命令，内部经 [`super::adapter`] 做响应适配后调用
//! [`CoreSystemService`](sealantern_application::service::CoreSystemService)。
//!
//! 部分命令为兼容原生（不经服务层，直接返回平台信息或调用 opener 插件）。
//! 未对接后端的命令一律返回 [`SystemServiceError::Unsupported`]。

use sealantern_application::services::AppServices;
use sealantern_core::instance::InstanceId;
use sealantern_core::server::ServerProcessState;
use sealantern_infra::platform::get_app_data_dir;
use sealantern_interface::{InstanceService, SystemService, SystemServiceError};
use tauri_plugin_opener::OpenerExt;

use super::adapter::{process_usage_to_resource_usage, system_snapshot_to_frontend};
use super::error::instance_err_to_system;
use super::models::{
    FrontendServerResourceUsage, FrontendSystemInfo, GetServerResourceUsageParams,
};

// ── 可用命令 ──────────────────────────────────────────────────────────

/// 获取系统信息（兼容 `get_system_info`）。
///
/// 调用 `system_snapshot()` 后用 [`system_snapshot_to_frontend`] 整形为前端形态
/// （网络重整形、swap 丢弃 available）。
#[tauri::command]
pub async fn get_system_info() -> Result<FrontendSystemInfo, SystemServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| SystemServiceError::OperationFailed)?;
    let snapshot = services.system().system_snapshot().await?;
    Ok(system_snapshot_to_frontend(snapshot))
}

/// 获取默认运行路径（兼容 `get_default_run_path`）。
///
/// 兼容原生：不经服务层，直接返回 `{app_data_dir}/servers`。
#[tauri::command]
pub async fn get_default_run_path() -> Result<String, SystemServiceError> {
    let path = get_app_data_dir().join("servers");
    Ok(path.to_string_lossy().to_string())
}

/// 打开文件（兼容 `open_file`）。
///
/// 兼容原生：用 `tauri-plugin-opener` 调用系统默认程序打开文件。
#[tauri::command]
pub async fn open_file(app: tauri::AppHandle, path: String) -> Result<(), SystemServiceError> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|_| SystemServiceError::OperationFailed)
}

/// 打开文件夹（兼容 `open_folder`）。
///
/// 兼容原生：用 `tauri-plugin-opener` 调用系统文件管理器打开目录。
#[tauri::command]
pub async fn open_folder(app: tauri::AppHandle, path: String) -> Result<(), SystemServiceError> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|_| SystemServiceError::OperationFailed)
}

// ── 前向就绪但后端暂 Unsupported ─────────────────────────────────────

/// 获取服务器资源占用（兼容 `get_server_resource_usage`）。
///
/// 跨域调用：先调 instance 服务取 pid，再调 system 服务取进程占用。
/// 第一阶段因 instance.status 未接 Daemon 而返回 Unsupported；适配代码前向就绪。
#[tauri::command]
pub async fn get_server_resource_usage(
    params: GetServerResourceUsageParams,
) -> Result<FrontendServerResourceUsage, SystemServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| SystemServiceError::OperationFailed)?;
    let instance_id = InstanceId::new(params.server_id.clone())
        .map_err(|_| SystemServiceError::OperationFailed)?;

    // 取实例状态获取 pid
    let status = services
        .instance()
        .status(&instance_id)
        .await
        .map_err(instance_err_to_system)?;

    // 仅 Running 时才有有效 pid；Exited 时 process_id 为 stale 值，不可用于查询
    if !matches!(status.state, ServerProcessState::Running) {
        return Err(SystemServiceError::ProcessNotFound);
    }
    let pid = status.process_id;
    let usage = services.system().process_usage(pid).await?;

    // 取实例名
    let instance = services
        .instance()
        .find(&instance_id)
        .await
        .map_err(instance_err_to_system)?
        .ok_or(SystemServiceError::OperationFailed)?;

    Ok(process_usage_to_resource_usage(
        params.server_id,
        instance.name,
        "Running".to_string(),
        usage,
    ))
}

// ── 显式 Unsupported（后端能力未装配）─────────────────────────────────

/// 获取安全模式状态（后端待建）。
#[tauri::command]
pub async fn get_safe_mode_status() -> Result<(), SystemServiceError> {
    Err(SystemServiceError::Unsupported)
}

/// 测试 IPv6 连通性（后端待建）。
#[tauri::command]
pub async fn test_ipv6_connectivity() -> Result<(), SystemServiceError> {
    Err(SystemServiceError::Unsupported)
}

/// 删除文件（后端待建）。
#[tauri::command]
pub async fn remove_file() -> Result<(), SystemServiceError> {
    Err(SystemServiceError::Unsupported)
}
