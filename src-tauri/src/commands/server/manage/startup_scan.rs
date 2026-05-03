mod archive;
mod common;
mod folder;

use crate::models::server::StartupScanResult;
use std::path::Path;

/// 扫描启动候选项
///
/// 会在线程池里执行阻塞扫描，避免卡住异步运行时
pub(super) async fn scan_startup_candidates(
    source_path: String,
    source_type: String,
) -> Result<StartupScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        scan_startup_candidates_blocking(source_path, source_type)
    })
    .await
    .map_err(|e| format!("扫描启动项任务失败: {}", e))?
}

fn scan_startup_candidates_blocking(
    source_path: String,
    source_type: String,
) -> Result<StartupScanResult, String> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err(format!("路径不存在: {}", source_path));
    }

    let source_kind = source_type.to_ascii_lowercase();

    if source_kind == "archive" {
        return archive::scan_archive_source(source_path);
    }

    if source_kind != "folder" {
        return Err("来源类型无效，仅支持 archive 或 folder".to_string());
    }

    folder::scan_folder_source(source)
}
