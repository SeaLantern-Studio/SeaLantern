//! download 子模块：统一下载/安装相关服务逻辑。
//!
//! - download_manager.rs：多线程下载任务管理（前端“下载管理器”）。
//! - java_installer.rs：Java 运行时下载与安装（带进度与取消）。
//! - starter_installer_links.rs：Starter 模式服务器核心下载链接的获取与缓存。

pub(crate) mod common;
pub(crate) mod download_manager;
pub(crate) mod java_installer;
#[cfg(test)]
pub(crate) mod starter_installer_links;
