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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::InstanceError;
use crate::plugin::{ApplicationPluginReadHost, CorePluginService, PluginServiceError};
use crate::service::{
    CoreCronTaskService, CoreDownloadService, CoreInstanceService, CoreProvisioningService,
    CoreServerService, CoreSettingsService, CoreSystemService, CoreUpdateCheckService,
    ProxyMonitoringService,
};

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
    background_started: AtomicBool,
    /// 下载任务管理服务。
    pub download: Arc<CoreDownloadService>,
    /// 服务器实例记录管理服务。
    pub instance: Arc<CoreInstanceService>,
    /// 服务端检查与供给计划服务。
    pub provisioning: Arc<CoreProvisioningService>,
    /// 服务器进程管理服务。
    pub server: Arc<CoreServerService>,
    /// 服务器定时任务服务。
    pub cron: Arc<CoreCronTaskService>,
    /// 设置信息服务。
    pub settings: Arc<CoreSettingsService>,
    /// 系统代理轮询服务。
    pub proxy_monitoring: Arc<ProxyMonitoringService>,
    /// 系统资源信息服务。
    pub system: Arc<CoreSystemService>,
    /// 应用更新检查服务。
    pub update: Arc<CoreUpdateCheckService>,
    /// 惰性初始化的应用插件服务。
    plugin: tokio::sync::OnceCell<Arc<CorePluginService>>,
}

/// 进程级全局容器。惰性初始化，可替换。
static SERVICES: tokio::sync::RwLock<Option<Arc<AppServicesInner>>> =
    tokio::sync::RwLock::const_new(None);

impl AppServices {
    /// 从既有实例构造句柄（供测试/重载注入 `register`）。
    ///
    /// 服务器进程服务共享同一实例服务句柄；下载/定时任务/系统资源服务自动构造。
    pub fn from_inner(instance: CoreInstanceService) -> Self {
        let instance = Arc::new(instance);
        let server = Arc::new(CoreServerService::new(instance.clone()));
        Self {
            inner: Arc::new(AppServicesInner {
                background_started: AtomicBool::new(false),
                download: Arc::new(CoreDownloadService::new()),
                cron: Arc::new(CoreCronTaskService::new(server.clone())),
                server,
                instance,
                provisioning: Arc::new(CoreProvisioningService),
                settings: Arc::new(CoreSettingsService::new()),
                proxy_monitoring: Arc::new(ProxyMonitoringService::new()),
                system: Arc::new(CoreSystemService),
                update: Arc::new(CoreUpdateCheckService::new()),
                plugin: tokio::sync::OnceCell::new(),
            }),
        }
    }

    /// 惰性获取全局服务。
    ///
    /// 首次调用时异步构造（实例注册表加载），并注册为全局；之后调用复用同一实例。
    /// 并发首次调用也只初始化一次，其余等待并复用首个注册的结果。
    pub async fn get() -> Result<Self, InstanceError> {
        // 快速路径：已初始化直接返回（读锁，无 IO）。
        if let Some(existing) = SERVICES.read().await.clone() {
            let services = Self { inner: existing };
            services.start_background_services().await;
            return Ok(services);
        }

        // 惰性构造：释放读锁后异步加载，避免持锁阻塞。
        let built = Self::from_inner(CoreInstanceService::new().await?);

        // 注册：加写锁；若并发期间已有人注册,则复用其结果，丢弃本次构造。
        let mut guard = SERVICES.write().await;
        let inner = match guard.as_ref() {
            Some(existing) => existing.clone(),
            None => {
                guard.replace(built.inner.clone());
                built.inner.clone()
            }
        };
        drop(guard);

        let services = Self { inner };
        services.start_background_services().await;
        Ok(services)
    }

    /// 显式注册给定服务（启动预热 / 测试注入 / 重载用），覆盖既有实例。
    pub async fn register(instance: CoreInstanceService) -> Result<Self, InstanceError> {
        let services = Self::from_inner(instance);
        let inner = services.inner.clone();
        let previous = SERVICES.write().await.replace(inner.clone());
        if let Some(previous) = previous {
            previous.cron.deactivate_scheduler().await;
            previous.proxy_monitoring.stop().await;
        }
        let services = Self { inner };
        services.start_background_services().await;
        Ok(services)
    }

    /// 非阻塞尝试取全局服务；未初始化时返回 `None`（供无需初始化的路径判断）。
    pub fn try_get() -> Option<Self> {
        SERVICES
            .try_read()
            .ok()
            .and_then(|g| g.clone().map(|inner| Self { inner }))
    }

    /// 访问下载任务管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn download(&self) -> &Arc<CoreDownloadService> {
        &self.inner.download
    }

    /// 便捷访问入口：一步拿到下载任务管理服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn download_service() -> Result<Arc<CoreDownloadService>, InstanceError> {
        Ok(Self::get().await?.download().clone())
    }

    /// 访问实例管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn instance(&self) -> &Arc<CoreInstanceService> {
        &self.inner.instance
    }

    /// 访问服务端检查与供给计划服务。
    pub fn provisioning(&self) -> &Arc<CoreProvisioningService> {
        &self.inner.provisioning
    }

    /// 便捷访问入口：一步拿到实例管理服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn instance_service() -> Result<Arc<CoreInstanceService>, InstanceError> {
        Ok(Self::get().await?.instance().clone())
    }

    /// 访问服务器进程管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn server(&self) -> &Arc<CoreServerService> {
        &self.inner.server
    }

    /// 便捷访问入口：一步拿到服务器进程管理服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn server_service() -> Result<Arc<CoreServerService>, InstanceError> {
        Ok(Self::get().await?.server().clone())
    }

    /// 访问设置信息服务（`Arc` 共享句柄，clone 廉价）。
    pub fn settings(&self) -> &Arc<CoreSettingsService> {
        &self.inner.settings
    }

    /// 加载持久化设置并同步全局网络运行时。
    pub async fn initialize_network_settings(
        &self,
    ) -> Result<(), sealantern_interface::SettingsServiceError> {
        self.inner.settings.initialize().await?;
        if self.inner.proxy_monitoring.start().await {
            tracing::info!(
                target: "sealantern.application.proxy_monitoring",
                poll_interval_seconds = 3,
                "system proxy monitoring started"
            );
        }
        Ok(())
    }

    /// 便捷访问入口：一步拿到设置信息服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn settings_service() -> Result<Arc<CoreSettingsService>, InstanceError> {
        Ok(Self::get().await?.settings().clone())
    }

    /// 访问服务器定时任务服务（`Arc` 共享句柄，clone 廉价）。
    pub fn cron(&self) -> &Arc<CoreCronTaskService> {
        &self.inner.cron
    }

    /// 便捷访问入口：一步拿到定时任务服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn cron_service() -> Result<Arc<CoreCronTaskService>, InstanceError> {
        Ok(Self::get().await?.cron().clone())
    }

    async fn start_background_services(&self) {
        if self
            .inner
            .background_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        if self.inner.cron.start_scheduler().await {
            tracing::info!(
                target: "sealantern.application.cron_task",
                "cron scheduler started"
            );
        }
    }

    /// 访问系统资源信息服务（`Arc` 共享句柄，clone 廉价）。
    pub fn system(&self) -> &Arc<CoreSystemService> {
        &self.inner.system
    }

    /// 便捷访问入口：一步拿到系统资源信息服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn system_service() -> Result<Arc<CoreSystemService>, InstanceError> {
        Ok(Self::get().await?.system().clone())
    }

    /// 访问应用更新检查服务（`Arc` 共享句柄，clone 廉价）。
    pub fn update(&self) -> &Arc<CoreUpdateCheckService> {
        &self.inner.update
    }

    /// 便捷访问入口：一步拿到更新检查服务的共享句柄（惰性初始化 + 可替换）。
    pub async fn update_service() -> Result<Arc<CoreUpdateCheckService>, InstanceError> {
        Ok(Self::get().await?.update().clone())
    }

    /// 获取应用插件服务；首次调用才打开策略数据库，避免阻塞常规启动路径。
    pub async fn plugin(&self) -> Result<&Arc<CorePluginService>, PluginServiceError> {
        let system = self.inner.system.clone();
        let instance = self.inner.instance.clone();
        let server = self.inner.server.clone();
        self.inner
            .plugin
            .get_or_try_init(move || async move {
                let root = sealantern_infra::platform::get_app_data_dir().join("plugins");
                CorePluginService::open_with_read_host(
                    &root,
                    root.join("data"),
                    root.join("plugin-state.sqlite"),
                    Some(Arc::new(ApplicationPluginReadHost::new(system, instance, server, &root))),
                )
                .await
                .map(Arc::new)
            })
            .await
    }

    /// 便捷访问入口：一步拿到应用插件服务的共享句柄。
    pub async fn plugin_service() -> Result<Arc<CorePluginService>, PluginServiceError> {
        Ok(Self::get()
            .await
            .map_err(|error| PluginServiceError::Initialization(error.to_string()))?
            .plugin()
            .await?
            .clone())
    }
}
