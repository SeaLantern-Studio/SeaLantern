//! 应用服务自托管容器。
//!
//! 在应用生命周期内持有并暴露全局服务（如实例管理），不绑定 Tauri 生命周期，
//! 便于 Tauri 命令、HTTP / IPC 等任意宿主复用。由 `run` 启动时一次性初始化。

use std::sync::OnceLock;

use sealantern_infra::fs::FsError;

use crate::services::instance::CoreInstanceService;

/// 应用服务的聚合容器，进程生命周期内单例。
pub struct AppServices {
    /// 服务器实例管理服务。
    pub instance: CoreInstanceService,
}

static INSTANCE: OnceLock<AppServices> = OnceLock::new();

impl AppServices {
    /// 初始化所有自托管服务并注册为全局单例。
    ///
    /// 异步构造（实例注册表需加载）。重复调用会失败而非静默覆盖。
    pub async fn init() -> Result<&'static Self, FsError> {
        let services = Self {
            instance: CoreInstanceService::new().await?,
        };
        INSTANCE.set(services).map_err(|_| FsError::Task {
            operation: "init AppServices",
            message: "AppServices already initialized".into(),
        })?;
        Ok(Self::get())
    }

    /// 获取全局 AppServices（须先经 [`Self::init`] 初始化）。
    pub fn get() -> &'static Self {
        INSTANCE.get().expect("AppServices not initialized")
    }
}
