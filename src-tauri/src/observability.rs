//! 桌面端可观测性初始化。

use std::sync::Once;

use tracing_subscriber::EnvFilter;

static INIT_TRACING: Once = Once::new();

/// 初始化 tracing 日志。
///
/// 通过 `RUST_LOG` 环境变量控制日志等级与目标过滤
/// （未设置时默认 `sealantern=info`，即仅输出本项目各 crate 的日志），
/// 以人类可读格式输出到标准输出。由 `Once` 保证幂等，多次调用安全。
pub fn init() {
    INIT_TRACING.call_once(|| {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("sealantern=info"));
        tracing_subscriber::fmt().with_env_filter(filter).init();
    });
}
