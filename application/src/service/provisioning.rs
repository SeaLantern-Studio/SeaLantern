//! Application adapter for side-effect-free provisioning operations.

use std::path::Path;

use async_trait::async_trait;
use sealantern_core::instance::{InstanceImportPlan, InstanceImportRequest};
use sealantern_core::provisioning::{
    inspect_server_artifact, parse_startup_script_file, plan_copy, plan_existing_instance,
    plan_modpack, CopyInstancePlan, CopyInstanceRequest, InspectionOptions, ModpackProvisionPlan,
    ModpackProvisionRequest, ServerInspectionReport, StartupScriptInfo,
};
use sealantern_interface::{ProvisioningService, ProvisioningServiceError};

/// Default provisioning service. All exposed operations inspect or plan only.
#[derive(Debug, Default)]
pub struct CoreProvisioningService;

#[async_trait]
impl ProvisioningService for CoreProvisioningService {
    async fn inspect_server(
        &self,
        path: &Path,
    ) -> Result<ServerInspectionReport, ProvisioningServiceError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            inspect_server_artifact(&path, &InspectionOptions::default())
        })
        .await
        .map_err(|_| ProvisioningServiceError::OperationFailed)?
        .map_err(|_| ProvisioningServiceError::InspectionFailed)
    }

    async fn parse_startup_script(
        &self,
        path: &Path,
    ) -> Result<StartupScriptInfo, ProvisioningServiceError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || parse_startup_script_file(&path))
            .await
            .map_err(|_| ProvisioningServiceError::OperationFailed)?
            .map_err(|_| ProvisioningServiceError::InvalidInput)
    }

    async fn plan_existing_instance(
        &self,
        request: InstanceImportRequest,
    ) -> Result<InstanceImportPlan, ProvisioningServiceError> {
        plan_existing_instance(request).map_err(|_| ProvisioningServiceError::InvalidInput)
    }

    async fn plan_copy(
        &self,
        request: CopyInstanceRequest,
    ) -> Result<CopyInstancePlan, ProvisioningServiceError> {
        plan_copy(request).map_err(|_| ProvisioningServiceError::InvalidInput)
    }

    async fn plan_modpack(
        &self,
        request: ModpackProvisionRequest,
    ) -> Result<ModpackProvisionPlan, ProvisioningServiceError> {
        plan_modpack(request).map_err(|_| ProvisioningServiceError::InvalidInput)
    }
}
