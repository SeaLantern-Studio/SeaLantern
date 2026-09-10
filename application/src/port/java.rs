//! Java 检测与校验宿主能力端口。

use async_trait::async_trait;
use sealantern_contract::JavaServiceError;
use sealantern_contract::java::{JavaDetectionReport, JavaInfo};

/// Java 检测与校验宿主能力端口。
#[async_trait]
pub trait JavaService: Send + Sync {
    /// 扫描本机可用的 Java 安装。
    ///
    /// 返回的报告中同时保留成功安装与非致命错误，供调用方判断缺失项。
    async fn detect(&self) -> Result<JavaDetectionReport, JavaServiceError>;
    /// 校验指定路径的 Java 安装是否可用，并返回其环境信息。
    ///
    /// 路径不可用、不可执行或无法解析版本时返回错误。
    async fn validate(&self, path: String) -> Result<JavaInfo, JavaServiceError>;
}
