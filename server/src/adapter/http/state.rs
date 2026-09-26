//! HTTP 应用状态。
//!
//! 持有宿主显式注入的 [`AppServices`] 轻量克隆句柄，handler 与中间件经此访问各服务。
//! web 特有服务（如下载任务管理）也在此按需组装，但数据源一律走 application 的装配逻辑。
//!
//! `AppState` 只随 HTTP 层会话 / 认证等状态扩展字段，不随服务数量膨胀——
//! 新增服务只需往 `AppServicesInner` 加字段，本状态无需改动。

use std::sync::Arc;

use sealantern_application::service::{
    CoreConsoleService, CoreCronTaskService, CoreDownloadService, CoreInstanceService,
    CoreProvisioningService, CoreServerService, CoreSettingsService, CoreSystemService,
    CoreUpdateCheckService,
};
use sealantern_application::services::AppServices;

use crate::event::ServerEventBus;

/// HTTP 层的共享应用状态。
///
/// `Clone` 是 axum 对 `State` 的要求（每次请求提取时克隆，成本仅为一个 `Arc`）。
#[derive(Clone)]
pub struct AppState {
    services: AppServices,
    events: Arc<ServerEventBus>,
}

impl AppState {
    /// 从宿主注入的服务句柄与事件总线构造应用状态。
    pub fn new(services: AppServices, events: Arc<ServerEventBus>) -> Self {
        Self { services, events }
    }

    /// 返回共享应用服务容器，供需要异步初始化子服务的传输适配器使用。
    pub fn services(&self) -> &AppServices {
        &self.services
    }

    /// 访问事件总线（供事件推送的传输适配器订阅）。
    pub fn events(&self) -> &Arc<ServerEventBus> {
        &self.events
    }

    /// 访问实例记录管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn instance(&self) -> Arc<CoreInstanceService> {
        self.services.instance().clone()
    }

    /// 访问服务器进程管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn server(&self) -> Arc<CoreServerService> {
        self.services.server().clone()
    }

    /// 访问服务器控制台日志服务（`Arc` 共享句柄，clone 廉价）。
    pub fn console(&self) -> Arc<CoreConsoleService> {
        self.services.console().clone()
    }

    /// 访问设置信息服务（`Arc` 共享句柄，clone 廉价）。
    pub fn settings(&self) -> Arc<CoreSettingsService> {
        self.services.settings().clone()
    }

    /// 访问服务器定时任务服务（`Arc` 共享句柄，clone 廉价）。
    pub fn cron(&self) -> Arc<CoreCronTaskService> {
        self.services.cron().clone()
    }

    /// 访问系统资源信息服务（`Arc` 共享句柄，clone 廉价）。
    pub fn system(&self) -> Arc<CoreSystemService> {
        self.services.system().clone()
    }

    /// 访问应用更新检查服务（`Arc` 共享句柄，clone 廉价）。
    pub fn update(&self) -> Arc<CoreUpdateCheckService> {
        self.services.update().clone()
    }

    /// 访问下载任务管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn download(&self) -> Arc<CoreDownloadService> {
        self.services.download().clone()
    }

    /// 访问服务端检查与供给计划服务（`Arc` 共享句柄，clone 廉价）。
    pub fn provisioning(&self) -> Arc<CoreProvisioningService> {
        self.services.provisioning().clone()
    }
}

/// 允许 handler 只提取事件总线，而不是整个 [`AppState`]。
///
/// 事件模块不依赖传输层状态；通过该转换让 WebSocket handler 保持窄依赖。
impl axum::extract::FromRef<AppState> for Arc<ServerEventBus> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.events)
    }
}
