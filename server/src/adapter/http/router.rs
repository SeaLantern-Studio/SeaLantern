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
        // ── 嵌套子资源（服务器配置：server.properties） ──
        .route(
            "/instances/{id}/server-properties",
            get(handlers::read_server_properties).put(handlers::write_server_properties),
        )
        .route(
            "/instances/{id}/server-properties/source",
            get(handlers::read_server_properties_source)
                .put(handlers::write_server_properties_source),
        )
        .route(
            "/instances/{id}/server-properties/preview",
            post(handlers::preview_server_properties_write),
        )
        // ── 嵌套子资源（服务器插件：plugins 目录） ──
        .route(
            "/instances/{id}/plugins",
            get(handlers::list_server_plugins)
                .post(handlers::install_server_plugin)
                .delete(handlers::delete_server_plugin),
        )
        .route(
            "/instances/{id}/plugins/config-files",
            get(handlers::read_server_plugin_config_files),
        )
        .route(
            "/instances/{id}/plugins/enabled",
            put(handlers::set_server_plugin_enabled),
        )
        // ── 嵌套子资源（后续扩展） ──
        // 示例：.route("/instances/{id}/logs", get(handlers::instance_logs))
        .route("/instances/{id}/path", put(handlers::update_instance_path));

    let settings_routes = Router::new()
        .route("/settings", get(handlers::settings_overview))
        .route("/settings/all", get(handlers::get_settings));

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

    // 非实例维度的纯文本变换端点：调用方直接给出源码，不触碰任何实例文件。
    //
    // 与实例维度同名端点的区别（两者都叫 "preview"，不要混淆）：
    // - `/server-properties/preview`（本组）：以调用方给出的 `source` 为基准做写入预览；
    // - `/instances/{id}/server-properties/preview`：以实例磁盘上现有文件为基准做写入预览。
    let server_properties_routes = Router::new()
        .route("/server-properties/parse", post(handlers::parse_server_properties_source))
        .route(
            "/server-properties/preview",
            post(handlers::preview_server_properties_write_from_source),
        );

    Router::new()
        .nest(API_PREFIX, instance_routes)
        .nest(API_PREFIX, provisioning_routes)
        .nest(API_PREFIX, settings_routes)
        .nest(API_PREFIX, system_routes)
        .nest(API_PREFIX, cron_routes)
        .nest(API_PREFIX, update_routes)
        .nest(API_PREFIX, download_routes)
        .nest(API_PREFIX, server_properties_routes)
        .merge(plugin_rpc_routes)
        .fallback_service(axctl_core::serve::spa(assets))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use sealantern_application::service::CoreInstanceService;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};
    use tower::ServiceExt;

    use super::*;

    async fn test_router() -> (Router, TempDir) {
        let directory = tempdir().expect("create temporary directory");
        let instance = CoreInstanceService::with_path(directory.path().join("instances.json"))
            .await
            .expect("create instance service");
        (
            build_router(AppServices::from_inner(instance), FrontendAssets::empty()),
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
    async fn unknown_instance_server_properties_returns_not_found() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::get("/api/instances/missing/server-properties")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call server properties route");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .expect("read error response");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse error response");
        assert_eq!(value.get("code").and_then(|code| code.as_str()), Some("instance_not_found"));
    }

    #[tokio::test]
    async fn blank_instance_id_is_rejected_as_client_error() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::get("/api/instances/%20/server-properties/source")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call server properties source route");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .expect("read error response");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse error response");
        assert_eq!(value.get("code").and_then(|code| code.as_str()), Some("invalid_instance_id"));
    }

    #[tokio::test]
    async fn server_properties_parse_route_needs_no_instance() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::post("/api/server-properties/parse")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"motd=hello\n"}"#))
                    .expect("build request"),
            )
            .await
            .expect("call parse route");

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read parse response");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse parse response");
        assert!(value.get("entries").is_some(), "unexpected payload: {value}");
        assert!(value.get("raw").is_some(), "unexpected payload: {value}");
        assert!(value.get("entriesSnake").is_none(), "unexpected payload: {value}");
    }

    /// 端到端正向路径：创建实例后按实例维度读写 server.properties。
    ///
    /// 覆盖 `resolve_instance_directory` 的成功分支，以及写入、读回、原始文本、
    /// 预览（含"预览不落盘"）各端点。
    #[tokio::test]
    async fn instance_server_properties_round_trip() {
        let (router, directory) = test_router().await;

        // ── 1. 注册一个实例，其目录位于临时目录内 ──
        let server_directory = directory.path().join("server-42");
        std::fs::create_dir_all(&server_directory).expect("create server directory");
        let spec = serde_json::json!({
            "id": "server-42",
            "name": "测试服",
            "aliases": [],
            "core_type": "paper",
            "core_version": "1.20.4",
            "game_version": "1.20.4",
            "directory": server_directory,
            "port": 25565,
            "max_memory_mib": 2048,
            "min_memory_mib": 512,
            "created_at_unix_secs": 0,
            "last_started_at_unix_secs": null,
            "server_metadata": null,
            "launch": {
                "startup_mode": "jar",
                "startup_target": server_directory.join("server.jar"),
                "custom_command": null,
                "custom_executable": null,
                "custom_arguments": [],
                "java_executable": null,
                "jvm_arguments": [],
            },
        });
        let response = router
            .clone()
            .oneshot(
                Request::post("/api/instances")
                    .header("content-type", "application/json")
                    .body(Body::from(spec.to_string()))
                    .expect("build request"),
            )
            .await
            .expect("call create instance route");
        assert_eq!(response.status(), StatusCode::CREATED);

        // ── 2. 按实例写入键值对 ──
        let response = router
            .clone()
            .oneshot(
                Request::put("/api/instances/server-42/server-properties")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"values":{"motd":"hello","server-port":"25566"}}"#))
                    .expect("build request"),
            )
            .await
            .expect("call write route");
        assert_eq!(response.status(), StatusCode::OK);

        // ── 3. 读回并校验写入结果 ──
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/instances/server-42/server-properties")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call read route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read properties response");
        let value: serde_json::Value =
            serde_json::from_slice(&body).expect("parse properties response");
        assert_eq!(value.pointer("/raw/motd").and_then(|item| item.as_str()), Some("hello"));
        assert_eq!(
            value
                .pointer("/raw/server-port")
                .and_then(|item| item.as_str()),
            Some("25566")
        );

        // ── 4. 原始文本端点 ──
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/instances/server-42/server-properties/source")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call read source route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read source response");
        let source: String = serde_json::from_slice(&body).expect("parse source response");
        assert!(source.contains("motd=hello"), "unexpected source: {source}");

        // ── 5. 预览：返回改写后的文本，但不落盘 ──
        let response = router
            .clone()
            .oneshot(
                Request::post("/api/instances/server-42/server-properties/preview")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"values":{"motd":"preview-only"}}"#))
                    .expect("build request"),
            )
            .await
            .expect("call preview route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read preview response");
        let preview: String = serde_json::from_slice(&body).expect("parse preview response");
        assert!(preview.contains("motd=preview-only"), "unexpected preview: {preview}");

        let response = router
            .oneshot(
                Request::get("/api/instances/server-42/server-properties/source")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call read source route again");
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read source response");
        let source: String = serde_json::from_slice(&body).expect("parse source response");
        assert!(
            source.contains("motd=hello") && !source.contains("preview-only"),
            "preview must not persist, got: {source}"
        );
    }

    /// 注册一个实例并返回其服务器目录；调用方负责后续断言。
    async fn create_instance_fixture(router: &Router, directory: &TempDir, id: &str) -> PathBuf {
        let server_directory = directory.path().join(id);
        std::fs::create_dir_all(&server_directory).expect("create server directory");
        let spec = serde_json::json!({
            "id": id,
            "name": "测试服",
            "aliases": [],
            "core_type": "paper",
            "core_version": "1.20.4",
            "game_version": "1.20.4",
            "directory": server_directory,
            "port": 25565,
            "max_memory_mib": 2048,
            "min_memory_mib": 512,
            "created_at_unix_secs": 0,
            "last_started_at_unix_secs": null,
            "server_metadata": null,
            "launch": {
                "startup_mode": "jar",
                "startup_target": server_directory.join("server.jar"),
                "custom_command": null,
                "custom_executable": null,
                "custom_arguments": [],
                "java_executable": null,
                "jvm_arguments": [],
            },
        });

        let response = router
            .clone()
            .oneshot(
                Request::post("/api/instances")
                    .header("content-type", "application/json")
                    .body(Body::from(spec.to_string()))
                    .expect("build request"),
            )
            .await
            .expect("call create instance route");
        assert_eq!(response.status(), StatusCode::CREATED);

        server_directory
    }

    #[tokio::test]
    async fn unknown_instance_plugins_returns_not_found() {
        let (router, _directory) = test_router().await;

        let response = router
            .oneshot(
                Request::get("/api/instances/missing/plugins")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call plugin list route");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .expect("read error response");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse error response");
        assert_eq!(value.get("code").and_then(|code| code.as_str()), Some("instance_not_found"));
    }

    /// 端到端：安装 → 列表 → 禁用 → 启用 → 删除。
    ///
    /// 同时验证文件名穿越被拒后返回 400，而不是落到实例目录之外。
    #[tokio::test]
    async fn instance_plugins_round_trip() {
        let (router, directory) = test_router().await;
        let server_directory = create_instance_fixture(&router, &directory, "server-7").await;

        // ── 1. 初始为空 ──
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/instances/server-7/plugins")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call plugin list route");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read plugin list");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse plugin list");
        assert_eq!(value.as_array().map(Vec::len), Some(0));

        // ── 2. 安装（字节数组与 Tauri 契约一致） ──
        let response = router
            .clone()
            .oneshot(
                Request::post("/api/instances/server-7/plugins")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"file_name":"Broken.jar","file_data":[1,2,3]}"#))
                    .expect("build request"),
            )
            .await
            .expect("call install route");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            server_directory
                .join("plugins")
                .join("Broken.jar")
                .is_file()
        );

        // ── 3. 列表中可见；jar 不是合法归档，名称回落到文件名 ──
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/instances/server-7/plugins")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call plugin list route");
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("read plugin list");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse plugin list");
        assert_eq!(
            value.pointer("/0/file_name").and_then(|item| item.as_str()),
            Some("Broken.jar")
        );
        assert_eq!(value.pointer("/0/enabled").and_then(|item| item.as_bool()), Some(true));

        // ── 4. 禁用后再列表，enabled 翻转 ──
        let response = router
            .clone()
            .oneshot(
                Request::put("/api/instances/server-7/plugins/enabled")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"file_name":"Broken.jar","enabled":false}"#))
                    .expect("build request"),
            )
            .await
            .expect("call set enabled route");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            server_directory
                .join("plugins")
                .join("Broken.jar.disabled")
                .is_file()
        );

        // ── 5. 删除 ──
        let response = router
            .clone()
            .oneshot(
                Request::delete("/api/instances/server-7/plugins?file_name=Broken.jar")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call delete route");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            !server_directory
                .join("plugins")
                .join("Broken.jar.disabled")
                .exists()
        );

        // ── 6. 路径穿越被拒 ──
        let response = router
            .oneshot(
                Request::delete("/api/instances/server-7/plugins?file_name=..%2Fescape.jar")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("call delete route");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .expect("read error response");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse error response");
        assert_eq!(value.get("code").and_then(|code| code.as_str()), Some("invalid_input"));
    }
}
