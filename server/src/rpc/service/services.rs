//! 所有可用的 RPC 宿主服务容器，与全局懒加载获取器。
//!
//! [`AppServices`] 作为服务容器持有各宿主能力端口（`Arc<dyn Trait>`），
//! 并提供与 `src-tauri` 侧同名容器一致的全局懒加载访问：
//! 惰性初始化、可替换、并发安全。当前只搭建容器与全局获取框架，具体服务实现由
//! 调用方在注册时注入（尚未内置任何默认实现）。

use std::sync::Arc;

use crate::rpc::traits::console::ConsoleCommandService;
use crate::rpc::traits::instance::InstanceService;

/// 进程级全局服务容器。惰性初始化，可替换。
static SERVICES: tokio::sync::RwLock<Option<Arc<AppServices>>> =
    tokio::sync::RwLock::const_new(None);

/// 所有可用的 RPC 宿主服务。
///
/// 每个字段对应一类宿主能力端口，由 [`build_router`] 消费后注入各 RPC 方法实例。
/// 新增模块时在此结构体添加字段即可，无需修改路由注册函数签名。
///
/// [`build_router`]: crate::rpc::router::build_router
pub struct AppServices {
    /// 服务器控制台命令服务。
    pub console: Arc<dyn ConsoleCommandService>,
    /// 服务器实例管理服务。
    pub instance: Arc<dyn InstanceService>,
}

impl AppServices {
    /// 创建 RPC 服务容器。
    pub fn new(
        console: Arc<dyn ConsoleCommandService>,
        instance: Arc<dyn InstanceService>,
    ) -> Self {
        Self { console, instance }
    }

    /// 访问服务器控制台命令服务。
    pub fn console(&self) -> Arc<dyn ConsoleCommandService> {
        self.console.clone()
    }

    /// 访问服务器实例管理服务。
    pub fn instance(&self) -> Arc<dyn InstanceService> {
        self.instance.clone()
    }

    /// 惰性获取全局服务。
    ///
    /// 首次调用时异步构造（由后续注入的默认实现或调用方注册）。
    /// 并发首次调用只初始化一次，其余等待并复用首个注册的结果。
    pub async fn get() -> Arc<Self> {
        // 快速路径：已初始化直接返回（读锁）。
        if let Some(existing) = SERVICES.read().await.as_ref() {
            return existing.clone();
        }

        // 注册：加写锁；若并发期间已有人注册，则复用其结果，丢弃本次构造。
        let guard = SERVICES.write().await;
        match guard.as_ref() {
            Some(existing) => existing.clone(),
            None => {
                // TODO: 后续在此接入默认服务实现；当前无实现，先占位不应到这里。
                unreachable!("AppServices 尚未注册默认构建方式，请先 register")
            }
        }
    }

    /// 显式注册全局服务（启动预热 / 测试注入 / 重载用），覆盖既有实例。
    pub async fn register(services: Self) -> Result<Arc<Self>, ()> {
        let services = Arc::new(services);
        *SERVICES.write().await = Some(services.clone());
        Ok(services)
    }

    /// 非阻塞尝试取全局服务；未初始化时返回 `None`。
    pub fn try_get() -> Option<Arc<Self>> {
        SERVICES.try_read().ok().and_then(|g| g.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopConsole;
    impl ConsoleCommandService for NoopConsole {
        fn send_console_command(
            &self,
            _instance_id: &str,
            _command: &str,
        ) -> Result<(), crate::rpc::traits::console::ConsoleCommandServiceError> {
            Ok(())
        }
    }

    struct NoopInstance;
    #[async_trait::async_trait]
    impl InstanceService for NoopInstance {
        async fn list(
            &self,
        ) -> Result<
            Vec<sealantern_core::instance::Instance>,
            crate::rpc::traits::instance::InstanceServiceError,
        > {
            Ok(Vec::new())
        }
        async fn find(
            &self,
            _id: &sealantern_core::instance::InstanceId,
        ) -> Result<
            Option<sealantern_core::instance::Instance>,
            crate::rpc::traits::instance::InstanceServiceError,
        > {
            Ok(None)
        }
        async fn status(
            &self,
            _id: &sealantern_core::instance::InstanceId,
        ) -> Result<
            sealantern_core::server::ServerStatus,
            crate::rpc::traits::instance::InstanceServiceError,
        > {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn start(
            &self,
            _id: &sealantern_core::instance::InstanceId,
        ) -> Result<(), crate::rpc::traits::instance::InstanceServiceError> {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn stop(
            &self,
            _id: &sealantern_core::instance::InstanceId,
        ) -> Result<(), crate::rpc::traits::instance::InstanceServiceError> {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn force_stop(
            &self,
            _id: &sealantern_core::instance::InstanceId,
        ) -> Result<(), crate::rpc::traits::instance::InstanceServiceError> {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn create(
            &self,
            _spec: sealantern_core::instance::InstanceSpec,
        ) -> Result<
            sealantern_core::instance::Instance,
            crate::rpc::traits::instance::InstanceServiceError,
        > {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn delete(
            &self,
            _id: &sealantern_core::instance::InstanceId,
        ) -> Result<bool, crate::rpc::traits::instance::InstanceServiceError> {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn rename(
            &self,
            _id: &sealantern_core::instance::InstanceId,
            _name: &str,
        ) -> Result<(), crate::rpc::traits::instance::InstanceServiceError> {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
        async fn update_path(
            &self,
            _id: &sealantern_core::instance::InstanceId,
            _path: &str,
        ) -> Result<(), crate::rpc::traits::instance::InstanceServiceError> {
            Err(crate::rpc::traits::instance::InstanceServiceError::Unsupported)
        }
    }

    #[tokio::test]
    async fn registers_and_retrieves_global_services() {
        let services = AppServices::new(Arc::new(NoopConsole), Arc::new(NoopInstance));
        let registered = AppServices::register(services).await.expect("register ok");

        assert!(Arc::ptr_eq(&registered, &AppServices::try_get().expect("some")));

        // 清理全局，避免污染其它测试
        *SERVICES.write().await = None;
    }

    #[tokio::test]
    async fn try_get_returns_none_before_registration() {
        *SERVICES.write().await = None;
        assert!(AppServices::try_get().is_none());
    }
}
