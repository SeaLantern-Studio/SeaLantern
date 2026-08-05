//! `sealantern-server` 服务器二进制 crate。
//!
//! 启动 HTTP 服务：装配全局服务 → 组合路由 → 绑定监听 → 提供服务，
//! 并在收到终止信号时优雅关闭。

use std::net::SocketAddr;
use std::path::PathBuf;

use axum_vite::ViteConfig;
use sealantern_application::services::AppServices;
use sealantern_server::adapter::http::build_router;
use sealantern_server::observability;

/// 监听地址环境变量；未设置时回退到本机 3000 端口。
const SERVER_ADDR_ENV: &str = "SEALANTERN_SERVER_ADDR";
/// 默认监听地址
const DEFAULT_ADDR: &str = "0.0.0.0:3000";

/// 服务器入口。
#[tokio::main]
pub async fn main() {
    observability::init();

    let services = match AppServices::get().await {
        Ok(services) => services,
        Err(error) => {
            tracing::error!(error = %error, "failed to assemble application services");
            std::process::exit(1);
        }
    };

    // 构建 Vite 配置并（在 dev 模式下）拉起 dev server。
    // 手柄必须在此持有，drop 时会终止 vite 子进程。
    let vite_config = vite_config();
    let _vite_dev_server = vite_config.maybe_spawn_dev_server();

    let app = build_router(services, vite_config);

    let addr: SocketAddr = match std::env::var(SERVER_ADDR_ENV) {
        Ok(value) => match value.parse() {
            Ok(addr) => addr,
            Err(_) => {
                tracing::error!(
                    env = SERVER_ADDR_ENV,
                    value = %value,
                    "invalid listen address, falling back to {DEFAULT_ADDR}"
                );
                DEFAULT_ADDR.parse().expect("default address must be valid")
            }
        },
        Err(_) => DEFAULT_ADDR.parse().expect("default address must be valid"),
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%addr, error = %error, "failed to bind TCP listener");
            std::process::exit(1);
        }
    };
    tracing::info!(%addr, "server listening");

    if let Err(error) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(error = %error, "server terminated with error");
        std::process::exit(1);
    }
    tracing::info!("server shut down gracefully");
}

/// 等待终止信号（Ctrl+C 或 SIGTERM），返回时触发优雅关闭。
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}

/// 构建 Vite 开发配置。
///
/// 前端项目根默认为仓库根目录（`server` 的父目录，即 `package.json` 所在处）；
/// 可通过 `VITE_ROOT` 覆盖。dev 模式下默认自动拉起 vite dev server
/// （`VITE_AUTO_START=false` 可关闭），release 模式不拉起、改为服务内嵌静态资源。
fn vite_config() -> ViteConfig {
    let mut config = ViteConfig::from_env(axum_vite::embedded_dir!("$CARGO_MANIFEST_DIR/../dist"));

    // 未显式设置前端根目录时，默认仓库根。
    if config.frontend_root.is_none() {
        config.frontend_root = Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
    }
    // 未显式设置 dev host 时，使用 127.0.0.1 而非 localhost——
    // 后者在部分平台解析为 IPv6 (::1)，而 vite 默认只监听 IPv4，会导致代理连不上。
    if std::env::var_os("VITE_DEV_HOST").is_none() {
        config.dev_host = "127.0.0.1".to_string();
    }
    // 未显式设置 auto_start 时，dev 模式默认自动拉起 vite。
    if std::env::var_os("VITE_AUTO_START").is_none() {
        config.auto_start = true;
    }

    config
}
