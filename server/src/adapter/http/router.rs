//! HTTP 路由总入口。
//!
//! 组装当前所有 REST 路由与前端 SPA 路由，并挂载应用状态。调用方构建
//! [`AppState`] 与 [`ViteConfig`] 后传入 [`build_router`]，返回的 [`Router`]
//! 可直接嵌套进更大应用或启动监听。

use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use axum_vite::{spa_router, ViteConfig};

use sealantern_application::services::AppServices;

use super::handlers;
use super::state::AppState;

/// REST API 路径前缀。
const API_PREFIX: &str = "/api";

/// 组装当前所有已实现 REST 路由与前端 SPA 路由的 Axum 应用。
///
/// - REST 路由挂载在 `/api` 前缀下，采用资源风格，并预留 `/{id}/xxx` 嵌套
///   子资源（如状态、日志流）的挂载位置，后续按需在此扩展。
/// - SPA 路由由 [`spa_router`] 提供，挂载在根路径（`/` 与 catch-all），
///   dev 模式下代理到 Vite dev server，release 模式从内嵌静态资源提供服务。
///
/// 调用方需保证 [`ViteConfig`] 的生命周期覆盖整个进程（dev server 手柄由
/// 调用方持有，drop 时会终止 vite 子进程）。
pub fn build_router(services: AppServices, config: ViteConfig) -> Router {
    let state = AppState::new(services);

    let instance_routes = Router::new()
        .route("/instances", get(handlers::list_instances))
        .route("/instances", post(handlers::create_instance))
        .route("/instances/{id}", get(handlers::get_instance))
        .route("/instances/{id}", delete(handlers::delete_instance))
        .route("/instances/{id}", patch(handlers::rename_instance))
        // ── 嵌套子资源（服务器进程生命周期） ──
        .route("/instances/{id}/status", get(handlers::server_status))
        .route("/instances/{id}/start", post(handlers::start_server))
        .route("/instances/{id}/stop", post(handlers::stop_server))
        .route(
            "/instances/{id}/force-stop",
            post(handlers::force_stop_server),
        )
        .route(
            "/instances/{id}/command",
            post(handlers::send_server_command),
        )
        // ── 嵌套子资源（后续扩展） ──
        // 示例：.route("/instances/{id}/logs", get(handlers::instance_logs))
        .route("/instances/{id}/path", put(handlers::update_instance_path));

    let settings_routes = Router::new().route("/settings", get(handlers::settings_overview));

    let system_routes = Router::new()
        .route("/system", get(handlers::system_snapshot))
        .route("/system/process/{pid}", get(handlers::process_usage))
        .route("/system/directory/{*path}", get(handlers::directory_usage));

    Router::new()
        .nest(API_PREFIX, instance_routes)
        .nest(API_PREFIX, settings_routes)
        .nest(API_PREFIX, system_routes)
        .merge(spa_router(config))
        .with_state(state)
}
