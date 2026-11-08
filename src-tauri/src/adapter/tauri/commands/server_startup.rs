//! 实例启动配置（`sl.json` 的 `startup` 段）Tauri 命令。
//!
//! 以受信任的实例标识定位：前端传入 `instance_id`，命令层解析为
//! `InstanceId` 后交给服务，由服务经实例服务解析为已登记信任的实例目录，
//! 再在其 `sl.json` 上读写内存覆盖——不接受任意文件系统路径。

use sealantern_application::port::ServerStartupService;
use sealantern_application::services::AppServices;
use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_core::instance::InstanceId;
use tauri::State;

/// 读取实例的启动内存覆盖。
#[tauri::command(rename_all = "snake_case")]
pub async fn read_sl_config(
    services: State<'_, AppServices>,
    instance_id: String,
) -> Result<SLStartupConfig, ServerStartupServiceError> {
    let id = parse_instance_id(&instance_id)?;
    services.server_startup().read(&id).await
}

/// 写入实例的启动内存覆盖。
#[tauri::command(rename_all = "snake_case")]
pub async fn write_sl_config(
    services: State<'_, AppServices>,
    instance_id: String,
    config: SLStartupConfig,
) -> Result<(), ServerStartupServiceError> {
    let id = parse_instance_id(&instance_id)?;
    services.server_startup().write(&id, &config).await
}

/// 将前端传入的实例标识字符串解析为 `InstanceId`；非法格式按 `InvalidInput`
/// 拒绝，避免无效 id 进入受信任实例解析链路。
fn parse_instance_id(raw: &str) -> Result<InstanceId, ServerStartupServiceError> {
    InstanceId::new(raw).map_err(|_| ServerStartupServiceError::InvalidInput)
}
