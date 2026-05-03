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
            let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "/app/dist".to_string());
            let static_dir = Path::new(&static_dir).exists().then_some(static_dir);

            return Self::HeadlessHttp {
                bind_addr: "0.0.0.0:3000".to_string(),
                static_dir,
            };
        }

        Self::Desktop
    }
}
