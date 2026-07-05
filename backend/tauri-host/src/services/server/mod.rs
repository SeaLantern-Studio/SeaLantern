//! server 相关服务。
//!
//! 这里放服务器本体相关的后端代码。
//! 当前主要包括：
//! - manager.rs：服务器管理入口
//! - log_pipeline.rs：日志写入、读取和事件推送
//! - id_manager.rs：服务器 ID 相关逻辑
//! - player.rs：玩家名单和权限管理
//! - join.rs：加入服务器相关逻辑
//!
//! 旧的顶层别名大多已经删掉了。
//! 新代码直接使用 `services::server::*` 下的真实路径。

pub mod extensions;
#[cfg(test)]
pub(crate) mod flavor_support;
pub mod id_manager;
pub mod join;
pub mod log_pipeline;
pub mod manager;
pub mod player;
pub mod plugin_manager;
pub mod runtime;
