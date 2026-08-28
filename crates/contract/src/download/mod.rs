//! 下载任务管理契约模型。
//!
//! 提供下载任务创建、进度查询与取消的宿主能力端口，供 tauri / server 等宿主
//! 统一消费。

mod models;

pub use models::{DownloadRequest, DownloadTaskInfo, DownloadTaskStatus};
