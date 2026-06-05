use std::path::{Path, PathBuf};

fn looks_like_existing_server_folder(path: &Path) -> bool {
    let Some(last_comp) = path.components().next_back() else {
        return false;
    };

    let comp_str = last_comp.as_os_str().to_string_lossy();
    let is_plain_hex = comp_str.len() == 32 && comp_str.chars().all(|c| c.is_ascii_hexdigit());
    let is_uuid_like = comp_str.len() == 36
        && comp_str.chars().take(32).all(|c| c.is_ascii_hexdigit())
        && comp_str.chars().nth(8) == Some('-')
        && comp_str.chars().nth(13) == Some('-')
        && comp_str.chars().nth(18) == Some('-')
        && comp_str.chars().nth(23) == Some('-');

    is_plain_hex || is_uuid_like
}

fn new_generated_server_dir(base_path: &str) -> PathBuf {
    let folder_name = uuid::Uuid::new_v4().to_string().replace('-', "")[..30].to_string();
    PathBuf::from(base_path).join(&folder_name)
}

pub(super) fn resolve_modpack_run_dir(base_path: &str) -> Result<PathBuf, String> {
    let trimmed = base_path.trim();
    if trimmed.is_empty() {
        return Err("运行目录不能为空，请选择开服路径".to_string());
    }

    let run_dir = if trimmed.contains('/') || trimmed.contains('\\') {
        let path = PathBuf::from(trimmed);
        if looks_like_existing_server_folder(&path) {
            path
        } else {
            new_generated_server_dir(trimmed)
        }
    } else {
        new_generated_server_dir(trimmed)
    };

    if run_dir.exists() {
        return Err(format!(
            "目录已存在：{}，请更换启动项或选择其他路径",
            run_dir.to_string_lossy()
        ));
    }

    Ok(run_dir)
}
