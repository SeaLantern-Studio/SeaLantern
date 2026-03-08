//! server 模块（Phase 1）
//!
//! 领域聚合：收敛与服务器相关的核心服务，作为 services 的子模块。
//! 当前阶段仅迁移以下文件且不改对外 API：
//! - manager.rs（原 services/server_manager.rs）
//! - log_pipeline.rs（原 services/server_log_pipeline.rs）
//! - installer.rs（原 services/server_installer.rs）
//!
//! 注意：为保持向后兼容，顶层 services 仍通过内联转发模块导出旧路径（server_manager、server_log_pipeline、server_installer）。

pub mod installer;
pub mod log_pipeline;
pub mod manager;
