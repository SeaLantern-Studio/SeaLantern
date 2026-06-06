pub(crate) mod execute;
pub(crate) mod pending;

use std::sync::atomic::AtomicBool;

/// 安装进度标志
#[allow(dead_code)] // 发布调用
pub static INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
