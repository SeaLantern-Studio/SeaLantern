use crate::models::server::{ServerStatus, ServerStatusInfo};
use crate::utils::cli::cli_env::effective_cli_web_bind_host;
use crate::utils::cli::server_args::{CliMode, CliServerCommand, WebMode};
use crate::utils::cli::server_endpoint::render_cli_web_browser_url;
use crate::utils::cli::server_flow::{
    ensure_transport_defaults, infer_server_name_from_folder, prepare_server_ports,
    resolve_server_command_name, validate_transport_mode,
};
use crate::utils::cli::server_ports::PreparedPorts;
use crate::utils::cli::server_test_support::{lock_env, sample_server, EnvGuard};
use crate::utils::cli::server_transport::{
    open_browser_or_warn_with, orchestrate_transports_with, should_auto_open_browser,
};
use crate::utils::server_status::status_blocks_start;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[test]
fn open_browser_helper_returns_true_on_success() {
    let opened = open_browser_or_warn_with("http://127.0.0.1:8888/console/test", |_| Ok(()));
    assert!(opened);
}

#[test]
fn open_browser_helper_returns_false_on_failure_without_bubbling_error() {
    let opened = open_browser_or_warn_with("http://127.0.0.1:8888/console/test", |_| {
        Err("headless environment".to_string())
    });
    assert!(!opened);
}

#[test]
fn browser_url_maps_container_bind_to_loopback() {
    let _env_lock = lock_env();
    {
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");
        assert_eq!(
            render_cli_web_browser_url(8888, "transport-test"),
            "http://127.0.0.1:8888/console/transport-test"
        );
    }

    {
        let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "127.0.0.1");
        assert_eq!(
            render_cli_web_browser_url(8888, "transport-test"),
            "http://127.0.0.1:8888/console/transport-test"
        );
    }
}

#[test]
fn auto_open_browser_is_disabled_for_headless_http_environments() {
    assert!(!should_auto_open_browser(true));
    assert!(should_auto_open_browser(false));
}

#[test]
fn web_bind_host_prefers_explicit_env_override() {
    let _env_lock = lock_env();
    let _bind_guard = EnvGuard::set("SEALANTERN_WEB_BIND", "192.168.1.10");

    let bind_host = effective_cli_web_bind_host();
    assert_eq!(bind_host, "192.168.1.10");
}

#[test]
fn cli_web_bind_defaults_to_loopback_even_in_headless_mode() {
    let _env_lock = lock_env();
    let _headless_guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
    let _http_bind_guard = EnvGuard::remove("SEALANTERN_HTTP_BIND");
    let _web_bind_guard = EnvGuard::remove("SEALANTERN_WEB_BIND");

    let bind_host = effective_cli_web_bind_host();
    assert_eq!(bind_host, "127.0.0.1");
}

#[tokio::test]
async fn cli_resolved_bind_still_enforces_http_auth_and_default_cors() {
    let bind_addr = {
        let _env_lock = lock_env();
        let _headless_guard = EnvGuard::set("SEALANTERN_HEADLESS_HTTP", "1");
        let _http_bind_guard = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web_bind_guard = EnvGuard::remove("SEALANTERN_WEB_BIND");
        let _cors_guard = EnvGuard::remove("SEALANTERN_HTTP_CORS_ORIGINS");
        let _auth_guard = EnvGuard::remove("SEALANTERN_HTTP_AUTH_TOKEN");
        format!("{}:{}", effective_cli_web_bind_host(), 3000)
    };

    assert_eq!(bind_addr, "127.0.0.1:3000");

    let app = crate::adapters::http::server::build_test_http_app(std::env::temp_dir());

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("health response");
    assert_eq!(health.status(), StatusCode::OK);

    let protected = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/does-not-exist")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"params":{}}"#))
                .unwrap(),
        )
        .await
        .expect("protected response");
    assert_eq!(protected.status(), StatusCode::UNAUTHORIZED);

    let cors = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header(header::ORIGIN, "https://example.com")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("cors response");
    assert!(cors
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());
}

#[test]
fn infer_server_name_from_folder_uses_last_path_segment() {
    assert_eq!(
        infer_server_name_from_folder(Some("E:/servers/cache-server")),
        Some("cache-server".to_string())
    );
}

#[test]
fn resolve_server_command_name_falls_back_to_folder_name() {
    let command = CliServerCommand {
        folder: Some("E:/servers/cache-server".to_string()),
        ..Default::default()
    };

    let resolved =
        resolve_server_command_name(&command).expect("folder name should be a valid fallback");
    assert_eq!(resolved, "cache-server");
}

#[test]
fn resolve_server_command_name_still_prefers_explicit_name_over_folder() {
    let command = CliServerCommand {
        name: Some("manual-name".to_string()),
        folder: Some("E:/servers/cache-server".to_string()),
        ..Default::default()
    };

    let resolved = resolve_server_command_name(&command).expect("explicit name should win");
    assert_eq!(resolved, "manual-name");
}

#[test]
fn ensure_transport_defaults_enables_cli_when_no_transport_selected() {
    let mut command = CliServerCommand::default();
    ensure_transport_defaults(&mut command);
    assert_eq!(command.cli, CliMode::Enabled);
    assert_eq!(command.web, WebMode::Disabled);
}

#[test]
fn ensure_transport_defaults_keeps_detach_non_interactive() {
    let mut command = CliServerCommand {
        detach: true,
        web: WebMode::Enabled,
        cli: CliMode::Enabled,
        ..Default::default()
    };

    ensure_transport_defaults(&mut command);
    assert_eq!(command.web, WebMode::Disabled);
    assert_eq!(command.cli, CliMode::Disabled);
}

#[test]
fn ensure_transport_defaults_keeps_create_only_non_interactive() {
    let mut command = CliServerCommand {
        create_only: true,
        web: WebMode::Enabled,
        cli: CliMode::Enabled,
        ..Default::default()
    };

    ensure_transport_defaults(&mut command);
    assert_eq!(command.web, WebMode::Disabled);
    assert_eq!(command.cli, CliMode::Disabled);
}

#[test]
fn validate_transport_mode_rejects_detach_with_cli_or_web() {
    let cli_err = validate_transport_mode(&CliServerCommand {
        detach: true,
        cli: CliMode::Enabled,
        ..Default::default()
    })
    .expect_err("detach + cli should be rejected");
    assert!(cli_err.contains("--detach"));

    let web_err = validate_transport_mode(&CliServerCommand {
        detach: true,
        web: WebMode::Enabled,
        ..Default::default()
    })
    .expect_err("detach + web should be rejected");
    assert!(web_err.contains("--web"));
}

#[test]
fn validate_transport_mode_rejects_create_only_with_interactive_transports_or_detach() {
    let cli_err = validate_transport_mode(&CliServerCommand {
        create_only: true,
        cli: CliMode::Enabled,
        ..Default::default()
    })
    .expect_err("create-only + cli should be rejected");
    assert!(cli_err.contains("--create-only"));

    let web_err = validate_transport_mode(&CliServerCommand {
        create_only: true,
        web: WebMode::Enabled,
        ..Default::default()
    })
    .expect_err("create-only + web should be rejected");
    assert!(web_err.contains("--web"));

    let detach_err = validate_transport_mode(&CliServerCommand {
        create_only: true,
        detach: true,
        ..Default::default()
    })
    .expect_err("create-only + detach should be rejected");
    assert!(detach_err.contains("--detach"));
}

#[test]
fn prepare_server_ports_skips_conflict_probe_in_create_only_mode() {
    let command = CliServerCommand {
        create_only: true,
        port: Some(Some(25565)),
        web: WebMode::Enabled,
        web_port: Some(Some(8000)),
        ..Default::default()
    };

    let ports = prepare_server_ports(
        &command,
        crate::utils::cli::server_shared::CliServerRuntimeKind::Docker,
        |_, _, _| Err("create-only should not invoke port probing".to_string()),
    )
    .expect("create-only should keep requested/default port without probing conflicts");

    assert_eq!(ports, PreparedPorts { game_port: 25565, web_port: None });
}

#[test]
fn orchestrate_transports_runs_web_only_flow_without_cli_attach() {
    let command = CliServerCommand {
        web: WebMode::Enabled,
        web_port: Some(Some(8000)),
        cli: CliMode::Disabled,
        ..Default::default()
    };
    let server = sample_server();
    let ports = PreparedPorts { game_port: 25565, web_port: Some(8000) };
    let events = Arc::new(Mutex::new(Vec::new()));

    let ensure_events = Arc::clone(&events);
    let start_events = Arc::clone(&events);
    let attach_events = Arc::clone(&events);
    let wait_events = Arc::clone(&events);

    orchestrate_transports_with(
        &command,
        &server,
        &ports,
        move |_| {
            ensure_events
                .lock()
                .expect("events lock")
                .push("ensure".to_string());
            Ok(())
        },
        move |port, server| {
            start_events
                .lock()
                .expect("events lock")
                .push(format!("web-start:{}:{}", server.id, port));
            Ok("web-handle")
        },
        move |_| {
            attach_events
                .lock()
                .expect("events lock")
                .push("cli".to_string());
            Ok(())
        },
        move |handle| {
            wait_events
                .lock()
                .expect("events lock")
                .push(format!("wait:{}", handle));
            Ok(())
        },
    )
    .expect("web-only orchestration should succeed");

    let events = events.lock().expect("events lock");
    assert_eq!(events.as_slice(), ["ensure", "web-start:server-1:8000", "wait:web-handle"]);
}

#[test]
fn orchestrate_transports_runs_web_and_cli_flow_in_order() {
    let command = CliServerCommand {
        web: WebMode::Enabled,
        web_port: Some(Some(8001)),
        cli: CliMode::Enabled,
        ..Default::default()
    };
    let server = sample_server();
    let ports = PreparedPorts { game_port: 25565, web_port: Some(8001) };
    let events = Arc::new(Mutex::new(Vec::new()));

    let ensure_events = Arc::clone(&events);
    let start_events = Arc::clone(&events);
    let attach_events = Arc::clone(&events);
    let wait_events = Arc::clone(&events);

    orchestrate_transports_with(
        &command,
        &server,
        &ports,
        move |_| {
            ensure_events
                .lock()
                .expect("events lock")
                .push("ensure".to_string());
            Ok(())
        },
        move |port, _| {
            start_events
                .lock()
                .expect("events lock")
                .push(format!("web-start:{}", port));
            Ok("web-handle")
        },
        move |_| {
            attach_events
                .lock()
                .expect("events lock")
                .push("cli".to_string());
            Ok(())
        },
        move |handle| {
            wait_events
                .lock()
                .expect("events lock")
                .push(format!("wait:{}", handle));
            Ok(())
        },
    )
    .expect("web+cli orchestration should succeed");

    let events = events.lock().expect("events lock");
    assert_eq!(events.as_slice(), ["ensure", "web-start:8001", "cli", "wait:web-handle"]);
}

#[test]
fn orchestrate_transports_runs_cli_only_without_web_start() {
    let command = CliServerCommand {
        web: WebMode::Disabled,
        cli: CliMode::Enabled,
        ..Default::default()
    };
    let server = sample_server();
    let ports = PreparedPorts { game_port: 25565, web_port: None };
    let events = Arc::new(Mutex::new(Vec::new()));

    let ensure_events = Arc::clone(&events);
    let attach_events = Arc::clone(&events);

    orchestrate_transports_with(
        &command,
        &server,
        &ports,
        move |_| {
            ensure_events
                .lock()
                .expect("events lock")
                .push("ensure".to_string());
            Ok(())
        },
        move |_, _| -> Result<&'static str, String> {
            panic!("web transport should not start for cli-only flow")
        },
        move |_| {
            attach_events
                .lock()
                .expect("events lock")
                .push("cli".to_string());
            Ok(())
        },
        move |_| -> Result<(), String> { panic!("wait should not run without web transport") },
    )
    .expect("cli-only orchestration should succeed");

    let events = events.lock().expect("events lock");
    assert_eq!(events.as_slice(), ["ensure", "cli"]);
}

#[test]
fn orchestrate_transports_bubbles_web_start_failure_before_cli_attach() {
    let command = CliServerCommand {
        web: WebMode::Enabled,
        web_port: Some(Some(8002)),
        cli: CliMode::Enabled,
        ..Default::default()
    };
    let server = sample_server();
    let ports = PreparedPorts { game_port: 25565, web_port: Some(8002) };
    let events = Arc::new(Mutex::new(Vec::new()));

    let ensure_events = Arc::clone(&events);
    let attach_events = Arc::clone(&events);

    let err = orchestrate_transports_with(
        &command,
        &server,
        &ports,
        move |_| {
            ensure_events
                .lock()
                .expect("events lock")
                .push("ensure".to_string());
            Ok(())
        },
        move |_, _| Err("web bind failed".to_string()),
        move |_| {
            attach_events
                .lock()
                .expect("events lock")
                .push("cli".to_string());
            Ok(())
        },
        move |_: ()| Ok(()),
    )
    .expect_err("web start failure should bubble");

    assert!(err.contains("web bind failed"));
    let events = events.lock().expect("events lock");
    assert_eq!(events.as_slice(), ["ensure"]);
}

#[test]
fn status_blocks_start_for_docker_error_snapshot_that_is_still_running() {
    let status = ServerStatusInfo {
        id: "docker-1".to_string(),
        status: ServerStatus::Error,
        pid: Some(4321),
        uptime: Some(42),
        detail_message: Some(
            "runtime=docker_itzg container=sea-test state=running running=true health=unhealthy exit_code=0 backend=cli command_mode=rcon"
                .to_string(),
        ),
        error_message: Some("Docker 容器健康检查失败".to_string()),
    };

    assert!(status_blocks_start(&status));
}
