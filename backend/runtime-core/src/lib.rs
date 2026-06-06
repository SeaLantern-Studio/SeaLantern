mod headless_runtime;
mod headless_http;
mod http_dispatch;
mod http_bind;
mod logging;
mod panic_report;
mod panic_report_pathing;
mod panic_report_system_info;
mod path_utils;
mod port_usage;
mod runtime_mode;
mod server_status;
mod tokio_runtime;

pub use http_bind::{resolve_http_bind_addr, resolve_http_bind_addr_checked, resolve_http_bind_host};
pub use logging::{
    GLOBAL_LOG_COLLECTOR, LogLine, capture_eprintln, capture_println, format_log_entry,
    log_debug, log_error, log_trace,
    log_user_action, log_user_action_error, log_warn, to_log_line,
};
pub use headless_runtime::{TokioServiceConfig, run_tokio_service};
pub use headless_http::{
    HTTP_AUTH_TOKEN_ENV, HTTP_CORS_ORIGINS_ENV, HeadlessHttpConfig, default_headless_http_config,
    default_headless_http_config_checked,
    describe_http_security_configuration, format_token_reference, log_headless_http_ready,
    log_headless_http_static_dir, prepare_headless_http_listener,
};
pub use http_dispatch::{
    CommandHandler, CommandRegistry, DispatchResult, RegistryBuilder, dispatch_http_command,
    handle_unsupported, is_supported_http_command, parse_params,
};
pub use panic_report::init_panic_hook;
pub use path_utils::{
    find_executable_in_path, find_root_startup_file, find_root_startup_file_checked,
    get_app_data_dir, get_or_create_app_data_dir, get_or_create_app_data_dir_checked,
    is_windows_absolute_path,
    strip_path_prefix_for_compare, validate_file_name_only,
};
pub use port_usage::{PortUsageKind, is_tcp_port_listening, is_tcp_port_listening_checked};
pub use runtime_mode::RuntimeMode;
pub use server_status::{
    StatusLevel, StatusSnapshot, status_blocks_start, status_detail_field, status_detail_health,
    status_detail_indicates_running, status_detail_runtime_kind,
    status_is_docker_command_ready, status_is_terminal_start_ready,
};
pub use tokio_runtime::{TokioRuntimeConfig, create_tokio_runtime, create_tokio_runtime_or_exit};

#[cfg(test)]
pub(crate) mod test_support {
    use std::ffi::OsString;
    use std::sync::{LazyLock, Mutex, MutexGuard};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
        match ENV_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    pub(crate) struct EnvGuard {
        name: &'static str,
        original: Option<OsString>,
    }

    impl EnvGuard {
        pub(crate) fn set(name: &'static str, value: &str) -> Self {
            let original = std::env::var_os(name);
            std::env::set_var(name, value);
            Self { name, original }
        }

        pub(crate) fn remove(name: &'static str) -> Self {
            let original = std::env::var_os(name);
            std::env::remove_var(name);
            Self { name, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(original) = &self.original {
                std::env::set_var(self.name, original);
            } else {
                std::env::remove_var(self.name);
            }
        }
    }
}
