use async_trait::async_trait;
use sealantern_extra::download_link::{LinkManager, TypeDownloadLinks};
use sealantern_interface::{ServerCatalogService, ServerCatalogServiceError};
#[derive(Debug, Default)]
pub struct CoreServerCatalogService;
#[async_trait]
impl ServerCatalogService for CoreServerCatalogService {
    async fn server_types(&self) -> Result<Vec<String>, ServerCatalogServiceError> {
        LinkManager::get_server_types()
            .await
            .map_err(|_| ServerCatalogServiceError::OperationFailed)
    }
    async fn versions(
        &self,
        server_type: String,
    ) -> Result<Vec<String>, ServerCatalogServiceError> {
        LinkManager::get_versions_by_type(&server_type)
            .await
            .map_err(|_| ServerCatalogServiceError::NotFound)
    }
    async fn details(
        &self,
        server_type: String,
    ) -> Result<TypeDownloadLinks, ServerCatalogServiceError> {
        LinkManager::get_type_by_name(&server_type)
            .await
            .map_err(|_| ServerCatalogServiceError::NotFound)
    }
}
