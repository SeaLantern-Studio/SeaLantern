//! HTTP 路由总入口。
//!
//! 组装当前所有 REST 路由与前端 SPA 路由，并挂载应用状态。调用方构建
//! [`AppState`] 与 [`ViteConfig`] 后传入 [`build_router`]，返回的 [`Router`]
//! 可直接嵌套进更大应用或启动监听。

use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use axum_vite::{ViteConfig, spa_router};

use sealantern_application::services::AppServices;

use crate::rpc::axum::AxumRpcState;
use crate::rpc::methods::plugin::InvokePluginCapability;
use crate::rpc::plugin_auth::PluginRpcTokenResolver;

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

    let mut plugin_rpc_routes = Router::new();
    crate::rpc_route!(plugin_rpc_routes, InvokePluginCapability::new(state.services().clone()));
    let plugin_rpc_routes = plugin_rpc_routes.with_state(AxumRpcState {
        access_resolver: std::sync::Arc::new(PluginRpcTokenResolver::from_env()),
    });

    let instance_routes = Router::new()
        .route("/instances", get(handlers::list_instances))
        .route("/instances", post(handlers::create_instance))
        .route("/instances/{id}", get(handlers::get_instance))
        .route("/instances/{id}", delete(handlers::delete_instance))
        .route("/instances/{id}", patch(handlers::rename_instance))
        // ── 嵌套子资源（服务器进程生命周期） ──
        .route("/instances/{id}/status", get(handlers::server_status))
        .route("/instances/{id}/start", post(handlers::start_server))
        .route("/instances/{id}/restart", post(handlers::restart_server))
        .route("/instances/{id}/stop", post(handlers::stop_server))
        .route(
            "/instances/{id}/force-stop",
            post(handlers::force_stop_server),
        )
        .route(
            "/instances/{id}/command",
            post(handlers::send_server_command),
        )
        .route("/instances/{id}/logs", get(handlers::console_logs))
        // ── 嵌套子资源（后续扩展） ──
        // 示例：.route("/instances/{id}/logs", get(handlers::instance_logs))
        .route("/instances/{id}/path", put(handlers::update_instance_path));

    let settings_routes = Router::new().route("/settings", get(handlers::settings_overview));

    let system_routes = Router::new()
        .route("/system", get(handlers::system_snapshot))
        .route("/system/default-run-path", get(handlers::default_run_path))
        .route("/system/servers/{instance_id}/usage", get(handlers::server_resource_usage));

    let cron_routes = Router::new()
        .route("/cron-tasks", get(handlers::list_cron_tasks))
        .route("/cron-tasks", post(handlers::create_cron_task))
        .route("/cron-tasks/{id}", put(handlers::update_cron_task))
        .route("/cron-tasks/{id}", delete(handlers::delete_cron_task))
        .route("/cron-tasks/{id}/enabled", put(handlers::set_cron_task_enabled))
        .route("/cron-tasks/{id}/run", post(handlers::run_cron_task));

    let update_routes = Router::new().route("/update", get(handlers::check_update));

    let provisioning_routes =
        Router::new().route("/provisioning/inspect", post(handlers::inspect_server));

    let download_routes = Router::new()
        .route("/downloads", post(handlers::create_download))
        .route("/downloads/{id}", get(handlers::query_download))
        .route("/downloads/{id}", axum::routing::delete(handlers::cancel_download));

    Router::new()
        .nest(API_PREFIX, instance_routes)
        .nest(API_PREFIX, settings_routes)
        .nest(API_PREFIX, system_routes)
        .nest(API_PREFIX, cron_routes)
        .nest(API_PREFIX, update_routes)
        .nest(API_PREFIX, provisioning_routes)
        .nest(API_PREFIX, download_routes)
        .merge(plugin_rpc_routes)
        .merge(spa_router(config))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use sealantern_application::service::CoreInstanceService;
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;

    use super::*;

    async fn test_router() -> (Router, TempDir) {
        let directory = tempdir().expect("create temporary directory");
        let instance = CoreInstanceService::with_path(directory.path().join("instances.json"))
            .await
            .expect("create instance service");
        (
            build_router(AppServices::from_inner(instance), ViteConfig::default()),
            directory,
        )
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn update_route_returns_snake_case_contract() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/update")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call update route");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .expect("read update response");
        let value: serde_json::Value =
            serde_json::from_slice(&body).expect("parse update response");
        assert!(value.get("has_update").is_some());
        assert!(value.get("latest_version").is_some());
        assert!(value.get("hasUpdate").is_none());
    }

    #[tokio::test]
    async fn update_route_rejects_post_requests() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::post("/api/update")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call update route");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn plugin_rpc_requires_a_valid_bearer_token() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::post("/api/rpc/plugin/v2/invoke")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("build request"),
            )
            .await
            .expect("call plugin RPC route");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().contains_key("x-request-id"));
    }
}
