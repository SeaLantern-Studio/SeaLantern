//! 实例启动配置（`sl.json` 的 `startup` 段）Tauri 命令。
//!
//! 与 `server_config` 命令一致，以服务器目录路径定位：服务在该目录的
//! `sl.json` 上读写内存覆盖，因此前端无需额外解析实例标识。

use sealantern_application::port::ServerStartupService;
use sealantern_application::services::AppServices;
use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use tauri::State;

/// 读取实例的启动内存覆盖。
#[tauri::command(rename_all = "snake_case")]
pub async fn read_sl_config(
    services: State<'_, AppServices>,
    server_path: String,
) -> Result<SLStartupConfig, ServerStartupServiceError> {
    services.server_startup().read(&server_path).await
}

/// 写入实例的启动内存覆盖。
#[tauri::command(rename_all = "snake_case")]
pub async fn write_sl_config(
    services: State<'_, AppServices>,
    server_path: String,
    config: SLStartupConfig,
) -> Result<(), ServerStartupServiceError> {
    services.server_startup().write(&server_path, &config).await
}
