//! 最小 RCON 客户端。
//!
//! 用途单一：向运行中的 Minecraft 服务器发命令并把回显拿回来。Nukkit / Paper /
//! 原版都实现了同一套 RCON 协议，所以不依赖任何服务端插件。
//!
//! 协议（每包都是小端）：
//!
//! ```text
//! 长度(i32) | 请求号(i32) | 类型(i32) | 负载(ASCIIZ) | 0x00
//! ```
//!
//! 认证走类型 3，命令走类型 2，响应是类型 0。认证失败时服务端回的请求号是 -1。
//!
//! 不引入第三方 rcon crate：协议本身不到 200 行，自己写可以控制超时和
//! 「多条响应包何时算收完」这个细节，也少一个依赖。

mod client;
mod query;

pub use client::{RconClient, RconError};
pub use query::{
    build_positions_command, build_rotation_command, parse_positions, parse_rotations,
};
