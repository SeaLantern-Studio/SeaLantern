use crate::log_fatal_ctx;

pub struct TokioRuntimeConfig<'a> {
    pub error_prefix: &'a str,
    pub error_hint: Option<&'a str>,
}

pub fn create_tokio_runtime(
    config: TokioRuntimeConfig<'_>,
) -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Runtime::new()
        .map_err(|error| format_runtime_creation_message(&config, &error.to_string()))
}

pub fn create_tokio_runtime_or_exit(config: TokioRuntimeConfig<'_>) -> tokio::runtime::Runtime {
    match create_tokio_runtime(config) {
        Ok(runtime) => runtime,
        Err(message) => {
            log_fatal_ctx("runtime.tokio", "create_tokio_runtime_or_exit", &message);
            for line in message.lines() {
                std::eprintln!("{}", line);
            }
            std::process::exit(1);
        }
    }
}

fn format_runtime_creation_message(config: &TokioRuntimeConfig<'_>, error_message: &str) -> String {
    let mut message = format!("{}: {}", config.error_prefix, error_message);
    if let Some(hint) = config.error_hint {
        message.push('\n');
        message.push_str(hint);
    }
    message
}

#[cfg(test)]
mod tests {
    use super::{format_runtime_creation_message, TokioRuntimeConfig};

    #[test]
    fn formats_runtime_creation_message_with_optional_hint() {
        let config = TokioRuntimeConfig {
            error_prefix: "prefix",
            error_hint: Some("hint"),
        };

        let message = format_runtime_creation_message(&config, "boom");
        assert_eq!(message, "prefix: boom\nhint");
    }

    #[test]
    fn formats_runtime_creation_message_without_hint() {
        let config = TokioRuntimeConfig { error_prefix: "prefix", error_hint: None };

        let message = format_runtime_creation_message(&config, "boom");
        assert_eq!(message, "prefix: boom");
    }
}
