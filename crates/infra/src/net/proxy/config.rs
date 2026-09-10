//! 网络代理配置模型由 `sealantern-contract` 公开拥有。
//!
//! 保留本模块的重导出，避免基础设施内部的代理运行时模块承担一次无意义的
//! 路径迁移；配置模型本身不依赖基础设施实现。

pub use sealantern_contract::proxy::{ProxyConfigError, ProxyMode, ProxySettings};
