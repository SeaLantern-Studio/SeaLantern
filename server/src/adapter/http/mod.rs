//! HTTP 传输适配层。
//!
//! 为服务器能力（当前为实例管理）提供 REST 风格 HTTP 接口，作为 `rpc` 之外
//! 的另一条宿主通道。`rpc` 将被逐步迁移至此，最终整体移除。

pub mod error;
pub mod handlers;
pub mod router;
pub mod state;

pub use error::HttpError;
pub use router::build_router;
pub use state::AppState;
