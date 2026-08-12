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

/// 监听地址环境变量；设置后完全覆盖默认地址选择。
const SERVER_ADDR_ENV: &str = "SEALANTERN_SERVER_ADDR";
/// 设置该环境变量为 `1` 时默认监听所有网卡（公网可达）；否则默认仅本机。
const SERVER_BIND_PUBLIC_ENV: &str = "SEALANTERN_SERVER_BIND_PUBLIC";
/// 默认监听地址（仅本机访问，安全兜底）。
const DEFAULT_ADDR: &str = "127.0.0.1:3000";
/// 公网绑定时的默认监听地址。
const DEFAULT_PUBLIC_ADDR: &str = "0.0.0.0:3000";

/// 解析监听地址：优先 `SEALANTERN_SERVER_ADDR`，其次按是否开启公网绑定
/// 选择默认地址，最后回退到仅本机监听。
fn listen_addr() -> SocketAddr {
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

    let services = match AppServices::get().await {
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

    // 构建 Vite 配置并（在 dev 模式下）拉起 dev server。
    // 手柄必须在此持有，drop 时会终止 vite 子进程。
    let vite_config = vite_config();
    let _vite_dev_server = vite_config.maybe_spawn_dev_server();

    let app = build_router(services, vite_config);

    let addr = listen_addr();

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
