//! 服务端检查与供给计划 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`ProvisioningService`] 检查服务器目录、解析启动脚本，并生成
//! 现有实例导入、实例复制与整合包供给计划。计划阶段不修改文件系统。
//!
//! 错误统一为接口契约错误 [`ProvisioningServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use std::path::Path;

use sealantern_application::port::ProvisioningService;
use sealantern_application::services::AppServices;
use sealantern_contract::ProvisioningServiceError;
use sealantern_core::instance::{InstanceImportPlan, InstanceImportRequest};
use sealantern_core::provisioning::{
    CopyInstancePlan, CopyInstanceRequest, ModpackProvisionPlan, ModpackProvisionRequest,
    ServerInspectionReport, StartupScriptInfo,
};
use tauri::State;

/// 检查指定服务器目录，返回服务器类型与版本等概况。
#[tauri::command(rename_all = "snake_case")]
pub async fn inspect_server(
    services: State<'_, AppServices>,
    path: String,
) -> Result<ServerInspectionReport, ProvisioningServiceError> {
    services
        .provisioning()
        .inspect_server(Path::new(&path))
        .await
}

/// 解析指定服务器目录下的启动脚本，返回内存与参数等配置信息。
#[tauri::command(rename_all = "snake_case")]
pub async fn parse_startup_script(
    services: State<'_, AppServices>,
    path: String,
) -> Result<StartupScriptInfo, ProvisioningServiceError> {
    services
        .provisioning()
        .parse_startup_script(Path::new(&path))
        .await
}

/// 为导入现有实例生成供给计划。
#[tauri::command(rename_all = "snake_case")]
pub async fn plan_existing_instance(
    services: State<'_, AppServices>,
    request: InstanceImportRequest,
) -> Result<InstanceImportPlan, ProvisioningServiceError> {
    services
        .provisioning()
        .plan_existing_instance(request)
        .await
}

/// 为复制实例生成供给计划。
#[tauri::command(rename_all = "snake_case")]
pub async fn plan_instance_copy(
    services: State<'_, AppServices>,
    request: CopyInstanceRequest,
) -> Result<CopyInstancePlan, ProvisioningServiceError> {
    services.provisioning().plan_copy(request).await
}

/// 为安装整合包生成供给计划。
#[tauri::command(rename_all = "snake_case")]
pub async fn plan_modpack_provision(
    services: State<'_, AppServices>,
    request: ModpackProvisionRequest,
) -> Result<ModpackProvisionPlan, ProvisioningServiceError> {
    services.provisioning().plan_modpack(request).await
}
