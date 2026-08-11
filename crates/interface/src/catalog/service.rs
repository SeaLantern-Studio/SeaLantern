use crate::error::ServerCatalogServiceError;
use async_trait::async_trait;
use sealantern_extra::download_link::TypeDownloadLinks;
#[async_trait]
pub trait ServerCatalogService: Send + Sync {
    async fn server_types(&self) -> Result<Vec<String>, ServerCatalogServiceError>;
    async fn versions(&self, server_type: String)
        -> Result<Vec<String>, ServerCatalogServiceError>;
    async fn details(
        &self,
        server_type: String,
    ) -> Result<TypeDownloadLinks, ServerCatalogServiceError>;
}
