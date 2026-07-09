pub(super) fn run_async_cli_task<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    let runtime = sea_lantern_runtime::create_tokio_runtime_or_exit(
        sea_lantern_runtime::TokioRuntimeConfig {
            error_prefix: "Failed to create Tokio runtime",
            error_hint: Some("This may be due to system resource limits."),
        },
    );

    runtime.block_on(future);
}
