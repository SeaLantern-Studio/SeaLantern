use crate::{TokioRuntimeConfig, capture_eprintln, create_tokio_runtime_or_exit};
use std::future::Future;

pub struct TokioServiceConfig<'a> {
    pub startup_message: String,
    pub runtime_creation_error_prefix: &'a str,
    pub runtime_creation_error_hint: Option<&'a str>,
    pub service_error_prefix: &'a str,
}

pub fn run_tokio_service<F, Fut>(config: TokioServiceConfig<'_>, run: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    capture_eprintln(config.startup_message.clone());

    let rt = create_tokio_runtime_or_exit(TokioRuntimeConfig {
        error_prefix: config.runtime_creation_error_prefix,
        error_hint: config.runtime_creation_error_hint,
    });

    if let Err(error) = rt.block_on(run()) {
        capture_eprintln(format!("{}: {}", config.service_error_prefix, error));
        std::process::exit(1);
    }
}
