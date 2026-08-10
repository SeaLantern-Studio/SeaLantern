//! RPC 方法所需的权限常量。
//!
//! 所有 RPC 方法模块的权限集中定义于此，便于审计和跨模块复用。
//! 新增权限时在此文件添加常量，各方法模块直接引用即可。

use crate::rpc::RpcPermission;

/// 向受管服务器控制台写入命令所需的 RPC 权限。
pub const PERMISSION_SERVER_CONSOLE_SEND: RpcPermission = RpcPermission::new("server.console.send");

/// 调用受 bearer 保护的插件 v2 能力入口所需的 RPC 权限。
pub const PERMISSION_PLUGIN_V2_INVOKE: RpcPermission = RpcPermission::new("plugin.v2.invoke");
