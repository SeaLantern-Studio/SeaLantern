//! `sealantern-server` 服务器二进制 crate。
//!
//! 启动 HTTP 服务：装配全局服务 → 组合路由 → 绑定监听 → 提供服务，
//! 并在收到终止信号时优雅关闭。

use std::net::SocketAddr;

use sealantern_application::services::AppServices;
use sealantern_server::adapter::http::build_router;
use sealantern_server::observability;

/// 监听地址环境变量；未设置时回退到本机 3000 端口。
const SERVER_ADDR_ENV: &str = "SEALANTERN_SERVER_ADDR";
/// 默认监听地址（仅本机访问）。
const DEFAULT_ADDR: &str = "127.0.0.1:3000";

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

    let app = build_router(services);

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
