//! Java 检测与校验契约模型。
//!
//! Java 检测能力端口定义在 `sealantern_application::port::java`。

mod models;

pub use models::{JavaDetectionReport, JavaDiscoveryError, JavaInfo};
