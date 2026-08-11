use async_trait::async_trait;
use sealantern_extra::java::{
    detect_java_installations_with_diagnostics, validate_java, JavaDetectionReport,
};
use sealantern_extra::models::JavaInfo;
use sealantern_interface::{JavaService, JavaServiceError};

#[derive(Debug, Default)]
pub struct CoreJavaService;
#[async_trait]
impl JavaService for CoreJavaService {
    async fn detect(&self) -> Result<JavaDetectionReport, JavaServiceError> {
        tokio::task::spawn_blocking(detect_java_installations_with_diagnostics)
            .await
            .map_err(|_| JavaServiceError::OperationFailed)
    }
    async fn validate(&self, path: String) -> Result<JavaInfo, JavaServiceError> {
        tokio::task::spawn_blocking(move || validate_java(&path))
            .await
            .map_err(|_| JavaServiceError::OperationFailed)?
            .map_err(|_| JavaServiceError::InvalidInput)
    }
}
