mod bootstrap;
mod command_catalog;
mod desktop_setup;
mod desktop_shell;
mod plugin_bridge;

use sea_lantern_runtime::RuntimeMode;

/// Selects the runtime mode and transfers control to the matching bootstrap path.
pub fn run() {
    crate::utils::cli::handle_cli();

    match RuntimeMode::detect_checked()
        .unwrap_or_else(|error| panic!("SeaLantern: invalid runtime mode configuration: {}", error))
    {
        RuntimeMode::HeadlessHttp { bind_addr, static_dir } => {
            run_headless_http_with_bind(&bind_addr, static_dir)
        }
        RuntimeMode::Desktop => run_desktop(),
    }
}

/// Boots the headless HTTP runtime using the shared runtime-mode detection defaults.
pub fn run_headless_http() {
    match RuntimeMode::detect_checked()
        .unwrap_or_else(|error| panic!("SeaLantern: invalid headless HTTP configuration: {}", error))
    {
        RuntimeMode::HeadlessHttp { bind_addr, static_dir } => {
            run_headless_http_with_bind(&bind_addr, static_dir)
        }
        RuntimeMode::Desktop => {
            let bind_addr = sea_lantern_runtime::resolve_http_bind_addr_checked(3000)
                .unwrap_or_else(|error| panic!("SeaLantern: invalid headless HTTP bind configuration: {}", error));
            run_headless_http_with_bind(&bind_addr, None);
        }
    }
}

/// Boots the embedded desktop runtime.
fn run_desktop() {
    // Fix white screen issue on Wayland desktop environments (tested on Arch Linux + KDE Plasma)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    // 尽早注册全局 panic hook，确保此后所有线程发生的 panic 都能被捕获。
    sea_lantern_runtime::init_panic_hook();
    bootstrap::run_desktop();
}

/// Boots the headless HTTP runtime on a dedicated Tokio runtime.
///
/// # Parameters
///
/// - `bind_addr`: the socket address to listen on
/// - `static_dir`: optional directory used to serve frontend assets
fn run_headless_http_with_bind(bind_addr: &str, static_dir: Option<String>) {
    let bind_addr = bind_addr.to_string();
    sea_lantern_runtime::run_tokio_service(
        sea_lantern_runtime::TokioServiceConfig {
            startup_message: format!("SeaLantern: Running in headless HTTP mode at {}", bind_addr),
            runtime_creation_error_prefix: "SeaLantern: Failed to create Tokio runtime for HTTP server",
            runtime_creation_error_hint: Some(
                "SeaLantern: This may be due to container resource limits (memory, threads, etc.)",
            ),
            service_error_prefix: "SeaLantern: Headless HTTP runtime failed to start or exited with error",
        },
        move || async move {
            crate::adapters::http::server::run_http_server(&bind_addr, static_dir, None).await
        },
    );
}
