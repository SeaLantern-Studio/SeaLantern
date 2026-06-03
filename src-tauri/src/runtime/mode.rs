use std::path::Path;

/// Describes how the backend process should boot in the current environment.
pub enum RuntimeMode {
    /// Start the embedded desktop application with the native Tauri host.
    Desktop,
    /// Start the headless HTTP transport used by Docker and future external WebUI modes.
    HeadlessHttp {
        /// Socket address the HTTP adapter should bind to.
        bind_addr: String,
        /// Optional directory used to serve prebuilt frontend assets.
        static_dir: Option<String>,
    },
}

impl RuntimeMode {
    /// Detects the effective runtime mode from the current process environment.
    ///
    /// # Returns
    ///
    /// The runtime mode that should be used for this process launch.
    pub fn detect() -> Self {
        if Path::new("/.dockerenv").exists() {
            let static_dir =
                std::env::var("STATIC_DIR").unwrap_or_else(|_| "/app/dist".to_string());
            let static_dir = Path::new(&static_dir).exists().then_some(static_dir);

            return Self::HeadlessHttp {
                bind_addr: resolve_headless_http_bind_addr(),
                static_dir,
            };
        }

        Self::Desktop
    }
}

fn resolve_headless_http_bind_addr() -> String {
    crate::adapters::http::resolve_http_bind_addr(3000)
}

#[cfg(test)]
mod tests {
    use super::resolve_headless_http_bind_addr;
    use std::sync::Mutex;

    static ENV_LOCK: once_cell::sync::Lazy<Mutex<()>> =
        once_cell::sync::Lazy::new(|| Mutex::new(()));

    struct EnvGuard {
        name: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let original = std::env::var(name).ok();
            std::env::set_var(name, value);
            Self { name, original }
        }

        fn remove(name: &'static str) -> Self {
            let original = std::env::var(name).ok();
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

    #[test]
    fn headless_http_bind_defaults_to_loopback() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _http_bind = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web_bind = EnvGuard::remove("SEALANTERN_WEB_BIND");

        assert_eq!(resolve_headless_http_bind_addr(), "127.0.0.1:3000");
    }

    #[test]
    fn headless_http_bind_prefers_explicit_http_bind() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _http_bind = EnvGuard::set("SEALANTERN_HTTP_BIND", "0.0.0.0:3000");
        let _web_bind = EnvGuard::set("SEALANTERN_WEB_BIND", "127.0.0.1");

        assert_eq!(resolve_headless_http_bind_addr(), "0.0.0.0:3000");
    }

    #[test]
    fn headless_http_bind_uses_web_bind_as_fallback() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _http_bind = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web_bind = EnvGuard::set("SEALANTERN_WEB_BIND", "0.0.0.0");

        assert_eq!(resolve_headless_http_bind_addr(), "0.0.0.0:3000");
    }
}
