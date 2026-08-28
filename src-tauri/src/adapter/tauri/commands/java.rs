//! Java 运行时检测与校验 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`JavaService`] 扫描本机 Java 安装，或校验指定路径的 Java 可执行文件。
//!
//! 错误统一为接口契约错误 [`JavaServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use sealantern_application::port::JavaService;
use sealantern_application::services::AppServices;
use sealantern_contract::JavaServiceError;
use sealantern_contract::java::{JavaDetectionReport, JavaInfo};

/// 自动检测本机已安装的 Java 运行时。
///
/// 返回检测报告，成功安装与非致命错误同时保留，供前端选择 Java 版本。
#[tauri::command(rename_all = "snake_case")]
pub async fn java_detect() -> Result<JavaDetectionReport, JavaServiceError> {
    AppServices::get()
        .await
        .map_err(|_| JavaServiceError::OperationFailed)?
        .java()
        .detect()
        .await
}

/// 校验指定路径的 Java 可执行文件并返回其运行信息。
#[tauri::command(rename_all = "snake_case")]
pub async fn java_validate(path: String) -> Result<JavaInfo, JavaServiceError> {
    AppServices::get()
        .await
        .map_err(|_| JavaServiceError::OperationFailed)?
        .java()
        .validate(path)
        .await
}
