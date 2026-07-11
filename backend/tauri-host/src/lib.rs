// License: GPL-3.0-only. Copyright (C) SeaLantern Studio.
mod adapters;
mod commands;
mod hardcode_data;
mod models;
pub mod plugins;
mod runtime;
mod services;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Desktop application entry point.
pub fn run() {
    runtime::run();
}

/// Headless HTTP entry point used by Docker and external host processes.
pub fn run_headless_http() {
    runtime::run_headless_http();
}

#[cfg(test)]
pub(crate) mod test_support {
    use once_cell::sync::Lazy;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
            if let Some(value) = &self.original {
                std::env::set_var(self.name, value);
            } else {
                std::env::remove_var(self.name);
            }
        }
    }
}
