//! 传输无关的宿主能力端口（trait），独立于实现。
//!
//! 本目录只定义接口契约与关联错误类型，供 `service`（实现）、`methods`、
//! 及各宿主胶水层引用。后续可将本目录整体迁往 `crates/application` 契约层。

pub mod console;
pub mod instance;

pub use console::{ConsoleCommandExecutor, ConsoleCommandService, ConsoleCommandServiceError};
pub use instance::{InstanceService, InstanceServiceError};
