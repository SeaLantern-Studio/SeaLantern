//! 服务器控制台日志 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`ConsoleService`] 增量读取服务器控制台日志。
//!
//! 错误统一为接口契约错误 [`ConsoleServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use sealantern_application::port::ConsoleService;
use sealantern_application::services::AppServices;
use sealantern_contract::ConsoleServiceError;
use sealantern_contract::console::ConsoleLogLine;
use sealantern_core::instance::InstanceId;

/// 解析 Tauri 命令传入的实例 ID 字符串。
///
/// 统一映射解析错误为 [`ConsoleServiceError::InvalidInput`]。
fn parse_id_for_tauri(id: String) -> Result<InstanceId, ConsoleServiceError> {
    InstanceId::new(id).map_err(|_| ConsoleServiceError::InvalidInput)
}

/// 读取服务器控制台日志（增量游标 + 最近 N 行窗口）。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_server_logs(
    id: String,
    since: i64,
    recent_limit: Option<i64>,
) -> Result<Vec<ConsoleLogLine>, ConsoleServiceError> {
    let service = AppServices::console_service()
        .await
        .map_err(|_| ConsoleServiceError::OperationFailed)?;
    let id = parse_id_for_tauri(id)?;
    service.logs(&id, since, recent_limit).await
}
