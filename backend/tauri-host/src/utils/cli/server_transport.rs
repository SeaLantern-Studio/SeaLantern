use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use crate::models::server::ServerInstance;
use crate::services::global;
use crate::utils::server_status::status_blocks_start;

use super::cli_env::{effective_cli_web_bind_host, is_headless_http_environment};
use super::server_args::{CliMode, CliServerCommand, WebMode};
use super::server_control::start_server_with_feedback;
use super::server_endpoint::{render_cli_web_binding_hint, render_cli_web_browser_url};
use super::server_ports::PreparedPorts;
use super::server_session::attach_server_cli;
use super::server_shared::{
    describe_server_instance, describe_status, trace_cli_action, trace_cli_error,
};

pub(super) struct WebTransportHandle {
    pub(super) join_handle: std::thread::JoinHandle<()>,
    pub(super) url: String,
}

pub(super) fn orchestrate_transports(
    command: &CliServerCommand,
    server: &ServerInstance,
    ports: &PreparedPorts,
) -> Result<(), String> {
    orchestrate_transports_with(
        command,
        server,
        ports,
        ensure_server_started,
        start_web_transport,
        |server| {
            attach_server_cli(server);
            Ok(())
        },
        |web_handle| {
            println!("web transport 正在运行: {}\n按 Ctrl+C 可结束当前命令进程。", web_handle.url);
            let _ = web_handle.join_handle.join();
            Ok(())
        },
    )
}

pub(super) fn orchestrate_transports_with<FEnsure, FStartWeb, FAttachCli, FWaitWeb, H>(
    command: &CliServerCommand,
    server: &ServerInstance,
    ports: &PreparedPorts,
    mut ensure_started: FEnsure,
    mut start_web: FStartWeb,
    mut attach_cli: FAttachCli,
    mut wait_web: FWaitWeb,
) -> Result<(), String>
where
    FEnsure: FnMut(&ServerInstance) -> Result<(), String>,
    FStartWeb: FnMut(u16, &ServerInstance) -> Result<H, String>,
    FAttachCli: FnMut(&ServerInstance) -> Result<(), String>,
    FWaitWeb: FnMut(H) -> Result<(), String>,
{
    trace_cli_action("transport_prepare", &describe_server_instance(server));
    ensure_started(server)?;

    let mut web_handle = None;
    if command.web == WebMode::Enabled {
        trace_cli_action(
            "transport_web_start",
            &format!("server_id={} port={}", server.id, ports.web_port.unwrap_or(8888)),
        );
        web_handle = Some(start_web(ports.web_port.unwrap_or(8888), server)?);
    }
    if command.cli == CliMode::Enabled {
        trace_cli_action("transport_cli_attach", &format!("server_id={}", server.id));
        attach_cli(server)?;
    }
    if let Some(web_handle) = web_handle {
        wait_web(web_handle)?;
    }
    Ok(())
}

pub(super) fn start_web_transport(
    port: u16,
    server: &ServerInstance,
) -> Result<WebTransportHandle, String> {
    let bind_host = effective_cli_web_bind_host();
    let bind_addr = format!("{}:{}", bind_host, port);
    let browser_url = render_cli_web_browser_url(port, &server.id);
    let static_dir = resolve_static_dir();
    println!("web transport 已计划启动: {}", browser_url);
    if let Some(hint) = render_cli_web_binding_hint(port) {
        println!("web transport 绑定提示: {}", hint);
    }

    let (startup_tx, startup_rx) = mpsc::channel::<Result<(), String>>();
    let bind_addr_for_thread = bind_addr.clone();

    let join_handle = std::thread::spawn(move || {
        let bind_addr = bind_addr_for_thread;
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(err) => {
                let message = format!("无法为 web transport 创建 Tokio runtime: {}", err);
                let _ = startup_tx.send(Err(message.clone()));
                eprintln!("{}", message);
                return;
            }
        };

        rt.block_on(async move {
            let _ =
                crate::adapters::http::run_http_server(&bind_addr, static_dir, Some(startup_tx))
                    .await;
        });
    });

    match startup_rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(())) => {
            trace_cli_action(
                "transport_web_ready",
                &format!("server_id={} bind_addr={} url={}", server.id, bind_addr, browser_url),
            );
        }
        Ok(Err(error)) => {
            trace_cli_error(
                "transport_web_start_failed",
                &format!("server_id={} bind_addr={} url={}", server.id, bind_addr, browser_url),
                &error,
            );
            return Err(format!("web transport 启动失败: {}", error));
        }
        Err(_) => {
            trace_cli_action(
                "transport_web_start_pending",
                &format!("server_id={} bind_addr={} url={}", server.id, bind_addr, browser_url),
            );
        }
    }

    if should_auto_open_browser(is_headless_http_environment()) {
        let browser_opened = open_browser_or_warn(&browser_url);
        if !browser_opened {
            println!("浏览器未自动打开，请手动访问: {}", browser_url);
        }
    } else {
        trace_cli_action(
            "transport_web_browser_skipped",
            &format!("reason=headless_http url={}", browser_url),
        );
        println!("当前环境不会自动打开浏览器，请手动访问: {}", browser_url);
    }
    Ok(WebTransportHandle { join_handle, url: browser_url })
}

pub(super) fn should_auto_open_browser(is_headless_http: bool) -> bool {
    !is_headless_http
}

pub(super) fn open_browser_or_warn(browser_url: &str) -> bool {
    open_browser_or_warn_with(browser_url, |url| {
        opener::open(url)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
}

pub(super) fn open_browser_or_warn_with<F>(browser_url: &str, open_browser: F) -> bool
where
    F: FnOnce(&str) -> Result<(), String>,
{
    match open_browser(browser_url) {
        Ok(()) => true,
        Err(error) => {
            trace_cli_error("transport_web_open_browser_failed", browser_url, &error);
            false
        }
    }
}

pub(super) fn ensure_server_started(server: &ServerInstance) -> Result<(), String> {
    let status = global::server_manager().get_server_status(&server.id);
    if status_blocks_start(&status) {
        trace_cli_action(
            "start_skipped_existing_state",
            &format!("server_id={} status={}", server.id, describe_status(&status.status)),
        );
        println!(
            "服务器已存在运行态，跳过重复启动: id={}, status={}",
            server.id,
            status.status.as_str()
        );
        return Ok(());
    }

    trace_cli_action("start_trigger", &format!("server_id={}", server.id));
    start_server_with_feedback(server, "create_flow", "服务器正在启动...")?;
    Ok(())
}

fn resolve_static_dir() -> Option<String> {
    if let Ok(static_dir) = std::env::var("STATIC_DIR") {
        if Path::new(&static_dir).exists() {
            return Some(static_dir);
        }
    }

    let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.join("dist"));
    candidate
        .filter(|path| path.join("index.html").exists())
        .map(|path| path.to_string_lossy().to_string())
}
