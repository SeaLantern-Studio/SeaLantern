use chrono::Utc;
use std::fs;
use std::io;
use std::path::PathBuf;

pub(super) fn build_report_path(now: &chrono::DateTime<Utc>) -> std::io::Result<PathBuf> {
    build_report_path_with(now, std::env::current_exe)
}

fn build_report_path_with<F>(
    now: &chrono::DateTime<Utc>,
    current_exe: F,
) -> std::io::Result<PathBuf>
where
    F: FnOnce() -> std::io::Result<PathBuf>,
{
    let base_dir = option_env!("CARGO_MANIFEST_DIR")
        .and_then(|manifest| PathBuf::from(manifest).parent().map(|path| path.to_path_buf()))
        .or_else(|| match current_exe() {
            Ok(path) => path.parent().map(|dir| dir.to_path_buf()),
            Err(_) => None,
        })
        .ok_or_else(|| io::Error::other("无法确定 panic 日志目录"))?;

    let log_dir = base_dir.join("panic-log");
    fs::create_dir_all(&log_dir)?;

    let file_name = now.format("panic_%Y%m%d_%H%M%S_%3f.log").to_string();
    Ok(log_dir.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::build_report_path_with;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn build_report_path_uses_manifest_parent_when_available() {
        let now = Utc::now();
        let report_path = build_report_path_with(&now, || {
            Err(std::io::Error::other("exe unavailable"))
        })
        .expect("manifest parent should provide a usable panic log directory");

        assert!(report_path.file_name().and_then(|name| name.to_str()).is_some());
        assert_eq!(report_path.parent().and_then(|dir| dir.file_name()).and_then(|name| name.to_str()), Some("panic-log"));
    }

    #[test]
    fn build_report_path_surfaces_current_exe_failure_when_manifest_parent_missing() {
        let now = Utc::now();
        let report_path = build_report_path_with(&now, || Ok(PathBuf::from("runtime-core.exe")));

        if option_env!("CARGO_MANIFEST_DIR").is_none() {
            let error = report_path.expect_err("missing manifest parent should surface current_exe failure path");
            assert!(error.to_string().contains("无法确定 panic 日志目录"));
        } else {
            assert!(report_path.is_ok());
        }
    }
}
