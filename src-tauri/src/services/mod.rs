//! 宿主能力实现。
//!
//! 用 `core` / `extra` 的能力实现 `server` crate 定义的 service trait
//! （`AppServices` 各字段对应的宿主端口）。每个 service trait 对应一个文件，
//! 例如 `instance.rs` 实现 `server::rpc::traits::InstanceService`。
//!
//! `adapter` 负责传输适配，`services` 负责能力实现，两者方向不同、各归其位。
//! `app_service.rs` 承载应用级自托管容器。

pub mod app_service;
pub mod instance;
pub mod rpc;

pub use app_service::AppServices;
pub use instance::CoreInstanceService;
