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
use sealantern_application::port::{InstanceDiscoveryView, InstanceService};
use sealantern_application::service::CoreInstanceService;
use sealantern_application::services::AppServices;
use sealantern_contract::InstanceServiceError;
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{ImportExistingServerRequest, ImportModpackRequest};
use tauri::State;

/// 获取宿主注入的实例管理服务句柄。
///
/// 应用层主错误 [`InstanceError`] 收敛为契约错误 [`InstanceServiceError`]。
fn instance_service(services: &AppServices) -> Arc<CoreInstanceService> {
    services.instance().clone()
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
pub async fn list_instances(
    services: State<'_, AppServices>,
) -> Result<Vec<Instance>, InstanceServiceError> {
    let service = instance_service(&services);
    service.list().await
}

/// 实例发现视图：已信任实例 + 待处理实例 + 问题清单 + 孤儿信任记录。
///
/// 供前端区分「已管理 / 待处理 / 有问题」三类目录；返回结构与 HTTP
/// `GET /api/instances/discovery` 一致。
#[tauri::command(rename_all = "snake_case")]
pub async fn discover_instances(
    services: State<'_, AppServices>,
) -> Result<InstanceDiscoveryView, InstanceServiceError> {
    let service = instance_service(&services);
    service.discovery().await
}

/// 按 ID 查找实例，不存在时返回 `None`。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_instance(
    services: State<'_, AppServices>,
    id: String,
) -> Result<Option<Instance>, InstanceServiceError> {
    let service = instance_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.find(&id).await
}

/// 创建新实例并持久化。
#[tauri::command(rename_all = "snake_case")]
pub async fn create_instance(
    services: State<'_, AppServices>,
    spec: InstanceSpec,
) -> Result<Instance, InstanceServiceError> {
    let service = instance_service(&services);
    service.create(spec).await
}

/// 删除实例；实例不存在时返回 [`InstanceServiceError::InstanceNotFound`]。
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_instance(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.delete(&id).await
}

/// 重命名实例。
#[tauri::command(rename_all = "snake_case")]
pub async fn rename_instance(
    services: State<'_, AppServices>,
    id: String,
    name: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.rename(&id, &name).await
}

/// 更新实例目录路径。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_instance_path(
    services: State<'_, AppServices>,
    id: String,
    path: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.update_path(&id, &path).await
}

/// 把实例登记为受信任（用户对「待处理实例」点「信任」）。
///
/// 幂等；目标实例身份无效时返回 [`InstanceServiceError::InstanceNotFound`]
/// 或 [`InstanceServiceError::InvalidInput`]。
#[tauri::command(rename_all = "snake_case")]
pub async fn trust_instance(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.trust(&id).await
}

/// 撤销实例的信任登记（用户对已管理实例「不再管理」或清理孤儿记录）。
///
/// 幂等：未登记时视为成功。
#[tauri::command(rename_all = "snake_case")]
pub async fn untrust_instance(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    let id = parse_id_for_tauri(id)?;
    service.untrust(&id).await
}

/// 把目录加入忽略名单（用户对「待处理/问题目录」点「忽略」）。
///
/// 目录不必是有效实例；幂等。
#[tauri::command(rename_all = "snake_case")]
pub async fn ignore_instance_dir(
    services: State<'_, AppServices>,
    path: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    service.ignore_dir(&path).await
}

/// 把目录移出忽略名单（用户「恢复提示」）。幂等。
#[tauri::command(rename_all = "snake_case")]
pub async fn unignore_instance_dir(
    services: State<'_, AppServices>,
    path: String,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    service.unignore_dir(&path).await
}

/// 为指定目录的实例重新分配 id（修复「同 id 出现在多个目录」的冲突）。
///
/// **按目录定位**而非按 id：重复组被发现层整组移出实例列表，按 id 的命令
/// 都够不着它们。`new_id` 缺省时由后端生成 v4 UUID。
#[tauri::command(rename_all = "snake_case")]
pub async fn reassign_instance_id(
    services: State<'_, AppServices>,
    dir: String,
    new_id: Option<String>,
) -> Result<(), InstanceServiceError> {
    let service = instance_service(&services);
    service.reassign_id(&dir, new_id.as_deref()).await
}

/// 导入已有服务器目录：转发至 service 层编排，仅做错误映射。
///
/// 校验、去重、检查与规格构建均在 `CoreInstanceService::import_existing_server`
/// 完成；导入的实例直接引用原始目录（FR-5：不复制文件）。
#[tauri::command(rename_all = "snake_case")]
pub async fn import_existing_server(
    services: State<'_, AppServices>,
    request: ImportExistingServerRequest,
) -> Result<Instance, InstanceServiceError> {
    let service = instance_service(&services);
    service.import_existing_server(request).await
}

/// 导入整合包或服务器文件夹。
///
/// 支持三种来源：
/// - zip/tar.gz/tgz 压缩包：解压到 run_path
/// - jar 单文件：复制到 run_path
/// - 文件夹：直接引用原路径
#[tauri::command(rename_all = "camelCase")]
pub async fn import_modpack(
    services: State<'_, AppServices>,
    request: ImportModpackRequest,
) -> Result<Instance, InstanceServiceError> {
    let service = instance_service(&services);
    service.import_modpack(request).await
}
