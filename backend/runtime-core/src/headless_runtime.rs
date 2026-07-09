use crate::{create_tokio_runtime_or_exit, log_fatal_ctx, log_info_ctx, TokioRuntimeConfig};
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
    std::eprintln!("{}", config.startup_message);
    log_info_ctx("runtime.headless_runtime", "run_tokio_service", &config.startup_message);

    let rt = create_tokio_runtime_or_exit(TokioRuntimeConfig {
        error_prefix: config.runtime_creation_error_prefix,
        error_hint: config.runtime_creation_error_hint,
    });

    if let Err(error) = rt.block_on(run()) {
        log_fatal_ctx(
            "runtime.headless_runtime",
            "run_tokio_service",
            &format!("{}: {}", config.service_error_prefix, error),
        );
        std::process::exit(1);
    }
}
