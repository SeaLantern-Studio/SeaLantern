use std::fs;
use std::path::PathBuf;

use chrono::Utc;

/// 构造崩溃日志的完整输出路径
pub(super) fn build_report_path(now: &chrono::DateTime<Utc>) -> std::io::Result<PathBuf> {
    let base_dir = option_env!("CARGO_MANIFEST_DIR")
        .and_then(|manifest| PathBuf::from(manifest).parent().map(|path| path.to_path_buf()))
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(|dir| dir.to_path_buf()))
        })
        .unwrap_or_else(|| PathBuf::from("."));

    let log_dir = base_dir.join("panic-log");
    fs::create_dir_all(&log_dir)?;

    let file_name = now.format("panic_%Y%m%d_%H%M%S_%3f.log").to_string();
    Ok(log_dir.join(file_name))
}
