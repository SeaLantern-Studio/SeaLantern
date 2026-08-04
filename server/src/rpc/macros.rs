//! RPC 路由注册的便捷宏。

/// 批量注册 RPC 方法到 Axum 路由。
///
/// 将一组方法及其对应的服务字段名写入路由注册表，展开为多次 `rpc_route!` 调用。
///
/// # 格式
///
/// ```ignore
/// register_methods!(router, services, [
///     (path::to::Method, service_field),
/// ]);
/// ```
///
/// `service_field` 是 [`RpcServices`] 的字段名，宏展开后等价于：
///
/// ```ignore
/// rpc_route!(router, path::to::Method::new(services.service_field.clone()));
/// ```
///
/// [`RpcServices`]: crate::rpc::service::RpcServices
#[macro_export]
macro_rules! register_methods {
    ($router:ident, $services:ident, [ $( ($method:ty, $field:ident) ),* $(,)? ]) => {
        $(
            $crate::rpc_route!($router, <$method>::new($services.$field.clone()));
        )*
    };
}
