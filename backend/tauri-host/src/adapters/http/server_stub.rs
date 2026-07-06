/// 非 docker 构建下不启动 HTTP 服务
pub async fn run_http_server(
    _addr: &str,
    _static_dir: Option<String>,
    _startup_notifier: Option<std::sync::mpsc::Sender<Result<(), String>>>,
) -> Result<(), String> {
    Err("headless HTTP runtime is unavailable without the 'docker' feature".to_string())
}
