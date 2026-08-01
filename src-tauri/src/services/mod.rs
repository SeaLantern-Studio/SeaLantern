//! 宿主能力实现。
//!
//! 用 `core` / `extra` 的能力实现 `server` crate 定义的 service trait
//! （`RpcServices` 各字段对应的宿主端口）。每个 service trait 对应一个文件，
//! 例如 `instance.rs` 实现 `server::rpc::service::InstanceService`。
//!
//! `adapter` 负责传输适配，`services` 负责能力实现，两者方向不同、各归其位。

pub mod instance;
