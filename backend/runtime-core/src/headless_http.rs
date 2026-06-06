use crate::{capture_eprintln, capture_println};
use http::{HeaderValue, header::AUTHORIZATION};
use sha2::{Digest, Sha256};
use std::{path::{Path, PathBuf}, sync::mpsc::Sender};
use tokio::{fs, net::TcpListener};
use uuid::Uuid;

pub(crate) const DEFAULT_UPLOAD_DIR: &str = "/app/uploads";
pub(crate) const DEFAULT_MAX_UPLOAD_BYTES: usize = 500 * 1024 * 1024;
pub(crate) const DEFAULT_MAX_UPLOAD_FILE_BYTES: usize = 100 * 1024 * 1024;
pub(crate) const DEFAULT_MAX_UPLOAD_FILES: usize = 16;
pub const HTTP_AUTH_TOKEN_ENV: &str = "SEALANTERN_HTTP_AUTH_TOKEN";
pub const HTTP_CORS_ORIGINS_ENV: &str = "SEALANTERN_HTTP_CORS_ORIGINS";

#[derive(Clone, Debug)]
pub struct HeadlessHttpConfig {
    pub auth_token: String,
    pub upload_dir: PathBuf,
    pub cors_allowed_origins: Vec<HeaderValue>,
    pub max_upload_bytes: usize,
    pub max_upload_file_bytes: usize,
    pub max_upload_files: usize,
}

pub fn default_headless_http_config() -> HeadlessHttpConfig {
    default_headless_http_config_checked().unwrap_or_else(|_| HeadlessHttpConfig {
        auth_token: env_var_trimmed(HTTP_AUTH_TOKEN_ENV)
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        upload_dir: PathBuf::from(DEFAULT_UPLOAD_DIR),
        cors_allowed_origins: Vec::new(),
        max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
        max_upload_file_bytes: DEFAULT_MAX_UPLOAD_FILE_BYTES,
        max_upload_files: DEFAULT_MAX_UPLOAD_FILES,
    })
}

pub fn default_headless_http_config_checked() -> Result<HeadlessHttpConfig, String> {
    let configured_token = env_var_trimmed(HTTP_AUTH_TOKEN_ENV);
    let auth_token = configured_token.unwrap_or_else(|| Uuid::new_v4().to_string());

    Ok(HeadlessHttpConfig {
        auth_token,
        upload_dir: PathBuf::from(DEFAULT_UPLOAD_DIR),
        cors_allowed_origins: parse_cors_allowed_origins_checked()?,
        max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
        max_upload_file_bytes: DEFAULT_MAX_UPLOAD_FILE_BYTES,
        max_upload_files: DEFAULT_MAX_UPLOAD_FILES,
    })
}

pub async fn prepare_headless_http_listener(
    addr: &str,
    config: &HeadlessHttpConfig,
    startup_notifier: Option<Sender<Result<(), String>>>,
) -> Result<TcpListener, String> {
    if let Err(error) = fs::create_dir_all(&config.upload_dir).await {
        let message = format_upload_dir_preparation_error(&config.upload_dir, &error.to_string());
        if let Some(notifier) = startup_notifier {
            let _ = notifier.send(Err(message.clone()));
        }
        capture_eprintln(message.clone());
        return Err(message);
    }

    for message in describe_http_security_configuration(config) {
        capture_println(message);
    }

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            let message = format_http_bind_error(addr, &error.to_string());
            if let Some(notifier) = startup_notifier {
                let _ = notifier.send(Err(message.clone()));
            }
            capture_eprintln(message.clone());
            return Err(message);
        }
    };

    if let Some(notifier) = startup_notifier {
        let _ = notifier.send(Ok(()));
    }

    Ok(listener)
}

pub fn log_headless_http_ready(addr: &str) {
    for message in describe_http_ready_messages(addr) {
        capture_println(message);
    }
}

pub fn log_headless_http_static_dir(dir: &str) {
    capture_println(format!("Serving static files from: {} (SPA fallback enabled)", dir));
}

pub fn describe_http_security_configuration(config: &HeadlessHttpConfig) -> Vec<String> {
    let token_reference = format_token_reference(&config.auth_token);
    let mut messages = Vec::new();

    if std::env::var(HTTP_AUTH_TOKEN_ENV).is_ok() {
        messages.push(format!(
            "SeaLantern HTTP auth enabled with configured token {}",
            token_reference
        ));
    } else {
        messages.push(format!(
            "SeaLantern HTTP auth generated a process-local token {}",
            token_reference
        ));
        messages.push(format!(
            "Set '{}' explicitly for a stable token; otherwise use the in-process generated token with header '{}: Bearer <token>' when calling /api/*, /upload, or /api/logs/stream",
            HTTP_AUTH_TOKEN_ENV,
            AUTHORIZATION.as_str(),
        ));
    }

    if config.cors_allowed_origins.is_empty() {
        messages.push(
            "SeaLantern HTTP CORS disabled by default; set SEALANTERN_HTTP_CORS_ORIGINS to allow browser origins"
                .to_string(),
        );
    } else {
        messages.push(format!(
            "SeaLantern HTTP CORS allowlist enabled for {} origin(s)",
            config.cors_allowed_origins.len()
        ));
    }

    messages
}

pub fn format_token_reference(token: &str) -> String {
    format!("prefix={} fingerprint={}", token_prefix(token), token_fingerprint(token))
}

fn describe_http_ready_messages(addr: &str) -> Vec<String> {
    vec![
        format!("SeaLantern HTTP server listening on {}", addr),
        format!("API endpoints available at http://{}/api/<command>", addr),
        format!("Health check at http://{}/health", addr),
        format!("File upload available at http://{}/upload", addr),
    ]
}

fn format_http_bind_error(addr: &str, error_message: &str) -> String {
    format!("SeaLantern HTTP server failed to bind at {}: {}", addr, error_message)
}

fn format_upload_dir_preparation_error(upload_dir: &Path, error_message: &str) -> String {
    format!(
        "Failed to create upload directory '{}': {}",
        upload_dir.display(),
        error_message
    )
}

fn parse_cors_allowed_origins_checked() -> Result<Vec<HeaderValue>, String> {
    let Some(value) = env_var_trimmed(HTTP_CORS_ORIGINS_ENV) else {
        return Ok(Vec::new());
    };

    let mut origins = Vec::new();
    for origin in value.split(',') {
        let trimmed = origin.trim();
        if trimmed.is_empty() {
            continue;
        }

        let header = HeaderValue::from_str(trimmed)
            .map_err(|e| format!("CORS origin 无效 '{}': {}", trimmed, e))?;
        origins.push(header);
    }

    Ok(origins)
}

fn env_var_trimmed(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn token_fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest[..6]
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

fn token_prefix(token: &str) -> &str {
    let len = token.len().min(8);
    &token[..len]
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_MAX_UPLOAD_BYTES, DEFAULT_MAX_UPLOAD_FILE_BYTES, DEFAULT_MAX_UPLOAD_FILES,
        DEFAULT_UPLOAD_DIR, HTTP_AUTH_TOKEN_ENV, HTTP_CORS_ORIGINS_ENV,
        default_headless_http_config, default_headless_http_config_checked,
        describe_http_security_configuration,
        parse_cors_allowed_origins_checked, prepare_headless_http_listener,
        describe_http_ready_messages, format_token_reference,
    };
    use crate::test_support::{lock_env, EnvGuard};
    use std::{fs as std_fs, path::PathBuf, sync::mpsc, time::{SystemTime, UNIX_EPOCH}};
    use tokio::runtime::Runtime;

    #[test]
    fn default_headless_http_config_uses_expected_defaults() {
        let _lock = lock_env();
        let _auth_guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _cors_guard = EnvGuard::remove(HTTP_CORS_ORIGINS_ENV);

        let config = default_headless_http_config();
        assert!(!config.auth_token.is_empty());
        assert_eq!(config.upload_dir, PathBuf::from(DEFAULT_UPLOAD_DIR));
        assert!(config.cors_allowed_origins.is_empty());
        assert_eq!(config.max_upload_bytes, DEFAULT_MAX_UPLOAD_BYTES);
        assert_eq!(config.max_upload_file_bytes, DEFAULT_MAX_UPLOAD_FILE_BYTES);
        assert_eq!(config.max_upload_files, DEFAULT_MAX_UPLOAD_FILES);
    }

    #[test]
    fn cors_allowlist_parses_comma_separated_origins() {
        let _lock = lock_env();
        let _cors_guard = EnvGuard::set(
            HTTP_CORS_ORIGINS_ENV,
            " https://example.com, ,https://second.example ",
        );

        let config = default_headless_http_config();
        assert_eq!(config.cors_allowed_origins.len(), 2);
        assert_eq!(
            config.cors_allowed_origins[0].to_str().unwrap(),
            "https://example.com"
        );
        assert_eq!(
            config.cors_allowed_origins[1].to_str().unwrap(),
            "https://second.example"
        );
    }

    #[test]
    fn cors_allowlist_checked_surfaces_invalid_origin_value() {
        let _lock = lock_env();
        let _cors_guard = EnvGuard::set(HTTP_CORS_ORIGINS_ENV, "https://ok.example, bad\nvalue");

        let error = parse_cors_allowed_origins_checked()
            .expect_err("checked CORS parsing should surface invalid header values");

        assert!(error.contains("CORS origin 无效"), "unexpected error: {}", error);
        assert!(error.contains("bad"), "unexpected error: {}", error);
    }

    #[test]
    fn default_headless_http_config_checked_surfaces_invalid_cors_env() {
        let _lock = lock_env();
        let _cors_guard = EnvGuard::set(HTTP_CORS_ORIGINS_ENV, "https://ok.example, bad\nvalue");

        let error = default_headless_http_config_checked()
            .expect_err("checked default config should reject invalid CORS env");

        assert!(error.contains("CORS origin 无效"), "unexpected error: {}", error);
    }

    #[test]
    fn legacy_default_headless_http_config_still_downgrades_invalid_cors_env() {
        let _lock = lock_env();
        let _cors_guard = EnvGuard::set(HTTP_CORS_ORIGINS_ENV, "https://ok.example, bad\nvalue");

        let config = default_headless_http_config();

        assert!(config.cors_allowed_origins.is_empty());
    }

    #[test]
    fn security_messages_do_not_leak_full_generated_token() {
        let _lock = lock_env();
        let _auth_guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _cors_guard = EnvGuard::remove(HTTP_CORS_ORIGINS_ENV);

        let config = default_headless_http_config();
        let messages = describe_http_security_configuration(&config);
        assert!(messages.iter().all(|message| !message.contains(&config.auth_token)));
        assert!(messages
            .iter()
            .any(|message| message.contains("process-local token prefix=")));
    }

    #[test]
    fn token_reference_uses_prefix_and_fingerprint_without_full_value() {
        let token = "12345678-abcdef-full-secret-token";
        let reference = format_token_reference(token);

        assert!(reference.contains("prefix=12345678"));
        assert!(reference.contains("fingerprint="));
        assert!(!reference.contains(token));
    }

    #[test]
    fn ready_messages_list_expected_endpoints() {
        let messages = describe_http_ready_messages("127.0.0.1:3000");

        assert_eq!(messages.len(), 4);
        assert!(messages[0].contains("127.0.0.1:3000"));
        assert!(messages[1].contains("/api/<command>"));
        assert!(messages[2].contains("/health"));
        assert!(messages[3].contains("/upload"));
    }

    #[test]
    fn prepare_listener_rejects_upload_dir_path_that_is_a_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let upload_path = std::env::temp_dir().join(format!("sealantern-upload-dir-test-{}.tmp", unique));
        std_fs::write(&upload_path, b"occupied by file").expect("create placeholder file");
        let (tx, rx) = mpsc::channel();
        let config = super::HeadlessHttpConfig {
            auth_token: "test-token".to_string(),
            upload_dir: upload_path.clone(),
            cors_allowed_origins: Vec::new(),
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            max_upload_file_bytes: DEFAULT_MAX_UPLOAD_FILE_BYTES,
            max_upload_files: DEFAULT_MAX_UPLOAD_FILES,
        };

        let runtime = Runtime::new().expect("tokio runtime");
        let error = runtime
            .block_on(prepare_headless_http_listener("127.0.0.1:0", &config, Some(tx)))
            .expect_err("upload dir preparation failure should abort startup");

        assert!(error.contains("Failed to create upload directory"), "unexpected error: {}", error);
        assert!(error.contains(upload_path.to_string_lossy().as_ref()), "unexpected error: {}", error);

        let startup = rx.recv().expect("startup result");
        let startup_error = startup.expect_err("startup notifier should receive failure");
        assert_eq!(startup_error, error);

        std_fs::remove_file(&upload_path).expect("cleanup placeholder file");
    }
}
