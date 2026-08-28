//! 服务端检查与实例供给计划宿主能力端口。

use std::path::Path;

use async_trait::async_trait;
use sealantern_core::instance::InstanceImportRequest;
use sealantern_core::provisioning::{
    CopyInstancePlan, CopyInstanceRequest, ModpackProvisionPlan, ModpackProvisionRequest,
    ServerInspectionReport, StartupScriptInfo,
};

use sealantern_contract::ProvisioningServiceError;

/// 服务端检查与供给计划宿主能力端口。
///
/// 所有 `plan_*` 方法只生成计划、不做任何文件系统修改，实际落盘由宿主执行。
#[async_trait]
pub trait ProvisioningService: Send + Sync {
    /// 检查指定服务器目录，返回核心类型、Minecraft 版本与 Java 要求等识别报告。
    async fn inspect_server(
        &self,
        path: &Path,
    ) -> Result<ServerInspectionReport, ProvisioningServiceError>;

    /// 静态解析指定启动脚本（不执行其内容），返回脚本格式与 Java 启动信息。
    async fn parse_startup_script(
        &self,
        path: &Path,
    ) -> Result<StartupScriptInfo, ProvisioningServiceError>;

    /// 规划将已有目录导入为受管实例，不执行任何文件系统操作。
    async fn plan_existing_instance(
        &self,
        request: InstanceImportRequest,
    ) -> Result<sealantern_core::instance::InstanceImportPlan, ProvisioningServiceError>;

    /// 规划将已有目录复制为受管实例，不执行任何文件系统操作。
    async fn plan_copy(
        &self,
        request: CopyInstanceRequest,
    ) -> Result<CopyInstancePlan, ProvisioningServiceError>;

    /// 规划整合包导入为受管实例，不执行任何文件系统操作。
    async fn plan_modpack(
        &self,
        request: ModpackProvisionRequest,
    ) -> Result<ModpackProvisionPlan, ProvisioningServiceError>;
}
