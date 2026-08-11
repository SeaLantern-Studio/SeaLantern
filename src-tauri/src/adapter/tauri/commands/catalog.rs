use sealantern_application::services::AppServices;
use sealantern_extra::download_link::TypeDownloadLinks;
use sealantern_interface::{ServerCatalogService, ServerCatalogServiceError};
#[tauri::command]
pub async fn catalog_server_types() -> Result<Vec<String>, ServerCatalogServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ServerCatalogServiceError::OperationFailed)?
        .catalog()
        .server_types()
        .await
}
#[tauri::command]
pub async fn catalog_versions(
    server_type: String,
) -> Result<Vec<String>, ServerCatalogServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ServerCatalogServiceError::OperationFailed)?
        .catalog()
        .versions(server_type)
        .await
}
#[tauri::command]
pub async fn catalog_details(
    server_type: String,
) -> Result<TypeDownloadLinks, ServerCatalogServiceError> {
    AppServices::get()
        .await
        .map_err(|_| ServerCatalogServiceError::OperationFailed)?
        .catalog()
        .details(server_type)
        .await
}
