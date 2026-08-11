use sealantern_application::services::AppServices;
use sealantern_extra::update::PendingUpdate;
use sealantern_interface::{UpdateInstallService, UpdateInstallServiceError};
async fn service() -> Result<AppServices, UpdateInstallServiceError> {
    AppServices::get().await.map_err(|error| {
        tracing::error!(
            target: "sealantern.tauri.update_install",
            error = %error,
            "failed to initialize application services for update installation"
        );
        UpdateInstallServiceError::OperationFailed
    })
}
#[tauri::command]
pub async fn update_download(
    url: String,
    expected_hash: Option<String>,
    version: String,
) -> Result<String, UpdateInstallServiceError> {
    service()
        .await?
        .update_install()
        .download(url, expected_hash, version)
        .await
}
#[tauri::command]
pub async fn update_pending() -> Result<Option<PendingUpdate>, UpdateInstallServiceError> {
    service().await?.update_install().pending().await
}
#[tauri::command]
pub async fn update_clear_pending() -> Result<(), UpdateInstallServiceError> {
    service().await?.update_install().clear_pending().await
}
#[tauri::command]
pub async fn update_install(
    file_path: String,
    arguments: Vec<String>,
) -> Result<(), UpdateInstallServiceError> {
    service()
        .await?
        .update_install()
        .install(file_path, arguments)
        .await
}
