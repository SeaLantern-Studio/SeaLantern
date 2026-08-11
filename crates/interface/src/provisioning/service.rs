use std::path::Path;

use async_trait::async_trait;
use sealantern_core::instance::InstanceImportRequest;
use sealantern_core::provisioning::{
    CopyInstancePlan, CopyInstanceRequest, ModpackProvisionPlan, ModpackProvisionRequest,
    ServerInspectionReport, StartupScriptInfo,
};

use crate::error::ProvisioningServiceError;

/// Host-facing provisioning capability. Planning never changes the filesystem.
#[async_trait]
pub trait ProvisioningService: Send + Sync {
    async fn inspect_server(
        &self,
        path: &Path,
    ) -> Result<ServerInspectionReport, ProvisioningServiceError>;

    async fn parse_startup_script(
        &self,
        path: &Path,
    ) -> Result<StartupScriptInfo, ProvisioningServiceError>;

    async fn plan_existing_instance(
        &self,
        request: InstanceImportRequest,
    ) -> Result<sealantern_core::instance::InstanceImportPlan, ProvisioningServiceError>;

    async fn plan_copy(
        &self,
        request: CopyInstanceRequest,
    ) -> Result<CopyInstancePlan, ProvisioningServiceError>;

    async fn plan_modpack(
        &self,
        request: ModpackProvisionRequest,
    ) -> Result<ModpackProvisionPlan, ProvisioningServiceError>;
}
