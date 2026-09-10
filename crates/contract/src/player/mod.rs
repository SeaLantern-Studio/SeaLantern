//! 玩家查询契约模型。
//!
//! 提供玩家档案与玩家列表（白名单 / 封禁 / OP）DTO，供 tauri / server
//! 等宿主统一消费，不依赖任何具体实现层；对应的服务端口位于
//! `sealantern-application::port::players`。

mod models;

pub use models::{BanEntryDto, OpEntryDto, PlayerEntryDto, PlayerProfile};
