use crate::utils::logger::{log_error_ctx, log_info_ctx};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

const DEFAULT_DESKTOP_WEB_BIND_ADDR: &str = "127.0.0.1:3000";

#[derive(Debug, Clone)]
pub struct DesktopWebStatusSnapshot {
    pub running: bool,
    pub bind_addr: String,
    pub static_dir_available: bool,
}

pub struct DesktopWebState {
    join_handle: tauri::async_runtime::JoinHandle<()>,
    bind_addr: String,
}

fn state() -> &'static Mutex<Option<DesktopWebState>> {
    static INSTANCE: OnceLock<Mutex<Option<DesktopWebState>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(None))
}

fn resolve_desktop_web_bind_addr() -> String {
    sea_lantern_runtime::resolve_http_bind_addr_checked(3000)
        .unwrap_or_else(|_| DEFAULT_DESKTOP_WEB_BIND_ADDR.to_string())
}

fn candidate_static_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("dist"));
        candidates.push(resource_dir);
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("dist"));
        candidates.push(current_dir.join("../../dist"));
        candidates.push(current_dir.join("../dist"));
    }

    candidates
}

fn resolve_static_dir(app: &AppHandle) -> Option<String> {
    candidate_static_dirs(app)
        .into_iter()
        .find(|dir| dir.join("index.html").exists())
        .map(|dir| dir.to_string_lossy().to_string())
}

fn start_locked(app: &AppHandle, slot: &mut Option<DesktopWebState>) -> Result<(), String> {
    if slot
        .as_ref()
        .is_some_and(|state| !state.join_handle.inner().is_finished())
    {
        return Ok(());
    }

    if slot
        .as_ref()
        .is_some_and(|state| state.join_handle.inner().is_finished())
    {
        *slot = None;
    }

    let bind_addr = resolve_desktop_web_bind_addr();
    let static_dir = resolve_static_dir(app);
    let bind_addr_for_task = bind_addr.clone();
    let static_dir_for_task = static_dir.clone();
    let (startup_tx, startup_rx) = std::sync::mpsc::channel();

    let join_handle = tauri::async_runtime::spawn(async move {
        if let Err(error) = crate::adapters::http::server::run_http_server(
            &bind_addr_for_task,
            static_dir_for_task,
            Some(startup_tx),
        )
        .await
        {
            log_error_ctx(
                "services.desktop_web",
                "start_locked",
                &format!("desktop web server exited: {}", error),
            );
        }
    });

    match startup_rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            join_handle.abort();
            return Err(error);
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            join_handle.abort();
            return Err("Timed out while starting desktop web server".to_string());
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            join_handle.abort();
            return Err("Desktop web server exited before signaling readiness".to_string());
        }
    }

    log_info_ctx(
        "services.desktop_web",
        "start_locked",
        &format!(
            "desktop web server started bind_addr={} static_dir={}",
            bind_addr,
            static_dir.as_deref().unwrap_or("<api-only>")
        ),
    );

    *slot = Some(DesktopWebState { join_handle, bind_addr });
    Ok(())
}

fn stop_locked(slot: &mut Option<DesktopWebState>) {
    if let Some(state) = slot.take() {
        state.join_handle.abort();
        log_info_ctx(
            "services.desktop_web",
            "stop_locked",
            &format!("desktop web server stopped bind_addr={}", state.bind_addr),
        );
    }
}

pub fn sync_desktop_web_server(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mut guard = state()
        .lock()
        .map_err(|_| "desktop web state lock poisoned".to_string())?;

    if enabled {
        start_locked(app, &mut guard)
    } else {
        stop_locked(&mut guard);
        Ok(())
    }
}

pub fn snapshot_desktop_web_status(app: &AppHandle) -> DesktopWebStatusSnapshot {
    let running_bind_addr = state().lock().ok().and_then(|guard| {
        guard.as_ref().and_then(|current| {
            if current.join_handle.inner().is_finished() {
                None
            } else {
                Some(current.bind_addr.clone())
            }
        })
    });

    let bind_addr = running_bind_addr
        .clone()
        .unwrap_or_else(resolve_desktop_web_bind_addr);

    DesktopWebStatusSnapshot {
        running: running_bind_addr.is_some(),
        bind_addr,
        static_dir_available: resolve_static_dir(app).is_some(),
    }
}

pub fn resolve_desktop_web_url(bind_addr: &str) -> String {
    format!("http://{}", bind_addr)
}

#[allow(dead_code)]
pub fn static_dir_exists(path: &Path) -> bool {
    path.join("index.html").exists()
}
