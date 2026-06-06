use crate::models::server::StartupScanResult;

/// 扫描启动候选项
///
/// 会在线程池里执行阻塞扫描，避免卡住异步运行时
pub(super) async fn scan_startup_candidates(
    source_path: String,
    source_type: String,
) -> Result<StartupScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        sea_lantern_server_startup_scan_core::scan_startup_candidates(
            &source_path,
            &source_type,
            &crate::utils::constants::STARTER_MC_VERSION_OPTIONS,
        )
    })
    .await
    .map_err(|e| format!("扫描启动项任务失败: {}", e))?
}
