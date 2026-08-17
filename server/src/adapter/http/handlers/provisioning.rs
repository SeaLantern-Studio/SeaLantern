//! 供给计划 REST handler。
//!
//! 提供服务器目录检查与启动脚本解析接口，薄转发到
//! [`ProvisioningService`](sealantern_interface::ProvisioningService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use std::path::Path;

use sealantern_core::provisioning::ServerInspectionReport;
use sealantern_interface::ProvisioningService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// `POST /api/provisioning/inspect` — 检查服务器目录。
pub async fn inspect_server(
    State(state): State<AppState>,
    Json(request): Json<InspectServerRequest>,
) -> Result<Json<ServerInspectionReport>, HttpError> {
    let report = state
        .provisioning()
        .inspect_server(Path::new(&request.path))
        .await?;
    Ok(Json(report))
}

/// 检查服务器目录请求体。
#[derive(Debug, serde::Deserialize)]
pub struct InspectServerRequest {
    /// 服务器目录或文件路径。
    pub path: String,
}
