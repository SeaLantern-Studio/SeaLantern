use async_trait::async_trait;
use sealantern_extra::java::JavaDetectionReport;
use sealantern_extra::models::JavaInfo;

use crate::error::JavaServiceError;

#[async_trait]
pub trait JavaService: Send + Sync {
    async fn detect(&self) -> Result<JavaDetectionReport, JavaServiceError>;
    async fn validate(&self, path: String) -> Result<JavaInfo, JavaServiceError>;
}
