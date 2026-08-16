//! 实例管理 REST handler。
//!
//! 提供实例的查询与 CRUD 接口，薄转发到
//! [`CoreInstanceService`](sealantern_application::service::CoreInstanceService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{
    build_import_spec, source_directories_equal, validate_source_directory,
    ImportExistingServerError as CoreImportError, ImportExistingServerRequest,
    SourceDirectoryError,
};
use sealantern_interface::InstanceService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// PATCH 重命名请求体。
#[derive(Debug, serde::Deserialize)]
pub struct RenameRequest {
    /// 新的实例名称。
    pub name: String,
}

/// 更新目录路径请求体。
#[derive(Debug, serde::Deserialize)]
pub struct UpdatePathRequest {
    /// 新的实例目录路径。
    pub path: String,
}

/// `GET /api/instances` — 列出全部实例。
pub async fn list_instances(
    State(state): State<AppState>,
) -> Result<Json<Vec<Instance>>, HttpError> {
    state
        .instance()
        .list()
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/instances` — 创建新实例。
pub async fn create_instance(
    State(state): State<AppState>,
    Json(spec): Json<InstanceSpec>,
) -> Result<(StatusCode, Json<Instance>), HttpError> {
    let instance = state.instance().create(spec).await?;
    Ok((StatusCode::CREATED, Json(instance)))
}

/// `GET /api/instances/{id}` — 按 ID 查找实例。
pub async fn get_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Instance>, HttpError> {
    let id = parse_id(&id)?;
    let instance = state
        .instance()
        .find(&id)
        .await?
        .ok_or(sealantern_interface::InstanceServiceError::InstanceNotFound)?;
    Ok(Json(instance))
}

/// `DELETE /api/instances/{id}` — 删除实例；不存在时返回 404。
pub async fn delete_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, HttpError> {
    let id = parse_id(&id)?;
    state.instance().delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PATCH /api/instances/{id}` — 重命名实例。
pub async fn rename_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<RenameRequest>,
) -> Result<Json<Instance>, HttpError> {
    let id = parse_id(&id)?;
    state.instance().rename(&id, &request.name).await?;
    let instance = state
        .instance()
        .find(&id)
        .await?
        .ok_or(sealantern_interface::InstanceServiceError::InstanceNotFound)?;
    Ok(Json(instance))
}

/// `PUT /api/instances/{id}/path` — 更新实例目录路径。
pub async fn update_instance_path(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdatePathRequest>,
) -> Result<Json<Instance>, HttpError> {
    let id = parse_id(&id)?;
    state.instance().update_path(&id, &request.path).await?;
    let instance = state
        .instance()
        .find(&id)
        .await?
        .ok_or(sealantern_interface::InstanceServiceError::InstanceNotFound)?;
    Ok(Json(instance))
}

/// 解析路径参数中的实例 ID，非法输入视为客户端错误。
fn parse_id(raw: &str) -> Result<InstanceId, HttpError> {
    InstanceId::new(raw.to_owned())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// `POST /api/instances/import-existing` — 导入已有服务器目录。
///
/// 导入的实例直接引用原始目录（FR-5：不复制文件），启动目标由检查结果采纳最优候选。
pub async fn import_existing_instance(
    State(state): State<AppState>,
    Json(request): Json<ImportExistingServerRequest>,
) -> Result<(StatusCode, Json<Instance>), HttpError> {
    validate_source_directory(&request.source_directory).map_err(|error| match error {
        SourceDirectoryError::Unavailable(_) => {
            HttpError::bad_request("source_unavailable", error.to_string())
        }
        SourceDirectoryError::NotDirectory(_) => {
            HttpError::bad_request("source_not_directory", error.to_string())
        }
    })?;
    let instances = state.instance().list().await?;
    if instances.iter().any(|instance| {
        source_directories_equal(instance.directory.as_path(), request.source_directory.as_path())
    }) {
        return Err(HttpError::bad_request(
            "source_already_imported",
            "the selected directory is already imported as a server instance",
        ));
    }
    let import_request = build_import_spec(&request)?;
    let instance = state.instance().create(import_request.instance).await?;
    Ok((StatusCode::CREATED, Json(instance)))
}

impl From<CoreImportError> for HttpError {
    fn from(error: CoreImportError) -> Self {
        let code = match error {
            CoreImportError::Inspection(_) => "inspection_failed",
            CoreImportError::NoLaunchCandidate => "no_launch_candidate",
            CoreImportError::InvalidInstance(_) => "invalid_instance",
        };
        HttpError::bad_request(code, error.to_string())
    }
}
