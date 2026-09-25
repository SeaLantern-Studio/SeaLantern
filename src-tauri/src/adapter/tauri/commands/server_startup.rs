//! 实例启动配置（SeaLantern/config.toml）Tauri 命令。
//!
//! 命令只接受实例标识：服务器目录由应用层从实例注册表解析，
//! 因此前端无法指定任意路径。

use sealantern_application::port::ServerStartupService;
use sealantern_application::services::AppServices;
use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_core::instance::InstanceId;
use tauri::State;

/// 解析前端传入的实例标识，非法输入视为客户端错误。
fn parse_id(raw: String) -> Result<InstanceId, ServerStartupServiceError> {
    InstanceId::new(raw).map_err(|_| ServerStartupServiceError::InvalidInput)
}

/// 读取实例的启动配置覆盖。
#[tauri::command]
pub async fn read_sl_config(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<SLStartupConfig, ServerStartupServiceError> {
    let id = parse_id(instance_id)?;
    services.server_startup().read(&id).await
}

/// 写入实例的启动配置覆盖。
#[tauri::command]
pub async fn write_sl_config(
    services: State<'_, AppServices>,
    instance_id: String,
    config: SLStartupConfig,
) -> Result<(), ServerStartupServiceError> {
    let id = parse_id(instance_id)?;
    services.server_startup().write(&id, &config).await
}
