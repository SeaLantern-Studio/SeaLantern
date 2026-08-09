//! HTTP 应用状态。
//!
//! 持有 [`AppServices`]（application 全局单例的轻量克隆句柄），handler 与中间件
//! 经此访问各服务。web 特有服务（如下载任务管理）也在此按需组装，但数据源一律
//! 走 application 的全局装配逻辑。
//!
//! `AppState` 只随 HTTP 层会话 / 认证等状态扩展字段，不随服务数量膨胀——
//! 新增服务只需往 `AppServicesInner` 加字段，本状态无需改动。

use std::sync::Arc;

use sealantern_application::service::{
    CoreCronTaskService, CoreInstanceService, CoreServerService, CoreSettingsService,
    CoreSystemService,
};
use sealantern_application::services::AppServices;

/// HTTP 层的共享应用状态。
///
/// `Clone` 是 axum 对 `State` 的要求（每次请求提取时克隆，成本仅为一个 `Arc`）。
#[derive(Clone)]
pub struct AppState {
    services: AppServices,
}

impl AppState {
    /// 从全局服务句柄构造应用状态。
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }

    /// 访问实例记录管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn instance(&self) -> Arc<CoreInstanceService> {
        self.services.instance().clone()
    }

    /// 访问服务器进程管理服务（`Arc` 共享句柄，clone 廉价）。
    pub fn server(&self) -> Arc<CoreServerService> {
        self.services.server().clone()
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
}
