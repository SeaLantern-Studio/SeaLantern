//! 应用服务自托管容器（装配层）。
//!
//! 在进程内持有并暴露全局服务，不绑定 Tauri 生命周期，便于 Tauri 命令、
//! HTTP / IPC 等任意宿主复用。本模块只负责把 [`CoreInstanceService`] 等
//! 实现装配进容器，不承载业务逻辑。
//!
//! 容器使用异步安全的 `tokio::sync::RwLock<Option<Arc<...>>>`：
//! - 惰性初始化：首次 `get()` 才异步构造，避免 `run()` 时 `block_on` 长时间阻塞启动；
//! - 可替换：测试可注入干净实例 / 配置变更可重载，不依赖 `OnceLock` 的"一次定死"；
//! - 并发安全：多次并发 `get()` 只初始化一次，不重复加载。
//!
//! `AppServices` 是内部 `Arc` 的轻量句柄（clone 廉价 → 可跨 async 边界随处持有）。

use std::sync::Arc;

use crate::error::InstanceError;
use crate::service::CoreInstanceService;

/// 真正的全局服务容器（进程级单例，内部为异步锁 + 可配置）。
#[derive(Clone)]
pub struct AppServices {
    inner: Arc<AppServicesInner>,
}

/// 被 `AppServices` 持有的服务聚合。
///
/// 各服务以 `Arc<T>` 持有，便于便捷访问函数直接对外暴露共享句柄；
/// 后续新增服务只需在此加一个 `Arc<XxxService>` 字段，在 [`AppServices`]
/// 下补一条 `pub async fn xxx_service()` 便捷函数即可，无需改动调用方。
pub struct AppServicesInner {
    /// 服务器实例管理服务。
    pub instance: Arc<CoreInstanceService>,
}

/// 进程级全局容器。惰性初始化，可替换。
static SERVICES: tokio::sync::RwLock<Option<Arc<AppServicesInner>>> =
    tokio::sync::RwLock::const_new(None);

impl AppServices {
    /// 从既有实例构造句柄（供测试/重载注入 `register`）。
    pub fn from_inner(instance: CoreInstanceService) -> Self {
        Self {
            inner: Arc::new(AppServicesInner { instance: Arc::new(instance) }),
        }
    }

    /// 惰性获取全局服务。
    ///
    /// 首次调用时异步构造（实例注册表加载），并注册为全局；之后调用复用同一实例。
    /// 并发首次调用也只初始化一次，其余等待并复用首个注册的结果。
    pub async fn get() -> Result<Self, InstanceError> {
        // 快速路径：已初始化直接返回（读锁，无 IO）。
        if let Some(existing) = SERVICES.read().await.as_ref() {
            return Ok(Self { inner: existing.clone() });
        }

        // 惰性构造：释放读锁后异步加载，避免持锁阻塞。
        let built = Self::from_inner(CoreInstanceService::new().await?);

        // 注册：加写锁；若并发期间已有人注册，则复用其结果，丢弃本次构造。
        let mut guard = SERVICES.write().await;
        Ok(Self {
            inner: match guard.as_ref() {
                Some(existing) => existing.clone(),
                None => {
                    guard.replace(built.inner.clone());
                    built.inner.clone()
                }
            },
        })
    }

    /// 显式注册给定服务（启动预热 / 测试注入 / 重载用），覆盖既有实例。
    pub async fn register(instance: CoreInstanceService) -> Result<Self, InstanceError> {
        let services = Self::from_inner(instance);
        let inner = services.inner.clone();
        *SERVICES.write().await = Some(inner.clone());
        Ok(Self { inner })
    }

    /// 非阻塞尝试取全局服务；未初始化时返回 `None`（供无需初始化的路径判断）。
    pub fn try_get() -> Option<Self> {
        SERVICES
            .try_read()
            .ok()
            .and_then(|g| g.clone().map(|inner| Self { inner }))
    }

    /// 访问实例管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn instance(&self) -> &Arc<CoreInstanceService> {
        &self.inner.instance
    }

    /// 便捷访问入口：一步拿到实例管理服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn instance_service() -> Result<Arc<CoreInstanceService>, InstanceError> {
        Ok(Self::get().await?.instance().clone())
    }
}
