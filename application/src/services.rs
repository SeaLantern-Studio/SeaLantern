//! 应用服务自托管容器（装配层）。
//!
//! 在进程内持有一组共享应用服务，具体由 Tauri / HTTP 等宿主的 composition root
//! 显式构造和持有。本模块只负责把 [`CoreInstanceService`] 等实现装配进容器，
//! 不承载业务逻辑，也不提供隐式的全局 service locator。
//!
//! `AppServices` 是内部 `Arc` 的轻量句柄（clone 廉价 → 可跨 async 边界随处持有）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::InstanceError;
use crate::plugin::{ApplicationPluginReadHost, CorePluginService, PluginServiceError};
use crate::port::OnlineTunnelService;
use crate::service::{
    CoreBackupService, CoreConsoleService, CoreCronTaskService, CoreDownloadService,
    CoreInstanceService, CoreJavaService, CoreOnlineTunnelService, CoreProvisioningService,
    CoreServerCatalogService, CoreServerConfigService, CoreServerService, CoreSettingsService,
    CoreSystemService, CoreUpdateCheckService, CoreUpdateInstallService, ProxyMonitoringService,
};

/// 应用服务聚合句柄；由宿主 composition root 创建并显式传递。
#[derive(Clone)]
pub struct AppServices {
    inner: Arc<AppServicesInner>,
}

/// 被 `AppServices` 持有的服务聚合。
///
/// 各服务以 `Arc<T>` 持有，便于便捷访问函数直接对外暴露共享句柄；
/// 后续新增服务只需在此加一个 `Arc<XxxService>` 字段，并在宿主状态或适配器
/// 中通过 [`AppServices`] 访问，无需引入新的全局访问函数。
pub struct AppServicesInner {
    background_started: AtomicBool,
    /// 服务器备份管理服务。
    pub backup: Arc<CoreBackupService>,
    /// 下载任务管理服务。
    pub download: Arc<CoreDownloadService>,
    /// 服务器实例记录管理服务。
    pub instance: Arc<CoreInstanceService>,
    /// Java 环境检测与校验服务。
    pub java: Arc<CoreJavaService>,
    /// 在线隧道服务。
    pub online_tunnel: Arc<CoreOnlineTunnelService>,
    /// 服务器目录（类型 / 版本 / 下载详情）服务。
    pub catalog: Arc<CoreServerCatalogService>,
    /// 服务端检查与供给计划服务。
    pub provisioning: Arc<CoreProvisioningService>,
    /// 服务器进程管理服务。
    pub server: Arc<CoreServerService>,
    /// 服务器配置（server.properties）服务。
    pub server_config: Arc<CoreServerConfigService>,
    /// 服务器控制台日志服务。
    pub console: Arc<CoreConsoleService>,
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
    /// 应用更新安装服务。
    pub update_install: Arc<CoreUpdateInstallService>,
    /// 惰性初始化的应用插件服务。
    plugin: tokio::sync::OnceCell<Arc<CorePluginService>>,
}

impl AppServices {
    /// 从既有实例构造句柄（供宿主装配和测试注入）。
    ///
    /// 服务器进程服务共享同一实例服务句柄；下载/定时任务/系统资源服务自动构造。
    pub fn from_inner(instance: CoreInstanceService) -> Self {
        let instance = Arc::new(instance);
        let settings = Arc::new(CoreSettingsService::new());
        let server = Arc::new(CoreServerService::new(instance.clone(), settings.clone()));
        Self {
            inner: Arc::new(AppServicesInner {
                background_started: AtomicBool::new(false),
                backup: Arc::new(CoreBackupService::new(instance.clone(), server.clone())),
                download: Arc::new(CoreDownloadService::new()),
                console: Arc::new(CoreConsoleService::new(instance.clone())),
                cron: Arc::new(CoreCronTaskService::new(server.clone())),
                system: Arc::new(CoreSystemService::new(instance.clone(), server.clone())),
                server,
                server_config: Arc::new(CoreServerConfigService),
                instance,
                java: Arc::new(CoreJavaService),
                online_tunnel: Arc::new(CoreOnlineTunnelService::default()),
                catalog: Arc::new(CoreServerCatalogService),
                provisioning: Arc::new(CoreProvisioningService),
                settings,
                proxy_monitoring: Arc::new(ProxyMonitoringService::new()),
                update: Arc::new(CoreUpdateCheckService::new()),
                update_install: Arc::new(CoreUpdateInstallService),
                plugin: tokio::sync::OnceCell::new(),
            }),
        }
    }

    /// 异步构造应用服务句柄，并启动由容器拥有的后台服务。
    ///
    /// 每个宿主的 composition root 只应调用一次；这是一个为当前宿主创建
    /// 独立服务图的构造器，不是进程级共享入口。构造后的句柄应通过宿主
    /// 状态显式传给 handler / command，而不是在业务代码中重复构造。
    pub async fn build() -> Result<Self, InstanceError> {
        sealantern_feature::config::data_migration::run_startup_migration().await?;
        let services = Self::from_inner(CoreInstanceService::new().await?);
        services.start_background_services().await;
        Ok(services)
    }

    /// 访问服务器备份管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn backup(&self) -> &Arc<CoreBackupService> {
        &self.inner.backup
    }

    /// 访问下载任务管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn download(&self) -> &Arc<CoreDownloadService> {
        &self.inner.download
    }

    /// 访问实例管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn instance(&self) -> &Arc<CoreInstanceService> {
        &self.inner.instance
    }

    /// 访问 Java 环境检测与校验服务（`Arc` 共享句柄，clone 廉价）。
    pub fn java(&self) -> &Arc<CoreJavaService> {
        &self.inner.java
    }

    /// 访问在线隧道服务（`Arc` 共享句柄，clone 廉价）。
    pub fn online_tunnel(&self) -> &Arc<CoreOnlineTunnelService> {
        &self.inner.online_tunnel
    }

    /// 访问服务器目录服务（`Arc` 共享句柄，clone 廉价）。
    pub fn catalog(&self) -> &Arc<CoreServerCatalogService> {
        &self.inner.catalog
    }

    /// 访问服务端检查与供给计划服务。
    pub fn provisioning(&self) -> &Arc<CoreProvisioningService> {
        &self.inner.provisioning
    }

    /// 访问服务器进程管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn server(&self) -> &Arc<CoreServerService> {
        &self.inner.server
    }

    /// 访问服务器配置（server.properties）服务（`Arc` 共享句柄，clone 廉价）。
    pub fn server_config(&self) -> &Arc<CoreServerConfigService> {
        &self.inner.server_config
    }

    /// 访问服务器控制台日志服务（`Arc` 共享句柄，clone 廉价）。
    pub fn console(&self) -> &Arc<CoreConsoleService> {
        &self.inner.console
    }

    /// 访问设置信息服务（`Arc` 共享句柄，clone 廉价）。
    pub fn settings(&self) -> &Arc<CoreSettingsService> {
        &self.inner.settings
    }

    /// 加载持久化设置并同步全局网络运行时。
    ///
    /// 代理同步失败会在返回的错误中体现（宿主决定是否降级启动），
    /// 但系统代理轮询始终启动：设置初始化失败仅表示网络运行时尚未
    /// 按持久化设置同步，此时使用默认直连；系统代理恢复后由轮询或
    /// 后续设置操作的重试自动跟上。
    pub async fn initialize_network_settings(
        &self,
    ) -> Result<(), sealantern_contract::SettingsServiceError> {
        let settings_result = self.inner.settings.initialize().await;
        if self.inner.proxy_monitoring.start().await {
            tracing::info!(
                target: "sealantern.application.proxy_monitoring",
                poll_interval_seconds = 3,
                "system proxy monitoring started"
            );
        }
        settings_result
    }

    /// 停止由服务容器启动的后台任务并关闭活动在线隧道。
    ///
    /// 所有步骤都是幂等的；Cron 和代理监控的停止不会失败，在线隧道的
    /// 底层错误则返回给宿主记录。宿主退出前应调用此方法，而不是依赖
    /// 丢弃句柄触发后台任务结束。
    pub async fn shutdown(&self) -> Result<(), sealantern_contract::OnlineTunnelServiceError> {
        self.inner.cron.stop_scheduler().await;
        self.inner.proxy_monitoring.stop().await;
        self.inner.online_tunnel.shutdown().await
    }

    /// 访问服务器定时任务服务（`Arc` 共享句柄，clone 廉价）。
    pub fn cron(&self) -> &Arc<CoreCronTaskService> {
        &self.inner.cron
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

    /// 访问应用更新检查服务（`Arc` 共享句柄，clone 廉价）。
    pub fn update(&self) -> &Arc<CoreUpdateCheckService> {
        &self.inner.update
    }

    /// 访问应用更新安装服务（`Arc` 共享句柄，clone 廉价）。
    pub fn update_install(&self) -> &Arc<CoreUpdateInstallService> {
        &self.inner.update_install
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
}
