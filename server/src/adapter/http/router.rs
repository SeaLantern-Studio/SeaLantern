//! HTTP 路由总入口。
//!
//! 组装当前所有 REST 路由与前端 SPA 路由，并挂载应用状态。调用方构建
//! [`AppState`] 与前端资源（`FrontendAssets`）后传入 [`build_router`]，
//! 返回的 [`Router`] 可直接嵌套进更大应用或启动监听。

use axctl_core::embed::FrontendAssets;
use axum::Router;
use axum::routing::{delete, get, patch, post, put};

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
/// - SPA fallback 由 [`axctl_core::serve::spa`] 提供：release 从内嵌静态资源
///   提供服务；debug 下前端由 vite 提供（axctl 代理统一入口），此处为空包装。
pub fn build_router(services: AppServices, assets: FrontendAssets<'static>) -> Router {
    let state = AppState::new(services);

    let mut plugin_rpc_routes = Router::new();
    crate::rpc_route!(plugin_rpc_routes, InvokePluginCapability::new(state.services().clone()));
    let plugin_rpc_routes = plugin_rpc_routes.with_state(AxumRpcState {
        access_resolver: std::sync::Arc::new(PluginRpcTokenResolver::from_env()),
    });

    let instance_routes = Router::new()
        .route("/instances", get(handlers::list_instances))
        .route("/instances/discovery", get(handlers::discover_instances))
        .route("/instances", post(handlers::create_instance))
        .route("/instances/import-existing", post(handlers::import_existing_instance))
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
        .route("/instances/{id}/path", put(handlers::update_instance_path))
        // ── 信任 / 忽略名单维护（按 id 定位，与 `{id}` 子资源同层） ──
        .route("/instances/{id}/trust", post(handlers::trust_instance))
        .route("/instances/{id}/untrust", post(handlers::untrust_instance));

    // 实例注册表维护（按目录定位）。
    //
    // 刻意放在独立前缀下：这些操作的标识符是**目录路径**而非实例 id，
    // 若挂在 `/instances/*` 下会与 `{id}` 同层共位——同一个路径段既是
    // 路由结构又是资源数据，取值撞名时（如 id 恰好是 `ignored-dirs`）
    // 静态段优先会让 DELETE 静默执行「解除忽略」而非「删除实例」。
    // 独立前缀消除了该歧义的结构前提。
    let instance_registry_routes = Router::new()
        .route("/instance-registry/ignored-dirs", post(handlers::ignore_instance_dir))
        .route("/instance-registry/ignored-dirs", delete(handlers::unignore_instance_dir))
        .route("/instance-registry/reassign-id", post(handlers::reassign_instance_id));

    let legacy_migration_routes = Router::new()
        .route("/legacy-migration", get(handlers::legacy_migration_status))
        .route("/legacy-migration", post(handlers::migrate_legacy_config))
        .route("/legacy-migration/dismiss", post(handlers::dismiss_legacy_migration));

    let settings_routes = Router::new()
        .route("/settings", get(handlers::settings_overview))
        .route("/settings/all", get(handlers::get_settings));

    let system_routes = Router::new()
        .route("/system", get(handlers::system_snapshot))
        .route("/system/default-run-path", get(handlers::default_run_path))
        .route("/system/temp-download-dir", get(handlers::temp_download_dir))
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
        .nest(API_PREFIX, instance_registry_routes)
        .nest(API_PREFIX, provisioning_routes)
        .nest(API_PREFIX, settings_routes)
        .nest(API_PREFIX, legacy_migration_routes)
        .nest(API_PREFIX, system_routes)
        .nest(API_PREFIX, cron_routes)
        .nest(API_PREFIX, update_routes)
        .nest(API_PREFIX, download_routes)
        .merge(plugin_rpc_routes)
        .fallback_service(axctl_core::serve::spa(assets))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;

    use super::*;

    async fn test_router() -> (Router, TempDir) {
        let directory = tempdir().expect("create temporary directory");
        let settings = std::sync::Arc::new(
            sealantern_application::service::CoreSettingsService::with_settings_file(
                directory.path().join("settings.json"),
            )
            .await
            .expect("create settings service"),
        );
        (
            build_router(AppServices::from_inner(settings), FrontendAssets::empty()),
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

    #[tokio::test]
    async fn registry_routes_live_under_a_separate_prefix() {
        // 名单维护操作按目录定位，挂在 `/instance-registry/*` 而非 `/instances/*`：
        // 后者会让静态段与 `{id}` 同层共位，撞名时静默走错语义。
        // 用 GET 探测该前缀（只注册了 POST/DELETE）——命中静态段返回 405；
        // 若误挂在 `/instances/*` 下，则会回退到 `{id}` 的 GET 并返回 404。
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::get("/api/instance-registry/ignored-dirs")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call route");

        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "registry prefix must be a real static route, not fall back to {{id}}"
        );
    }

    #[tokio::test]
    async fn instance_id_route_is_no_longer_shadowed_by_registry_paths() {
        // 回归防线：`/instances/ignored-dirs` 与 `/instances/reassign-id` 不得
        // 再注册为静态路由，否则同 id 的实例会被遮蔽。以 GET 探测——若存在
        // 静态段会返回 405，正常情况应落到 `{id}` 的 GET（实例不存在 → 404）。
        let (router, _directory) = test_router().await;

        for id in ["ignored-dirs", "reassign-id"] {
            let response = router
                .clone()
                .oneshot(
                    Request::get(format!("/api/instances/{id}"))
                        .body(Body::empty())
                        .expect("build request"),
                )
                .await
                .expect("call route");

            assert_ne!(
                response.status(),
                StatusCode::METHOD_NOT_ALLOWED,
                "id {id} must reach the dynamic route, not a static segment"
            );
        }
    }
}
