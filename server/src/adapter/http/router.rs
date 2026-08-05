//! HTTP 路由总入口。
//!
//! 组装当前所有 REST 路由并挂载应用状态。调用方构建 [`AppState`] 后传入
//! [`build_router`]，返回的 [`Router`] 可直接嵌套进更大应用或启动监听。

use axum::routing::{delete, get, patch, post, put};
use axum::Router;

use sealantern_application::services::AppServices;

use super::handlers;
use super::state::AppState;

/// REST API 路径前缀。
const API_PREFIX: &str = "/api";

/// 组装当前所有已实现 REST 路由的 Axum 应用。
///
/// 路由采用资源风格，并预留 `/{id}/xxx` 嵌套子资源（如状态、日志流）的
/// 挂载位置，后续按需在此扩展。
pub fn build_router(services: AppServices) -> Router {
    let state = AppState::new(services);

    let instance_routes = Router::new()
        .route("/instances", get(handlers::list_instances))
        .route("/instances", post(handlers::create_instance))
        .route("/instances/{id}", get(handlers::get_instance))
        .route("/instances/{id}", delete(handlers::delete_instance))
        .route("/instances/{id}", patch(handlers::rename_instance))
        // ── 嵌套子资源（后续扩展） ──
        // 示例：.route("/instances/{id}/status", get(handlers::instance_status))
        //       .route("/instances/{id}/logs", get(handlers::instance_logs))
        .route("/instances/{id}/path", put(handlers::update_instance_path));

    Router::new()
        .nest(API_PREFIX, instance_routes)
        .with_state(state)
}
