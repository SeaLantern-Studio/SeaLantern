//! 玩家查询服务契约。
//!
//! 提供按用户名查询 Minecraft 玩家档案的宿主能力端口。

mod services;

pub use services::{PlayerLookupService, PlayerProfile};
