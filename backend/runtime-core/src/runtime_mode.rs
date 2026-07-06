use crate::{resolve_http_bind_addr, resolve_http_bind_addr_checked};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMode {
    Desktop,
    HeadlessHttp {
        bind_addr: String,
        static_dir: Option<String>,
    },
}

impl RuntimeMode {
    pub fn detect() -> Self {
        if Path::new("/.dockerenv").exists() {
            let static_dir =
                std::env::var("STATIC_DIR").unwrap_or_else(|_| "/app/dist".to_string());
            let static_dir = Path::new(&static_dir).exists().then_some(static_dir);

            return Self::HeadlessHttp {
                bind_addr: resolve_http_bind_addr(3000),
                static_dir,
            };
        }

        Self::Desktop
    }

    pub fn detect_checked() -> Result<Self, String> {
        if Path::new("/.dockerenv").exists() {
            let static_dir =
                std::env::var("STATIC_DIR").unwrap_or_else(|_| "/app/dist".to_string());
            let static_dir = Path::new(&static_dir).exists().then_some(static_dir);

            return Ok(Self::HeadlessHttp {
                bind_addr: resolve_http_bind_addr_checked(3000)?,
                static_dir,
            });
        }

        Ok(Self::Desktop)
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeMode;
    use crate::test_support::{lock_env, EnvGuard};
    use std::path::Path;

    #[test]
    fn headless_http_bind_defaults_to_loopback() {
        let _lock = lock_env();
        let _http_bind = EnvGuard::remove("SEALANTERN_HTTP_BIND");
        let _web_bind = EnvGuard::remove("SEALANTERN_WEB_BIND");

        let mode = RuntimeMode::detect();
        if Path::new("/.dockerenv").exists() {
            assert_eq!(
                mode,
                RuntimeMode::HeadlessHttp {
                    bind_addr: "127.0.0.1:3000".to_string(),
                    static_dir: std::env::var("STATIC_DIR")
                        .ok()
                        .filter(|dir| std::path::Path::new(dir).exists()),
                }
            );
        } else {
            assert_eq!(mode, RuntimeMode::Desktop);
        }
    }

    #[test]
    fn headless_http_bind_prefers_explicit_http_bind() {
        let _lock = lock_env();
        let _http_bind = EnvGuard::set("SEALANTERN_HTTP_BIND", "0.0.0.0:3000");
        let _web_bind = EnvGuard::set("SEALANTERN_WEB_BIND", "127.0.0.1");

        if let RuntimeMode::HeadlessHttp { bind_addr, .. } = RuntimeMode::detect() {
            assert_eq!(bind_addr, "0.0.0.0:3000");
        }
    }

    #[test]
    fn headless_http_detect_checked_rejects_invalid_bind_when_containerized() {
        let _lock = lock_env();
        let _http_bind = EnvGuard::set("SEALANTERN_HTTP_BIND", "localhost:notaport");
        let _web_bind = EnvGuard::remove("SEALANTERN_WEB_BIND");

        if Path::new("/.dockerenv").exists() {
            let error = RuntimeMode::detect_checked()
                .expect_err("checked runtime mode detection should reject invalid bind config");

            assert!(error.contains("HTTP 绑定地址无效"), "unexpected error: {}", error);
            assert_eq!(
                RuntimeMode::detect(),
                RuntimeMode::HeadlessHttp {
                    bind_addr: "127.0.0.1:3000".to_string(),
                    static_dir: std::env::var("STATIC_DIR")
                        .ok()
                        .filter(|dir| std::path::Path::new(dir).exists()),
                }
            );
        } else {
            assert_eq!(RuntimeMode::detect_checked().unwrap(), RuntimeMode::Desktop);
        }
    }
}
