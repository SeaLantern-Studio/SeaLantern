//! 实例管理 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`CoreInstanceService`](sealantern_application::service::CoreInstanceService)
//! 执行查询与 CRUD。
//!
//! 错误统一为接口契约错误 [`InstanceServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use std::sync::Arc;

use sealantern_application::error::InstanceError;
use sealantern_application::service::CoreInstanceService;
use sealantern_application::services::AppServices;
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{
    ImportExistingServerRequest, ImportModpackRequest, SourceType, infer_source_type,
    plan_import_modpack,
};
use sealantern_infra::archive::extract_zip;
use sealantern_interface::{
    ImportExistingServerError as AppImportError, InstanceService, InstanceServiceError,
};

/// 获取全局实例管理服务句柄（惰性初始化容器）。
///
/// 应用层主错误 [`InstanceError`] 收敛为契约错误 [`InstanceServiceError`]。
async fn instance_service() -> Result<Arc<CoreInstanceService>, InstanceServiceError> {
    let services = AppServices::get().await?;
    Ok(services.instance().clone())
}

/// 解析 Tauri 命令传入的实例 ID 字符串。
///
/// 统一映射解析错误为 [`InstanceServiceError::InvalidInput`]，避免各命令重复
/// 内联解析与错误映射；后续若调整非法输入的错误变体，只需修改此处。
fn parse_id_for_tauri(id: String) -> Result<InstanceId, InstanceServiceError> {
    InstanceId::new(id)
        .map_err(InstanceError::from)
        .map_err(InstanceServiceError::from)
}

/// 列出全部实例。
#[tauri::command(rename_all = "snake_case")]
pub async fn list_instances() -> Result<Vec<Instance>, InstanceServiceError> {
    let service = instance_service().await?;
    service.list().await
}

/// 按 ID 查找实例，不存在时返回 `None`。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_instance(id: String) -> Result<Option<Instance>, InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.find(&id).await
}

/// 创建新实例并持久化。
#[tauri::command(rename_all = "snake_case")]
pub async fn create_instance(spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
    let service = instance_service().await?;
    service.create(spec).await
}

/// 删除实例；实例不存在时返回 [`InstanceServiceError::InstanceNotFound`]。
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_instance(id: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.delete(&id).await
}

/// 重命名实例。
#[tauri::command(rename_all = "snake_case")]
pub async fn rename_instance(id: String, name: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.rename(&id, &name).await
}

/// 更新实例目录路径。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_instance_path(id: String, path: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.update_path(&id, &path).await
}

/// 导入已有服务器目录失败时返回给前端的错误。
///
/// 同时实现 `Serialize` 与 `std::error::Error`，便于 Tauri 序列化回调用方并携带
/// 稳定的机器可读错误码（如 `source_unavailable` / `no_launch_candidate`）。
#[derive(Debug, serde::Serialize)]
pub struct ImportExistingServerError {
    /// 稳定错误码（机器可读）。
    pub code: String,
    /// 人类可读消息。
    pub message: String,
}

impl std::fmt::Display for ImportExistingServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ImportExistingServerError {}

impl From<AppImportError> for ImportExistingServerError {
    fn from(error: AppImportError) -> Self {
        let code = match error {
            AppImportError::SourceUnavailable => "source_unavailable",
            AppImportError::SourceNotDirectory => "source_not_directory",
            AppImportError::AlreadyImported => "source_already_imported",
            AppImportError::InspectionPanicked => "import_panic",
            AppImportError::BuildFailed => "import_invalid",
            AppImportError::PlanInvalid => "invalid_instance",
            AppImportError::ListFailed => "list_failed",
            AppImportError::CreateFailed => "create_failed",
        };
        Self {
            code: code.to_string(),
            message: error.to_string(),
        }
    }
}

/// 导入已有服务器目录：转发至 service 层编排，仅做错误映射。
///
/// 校验、去重、检查与规格构建均在 `CoreInstanceService::import_existing_server`
/// 完成；导入的实例直接引用原始目录（FR-5：不复制文件）。
#[tauri::command(rename_all = "snake_case")]
pub async fn import_existing_server(
    request: ImportExistingServerRequest,
) -> Result<Instance, ImportExistingServerError> {
    let service = instance_service()
        .await
        .map_err(|error| ImportExistingServerError {
            code: "service_unavailable".to_string(),
            message: error.to_string(),
        })?;
    service
        .import_existing_server(request)
        .await
        .map_err(Into::into)
}

/// 整合包导入失败时返回给前端的错误。
#[derive(Debug, serde::Serialize)]
pub struct ImportModpackError {
    /// 稳定错误码（机器可读）。
    pub code: String,
    /// 人类可读消息。
    pub message: String,
}

impl std::fmt::Display for ImportModpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ImportModpackError {}

/// 构造一个带稳定错误码的整合包导入错误。
fn modpack_error(code: &'static str, message: impl Into<String>) -> ImportModpackError {
    ImportModpackError {
        code: code.to_string(),
        message: message.into(),
    }
}

/// 导入整合包或服务器文件夹。
///
/// 支持三种来源：
/// - zip/tar.gz/tgz 压缩包：解压到 run_path
/// - jar 单文件：复制到 run_path
/// - 文件夹：直接引用原路径
#[tauri::command(rename_all = "camelCase")]
pub async fn import_modpack(request: ImportModpackRequest) -> Result<Instance, ImportModpackError> {
    use sealantern_core::provisioning::ImportModpackError as CoreError;

    // 1. 判断来源类型
    let source_type = infer_source_type(&request.modpack_path);

    // 2. 规划导入（构建 InstanceSpec）
    let result = plan_import_modpack(&request).map_err(|error| match error {
        CoreError::InvalidStartupMode(msg) => {
            modpack_error("invalid_startup_mode", format!("invalid startup mode: {msg}"))
        }
        CoreError::InvalidInstanceId(err) => {
            modpack_error("invalid_instance_id", format!("failed to create instance ID: {err}"))
        }
        CoreError::ExtractFailed(msg) => modpack_error("extract_failed", msg),
        CoreError::CreateDirectoryFailed(msg) => modpack_error("create_directory_failed", msg),
        CoreError::CopyFailed(msg) => modpack_error("copy_failed", msg),
    })?;

    // 3. 执行文件操作（根据来源类型）
    match source_type {
        SourceType::Archive => {
            // 解压 zip/tar.gz 到 run_path
            extract_zip(&request.modpack_path, &result.directory).map_err(|error| {
                modpack_error("extract_failed", format!("failed to extract archive: {error}"))
            })?;
        }
        SourceType::JarFile => {
            // 创建 run_path 目录并复制 jar
            std::fs::create_dir_all(&result.directory).map_err(|error| {
                modpack_error(
                    "create_directory_failed",
                    format!("failed to create run directory: {error}"),
                )
            })?;
            if let Some(ref dest_path) = result.startup_target {
                std::fs::copy(&request.modpack_path, dest_path).map_err(|error| {
                    modpack_error("copy_jar_failed", format!("failed to copy jar file: {error}"))
                })?;
            }
        }
        SourceType::Folder => {
            // 直接引用原目录，无需文件操作
        }
    }

    // 4. 创建实例
    let service = instance_service()
        .await
        .map_err(|error| modpack_error("service_unavailable", error.to_string()))?;
    service
        .create(result.spec)
        .await
        .map_err(|error| modpack_error("create_failed", error.to_string()))
}
