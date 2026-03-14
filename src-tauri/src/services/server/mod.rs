//! server 模块（Phase 1 / Phase 2）
//!
//! 领域聚合：收敛与服务器相关的核心服务，作为 services 的子模块。
//! 当前阶段迁移以下文件且保持对外 API 稳定：
//! - manager.rs（原 services/server_manager.rs）
//! - log_pipeline.rs（原 services/server_log_pipeline.rs）
//! - installer.rs（原 services/server_installer.rs）
//! - downloader.rs（原 services/server_downloader.rs）
//! - id_manager.rs（原 services/server_id_manager.rs）
//! - player.rs（原 services/player_manager.rs）
//! - config.rs（原 services/config_parser.rs）
//! - join.rs（原 services/join_manager.rs）
//!
//! 注意：为保持向后兼容，顶层 services 仍通过转发导出旧路径。

pub mod config;
pub mod downloader;
pub mod id_manager;
pub mod installer;
pub mod join;
pub mod log_pipeline;
pub mod manager;
pub mod player;
