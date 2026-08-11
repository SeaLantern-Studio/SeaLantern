//! Snake-case Tauri commands for inspection and provisioning plans.

use std::path::Path;

use sealantern_application::services::AppServices;
use sealantern_core::instance::{InstanceImportPlan, InstanceImportRequest};
use sealantern_core::provisioning::{
    CopyInstancePlan, CopyInstanceRequest, ModpackProvisionPlan, ModpackProvisionRequest,
    ServerInspectionReport, StartupScriptInfo,
};
use sealantern_interface::{ProvisioningService, ProvisioningServiceError};

async fn services() -> Result<AppServices, ProvisioningServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ProvisioningServiceError::OperationFailed)
}

#[tauri::command]
pub async fn inspect_server(
    path: String,
) -> Result<ServerInspectionReport, ProvisioningServiceError> {
    services()
        .await?
        .provisioning()
        .inspect_server(Path::new(&path))
        .await
}

#[tauri::command]
pub async fn parse_startup_script(
    path: String,
) -> Result<StartupScriptInfo, ProvisioningServiceError> {
    services()
        .await?
        .provisioning()
        .parse_startup_script(Path::new(&path))
        .await
}

#[tauri::command]
pub async fn plan_existing_instance(
    request: InstanceImportRequest,
) -> Result<InstanceImportPlan, ProvisioningServiceError> {
    services()
        .await?
        .provisioning()
        .plan_existing_instance(request)
        .await
}

#[tauri::command]
pub async fn plan_instance_copy(
    request: CopyInstanceRequest,
) -> Result<CopyInstancePlan, ProvisioningServiceError> {
    services().await?.provisioning().plan_copy(request).await
}

#[tauri::command]
pub async fn plan_modpack_provision(
    request: ModpackProvisionRequest,
) -> Result<ModpackProvisionPlan, ProvisioningServiceError> {
    services().await?.provisioning().plan_modpack(request).await
}
