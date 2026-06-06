pub(super) fn run_async_cli_task<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("Failed to create Tokio runtime: {}", err);
            eprintln!("This may be due to system resource limits.");
            std::process::exit(1);
        }
    };

    runtime.block_on(future);
}
