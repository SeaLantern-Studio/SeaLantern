//! 服务端检查与供给计划 REST handler。
//!
//! 薄转发到 [`CoreProvisioningService`](sealantern_application::service::CoreProvisioningService)，
//! 错误收敛为 [`HttpError`](super::super::error::HttpError)。

use std::path::Path;

use axum::Json;
use axum::extract::State;

use sealantern_core::provisioning::ServerInspectionReport;
use sealantern_interface::ProvisioningService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// `POST /api/provisioning/inspect` — 检查服务器目录，返回识别概况。
///
/// 浏览器/Docker 模式与桌面 Tauri `inspect_server` 命令共享同一应用层实现，
/// 前端预览「导入已有服务器」时复用此接口。
#[derive(Debug, serde::Deserialize)]
pub struct InspectRequest {
    /// 待检查的服务器目录或文件绝对路径。
    pub path: String,
}

pub async fn inspect_server(
    State(state): State<AppState>,
    Json(request): Json<InspectRequest>,
) -> Result<Json<ServerInspectionReport>, HttpError> {
    let report = state
        .provisioning()
        .inspect_server(Path::new(&request.path))
        .await?;
    Ok(Json(report))
}
