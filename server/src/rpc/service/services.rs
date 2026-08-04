//! 所有可用的 RPC 宿主服务容器。

use std::sync::Arc;

use super::console::ConsoleCommandService;

/// 所有可用的 RPC 宿主服务。
///
/// 每个字段对应一类宿主能力端口，由 [`build_router`] 消费后注入各 RPC 方法实例。
/// 新增模块时在此结构体添加字段即可，无需修改路由注册函数签名。
///
/// [`build_router`]: crate::rpc::router::build_router
pub struct RpcServices {
    /// 服务器控制台命令服务。
    pub console: Arc<dyn ConsoleCommandService>,
}

impl RpcServices {
    /// 创建 RPC 服务容器。
    pub fn new(console: Arc<dyn ConsoleCommandService>) -> Self {
        Self { console }
    }
}
