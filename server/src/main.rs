//! SeaLantern 独立 HTTP 服务器入口。

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sealantern_server::rpc::router::build_router;
use sealantern_server::rpc::service::NoOpConsoleService;

/// 全放行访问解析器（预览版暂不实现认证）。
struct AllowAllAccessResolver;

impl sealantern_server::rpc::axum::HttpRpcAccessResolver for AllowAllAccessResolver {
    fn resolve(
        &self,
        _headers: &axum::http::HeaderMap,
    ) -> sealantern_server::rpc::RpcResult<sealantern_server::rpc::RpcAccess> {
        Ok(sealantern_server::rpc::RpcAccess::allow_all())
    }
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 静态文件目录（可通过环境变量配置）
    let static_dir: PathBuf = env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./dist"));

    tracing::info!("Serving static files from {:?}", static_dir);

    // 创建 RPC 服务容器
    let services = sealantern_server::rpc::service::RpcServices::new(Arc::new(NoOpConsoleService));

    // 构建 Axum 路由
    let app = Router::new()
        // 健康检查端点
        .route("/health", get(|| async { "ok" }))
        // RPC API 路由
        .nest("/api", build_router(services, AllowAllAccessResolver))
        // 前端静态文件服务（fallback）
        .fallback_service(
            tower_http::services::ServeFile::new(&static_dir.join("index.html"))
        );

    // 监听地址
    let addr: SocketAddr = "0.0.0.0:3000".parse().expect("valid socket address");

    tracing::info!("SeaLantern HTTP server listening on {addr}");

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    // 优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server failed");
}

/// 优雅关闭信号。
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down gracefully...");
}