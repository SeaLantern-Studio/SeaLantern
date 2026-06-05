use crate::hardcode_data::app_files::APP_DIRECTORY_NAME;

pub fn open_file(path: String) -> Result<(), String> {
    opener::open(&path)
        .map(|_| ())
        .map_err(|e| format!("打开文件失败: {}", e))
}

pub fn open_folder(path: String) -> Result<(), String> {
    opener::open(&path)
        .map(|_| ())
        .map_err(|e| format!("打开文件夹失败: {}", e))
}

pub fn get_default_run_path() -> Result<String, String> {
    let base = dirs_next::data_dir()
        .or_else(dirs_next::document_dir)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| "无法确定默认运行路径".to_string())?;

    Ok(base.join(APP_DIRECTORY_NAME).to_string_lossy().to_string())
}

pub fn append_generated_server_dir(base_path: &str) -> String {
    let folder_name = uuid::Uuid::new_v4().to_string().replace('-', "")[..30].to_string();
    std::path::PathBuf::from(base_path)
        .join(folder_name)
        .to_string_lossy()
        .to_string()
}

pub fn get_safe_mode_status() -> Result<bool, String> {
    Ok(std::env::args().any(|arg| arg == "--safe-mode"))
}

pub fn frontend_heartbeat() -> Result<(), String> {
    crate::services::global::update_frontend_heartbeat();
    Ok(())
}
