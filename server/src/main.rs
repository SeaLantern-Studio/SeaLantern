//! `sealantern-server` 服务器二进制 crate。
//!
//! 启动 HTTP 服务：装配应用服务 → 组合路由 → 绑定监听 → 提供服务，
//! 并在收到终止信号时优雅关闭。

use std::net::SocketAddr;

use sealantern_application::services::AppServices;
use sealantern_server::adapter::http::build_router;
use sealantern_server::observability;

/// axctl dev 注入的后端监听地址（托管模式：本进程只提供 API）。
const AXCTL_BACKEND_ADDR_ENV: &str = "AXCTL_BACKEND_ADDR";
/// 监听地址环境变量；设置后完全覆盖默认地址选择。
const SERVER_ADDR_ENV: &str = "SEALANTERN_SERVER_ADDR";
/// 设置该环境变量为 `1` 时默认监听所有网卡（公网可达）；否则默认仅本机。
const SERVER_BIND_PUBLIC_ENV: &str = "SEALANTERN_SERVER_BIND_PUBLIC";
/// 默认监听地址（仅本机访问，安全兜底）。
const DEFAULT_ADDR: &str = "127.0.0.1:3000";
/// 公网绑定时的默认监听地址。
const DEFAULT_PUBLIC_ADDR: &str = "0.0.0.0:3000";

/// 解析监听地址：axctl 注入 > SEALANTERN_SERVER_ADDR > 默认（公网/本机）。
fn listen_addr() -> SocketAddr {
    if let Ok(value) = std::env::var(AXCTL_BACKEND_ADDR_ENV) {
        // 注入地址无效视为致命错误，不回退（默认地址通常是代理端口）
        return match value.parse() {
            Ok(addr) => addr,
            Err(error) => {
                tracing::error!(
                    env = AXCTL_BACKEND_ADDR_ENV,
                    value = %value,
                    error = %error,
                    "invalid axctl backend address"
                );
                std::process::exit(1);
            }
        };
    }
    if let Ok(value) = std::env::var(SERVER_ADDR_ENV) {
        return match value.parse() {
            Ok(addr) => addr,
            Err(_) => {
                tracing::error!(
                    env = SERVER_ADDR_ENV,
                    value = %value,
                    "invalid listen address, falling back to default"
                );
                default_addr()
            }
        };
    }
    default_addr()
}

/// 默认监听地址：`SEALANTERN_SERVER_BIND_PUBLIC=1` 时监听所有网卡，否则仅本机。
fn default_addr() -> SocketAddr {
    let public = std::env::var(SERVER_BIND_PUBLIC_ENV)
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    let raw = if public {
        DEFAULT_PUBLIC_ADDR
    } else {
        DEFAULT_ADDR
    };
    raw.parse().expect("default address must be valid")
}

/// 服务器入口。
#[tokio::main]
pub async fn main() {
    observability::init();

    let services = match AppServices::build().await {
        Ok(services) => services,
        Err(error) => {
            tracing::error!(error = %error, "failed to assemble application services");
            std::process::exit(1);
        }
    };
    if let Err(error) = services.initialize_network_settings().await {
        // 网络设置同步失败不阻止启动：网络运行时保持默认直连，
        // 系统代理恢复后由轮询与后续设置操作的重试自动跟上。
        tracing::error!(
            error = %error,
            "failed to initialize persisted network settings; continuing with direct network"
        );
    }

    // 前端资源：release 编译期内嵌 dist；debug 为空包装（dev 前端由 vite 提供，
    // axctl 代理统一入口）。
    let app = build_router(services.clone(), axctl_core::frontend!("$CARGO_MANIFEST_DIR/../dist"));

    let addr = listen_addr();

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%addr, error = %error, "failed to bind TCP listener");
            if let Err(shutdown_error) = services.shutdown().await {
                tracing::error!(
                    error = %shutdown_error,
                    "failed to shut down application services after bind failure"
                );
            }
            std::process::exit(1);
        }
    };
    tracing::info!(%addr, "server listening");

    let serve_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;
    if let Err(error) = &serve_result {
        tracing::error!(error = %error, "server terminated with error");
    }
    if let Err(error) = services.shutdown().await {
        tracing::error!(error = %error, "failed to shut down application services");
    }
    if serve_result.is_err() {
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
