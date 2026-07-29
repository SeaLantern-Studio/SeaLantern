mod bootstrap;
mod command_catalog;
mod desktop_setup;
mod desktop_shell;
mod mode;
mod plugin_bridge;

use crate::services;
use mode::RuntimeMode;

/// Selects the runtime mode and transfers control to the matching bootstrap path.
pub fn run() {
    crate::utils::cli::handle_cli();

    match RuntimeMode::detect() {
        RuntimeMode::HeadlessHttp { bind_addr, static_dir } => {
            run_headless_http(&bind_addr, static_dir)
        }
        RuntimeMode::Desktop => run_desktop(),
    }
}

/// Boots the embedded desktop runtime.
fn run_desktop() {
    // Fix white screen issue on Wayland desktop environments (tested on Arch Linux + KDE Plasma)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    // 尽早注册全局 panic hook，确保此后所有线程发生的 panic 都能被捕获。
    services::panic_report::init_panic_hook();
    bootstrap::run_desktop();
}

/// Boots the headless HTTP runtime on a dedicated Tokio runtime.
///
/// # Parameters
///
/// - `bind_addr`: the socket address to listen on
/// - `static_dir`: optional directory used to serve frontend assets
fn run_headless_http(bind_addr: &str, static_dir: Option<String>) {
    eprintln!("SeaLantern: Running in headless HTTP mode at {}", bind_addr);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("SeaLantern: Failed to create Tokio runtime for HTTP server: {}", e);
            eprintln!(
                "SeaLantern: This may be due to container resource limits (memory, threads, etc.)"
            );
            std::process::exit(1);
        }
    };

    rt.block_on(async {
        crate::adapters::http::run_http_server(bind_addr, static_dir).await;
    });
}
