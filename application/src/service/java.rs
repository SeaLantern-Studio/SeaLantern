//! Java 环境检测与校验服务实现。
//!
//! 实现 [`sealantern_interface::JavaService`] 能力端口，组合 `extra` 的
//! Java 探测能力（[`detect_java_installations_with_diagnostics`]、
//! [`validate_java`]），向宿主提供本机 Java 安装检测与指定可执行文件校验。
//!
//! 探测与校验都是同步且可能耗时的文件系统 / 进程操作，经 `spawn_blocking`
//! 调度到阻塞线程池执行，避免阻塞异步运行时的核心线程。
//!
//! 错误分层：任务调度失败（join 错误）收敛为
//! [`JavaServiceError::OperationFailed`]；校验判定 Java 无效收敛为
//! [`JavaServiceError::InvalidInput`]。

use async_trait::async_trait;
use sealantern_extra::java::{
    JavaDetectionReport, detect_java_installations_with_diagnostics, validate_java,
};
use sealantern_extra::models::JavaInfo;
use sealantern_interface::{JavaService, JavaServiceError};

/// 基于 `extra` Java 探测能力的 Java 环境服务实现。
#[derive(Debug, Default)]
pub struct CoreJavaService;

#[async_trait]
impl JavaService for CoreJavaService {
    /// 检测本机已安装的全部 Java 环境。
    ///
    /// Java 探测是同步的文件系统扫描，经 `spawn_blocking` 调度到阻塞
    /// 线程池执行，避免阻塞异步运行时的核心线程。
    async fn detect(&self) -> Result<JavaDetectionReport, JavaServiceError> {
        tokio::task::spawn_blocking(detect_java_installations_with_diagnostics)
            .await
            .map_err(|_| JavaServiceError::OperationFailed)
    }
    /// 校验指定 Java 可执行文件并读取其安装信息。
    ///
    /// 同步的进程 / 文件操作经 `spawn_blocking` 调度；闭包 move 捕获路径，
    /// 避免借用跨 await 边界。
    async fn validate(&self, path: String) -> Result<JavaInfo, JavaServiceError> {
        tokio::task::spawn_blocking(move || validate_java(&path))
            .await
            // 任务调度失败（任务 panic / 取消）视为操作失败。
            .map_err(|_| JavaServiceError::OperationFailed)?
            // 校验判定 Java 无效（路径为空 / 不是有效安装）视为输入非法。
            .map_err(|_| JavaServiceError::InvalidInput)
    }
}
