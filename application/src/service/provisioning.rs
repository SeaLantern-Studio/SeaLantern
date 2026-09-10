//! 服务端检查与供给计划服务实现。
//!
//! 实现 [`crate::port::ProvisioningService`] 能力端口，组合
//! `core` 的供给规划能力（[`inspect_server_artifact`]、
//! [`parse_startup_script_file`]、[`plan_existing_instance`]、
//! [`plan_copy`]、[`plan_modpack`]），向宿主提供服务器文件检查、
//! 启动脚本解析与各类供给计划生成。
//!
//! 所有暴露操作只做"检查 / 规划"，不产生副作用（不复制文件、不写盘）。
//!
//! 错误分层：`spawn_blocking` 任务调度失败收敛为
//! [`ProvisioningServiceError::OperationFailed`]；服务器检查失败与启动脚本
//! 不可读收敛为 [`ProvisioningServiceError::InspectionFailed`]；脚本格式
//! 不支持与各规划请求非法收敛为 [`ProvisioningServiceError::InvalidInput`]。

use std::path::Path;

use async_trait::async_trait;
use sealantern_contract::ProvisioningServiceError;
use sealantern_core::instance::{InstanceImportPlan, InstanceImportRequest};
use sealantern_core::provisioning::{
    CopyInstancePlan, CopyInstanceRequest, InspectionOptions, ModpackProvisionPlan,
    ModpackProvisionRequest, ServerInspectionReport, StartupParseError, StartupScriptInfo,
    inspect_server_artifact, parse_startup_script_file, plan_copy, plan_existing_instance,
    plan_modpack,
};

use crate::port::ProvisioningService;

/// 基于 `core` 供给规划能力的检查与计划服务实现。
///
/// 暴露的操作均为纯检查 / 规划，不产生副作用。
#[derive(Debug, Default)]
pub struct CoreProvisioningService;

#[async_trait]
impl ProvisioningService for CoreProvisioningService {
    /// 检查服务器文件结构并生成检查报告。
    ///
    /// 文件检查是同步且可能耗时的目录扫描，经 `spawn_blocking` 调度到
    /// 阻塞线程池执行，避免阻塞异步运行时的核心线程。
    async fn inspect_server(
        &self,
        path: &Path,
    ) -> Result<ServerInspectionReport, ProvisioningServiceError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            inspect_server_artifact(&path, &InspectionOptions::default())
        })
        .await
        // 任务调度失败（任务 panic / 取消）视为操作失败。
        .map_err(|_| ProvisioningServiceError::OperationFailed)?
        // 检查判定目标不是可用服务器结构（目录缺失 / 结构不符）视为检查失败。
        .map_err(|_| ProvisioningServiceError::InspectionFailed)
    }

    /// 解析启动脚本并提取可移植的启动信息。
    ///
    /// 同步的文件解析经 `spawn_blocking` 调度，避免阻塞异步 runtime；
    /// 脚本无法读取（缺失 / 不可访问）视为检查失败，脚本格式不支持
    /// 视为输入非法。
    async fn parse_startup_script(
        &self,
        path: &Path,
    ) -> Result<StartupScriptInfo, ProvisioningServiceError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || parse_startup_script_file(&path))
            .await
            // 任务调度失败视为操作失败。
            .map_err(|_| ProvisioningServiceError::OperationFailed)?
            // 读取失败视为检查失败，扩展名不支持视为输入非法。
            .map_err(|error| match error {
                StartupParseError::Read { .. } => ProvisioningServiceError::InspectionFailed,
                StartupParseError::UnsupportedScript { .. } => {
                    ProvisioningServiceError::InvalidInput
                }
            })
    }

    /// 为导入既有实例生成供给计划。
    ///
    /// 请求非法（目录缺失 / 信息不完整）时收敛为非法输入。
    async fn plan_existing_instance(
        &self,
        request: InstanceImportRequest,
    ) -> Result<InstanceImportPlan, ProvisioningServiceError> {
        plan_existing_instance(request).map_err(|_| ProvisioningServiceError::InvalidInput)
    }

    /// 为复制实例生成供给计划（复制范围与步骤清单）。
    async fn plan_copy(
        &self,
        request: CopyInstanceRequest,
    ) -> Result<CopyInstancePlan, ProvisioningServiceError> {
        plan_copy(request).map_err(|_| ProvisioningServiceError::InvalidInput)
    }

    /// 为导入整合包生成供给计划。
    async fn plan_modpack(
        &self,
        request: ModpackProvisionRequest,
    ) -> Result<ModpackProvisionPlan, ProvisioningServiceError> {
        plan_modpack(request).map_err(|_| ProvisioningServiceError::InvalidInput)
    }
}
