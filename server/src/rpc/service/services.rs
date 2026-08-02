//! 所有可用的 RPC 宿主服务容器。

use std::sync::Arc;

use crate::rpc::traits::console::ConsoleCommandService;
use crate::rpc::traits::instance::InstanceService;

/// 所有可用的 RPC 宿主服务。
///
/// 每个字段对应一类宿主能力端口，由 [`build_router`] 消费后注入各 RPC 方法实例。
/// 新增模块时在此结构体添加字段即可，无需修改路由注册函数签名。
///
/// [`build_router`]: crate::rpc::router::build_router
pub struct RpcServices {
    /// 服务器控制台命令服务。
    pub console: Arc<dyn ConsoleCommandService>,
    /// 服务器实例管理服务。
    pub instance: Arc<dyn InstanceService>,
}

impl RpcServices {
    /// 创建 RPC 服务容器。
    pub fn new(
        console: Arc<dyn ConsoleCommandService>,
        instance: Arc<dyn InstanceService>,
    ) -> Self {
        Self { console, instance }
    }
}
