use sealantern_application::services::AppServices;
use sealantern_extra::java::JavaDetectionReport;
use sealantern_extra::models::JavaInfo;
use sealantern_interface::{JavaService, JavaServiceError};

#[tauri::command]
pub async fn java_detect() -> Result<JavaDetectionReport, JavaServiceError> {
    AppServices::get()
        .await
        .map_err(|_| JavaServiceError::OperationFailed)?
        .java()
        .detect()
        .await
}
#[tauri::command]
pub async fn java_validate(path: String) -> Result<JavaInfo, JavaServiceError> {
    AppServices::get()
        .await
        .map_err(|_| JavaServiceError::OperationFailed)?
        .java()
        .validate(path)
        .await
}
