//! 配置/数据目录迁移逻辑。
//!
//! 负责在启动时检测旧版 `data_dir.json` 定位器文件，
//! 将数据一次性回迁到默认目录后清理定位器。
//! 后续版本不再支持自定义路径定位，仅保留环境变量控制。

use std::path::PathBuf;

use crate::observability;
use sealantern_infra::platform::get_app_data_dir;

const APP_DATA_LOCATOR_FILE: &str = "data_dir.json";

/// 运行启动迁移：检测旧版定位器，回迁到默认目录。
///
/// 如果默认数据目录下存在 `data_dir.json`，说明用户曾使用 v1.3.0 的
/// 定位器功能指定了自定义数据目录。此函数将该目录下的内容搬回默认目录，
/// 然后删除定位器文件。此后启动均走默认路径。
///
/// 环境变量 `SEALANTERN_DATA_DIR` 不受此迁移影响（优先级更高）。
pub fn run_startup_migration() {
    let default_dir = get_app_data_dir();
    let locator_path = default_dir.join(APP_DATA_LOCATOR_FILE);

    if !locator_path.exists() {
        return;
    }

    // 读取定位器中的旧数据目录
    let old_dir = match read_locator(&locator_path) {
        Some(dir) if dir != default_dir => dir,
        None => {
            observability::config_locator_unreadable(&locator_path);
            return;
        }
        _ => {
            // 定位器指向的就是默认目录，不需要迁移，清理文件即可
            if let Err(e) = std::fs::remove_file(&locator_path) {
                observability::config_locator_cleanup_failed(&locator_path, &e);
            }
            return;
        }
    };

    observability::config_migration_started(&old_dir, &default_dir);

    // 搬迁数据：将旧目录下的内容复制到默认目录
    if let Err(e) = migrate_data_dir(&old_dir, &default_dir) {
        observability::config_migration_failed(&e);
        return;
    }

    // 删除定位器文件
    if let Err(e) = std::fs::remove_file(&locator_path) {
        observability::config_locator_cleanup_failed(&locator_path, &e);
    }

    observability::config_migration_completed();
}

/// 从定位器文件读取自定义路径
fn read_locator(path: &std::path::Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    let dir = parsed.get("data_dir")?.as_str()?;
    let trimmed = dir.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

/// 搬迁数据目录内容（复制 + 删除旧源）
fn migrate_data_dir(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if !src.exists() {
        return Ok(()); // 旧目录不存在，无需搬迁
    }

    // 确保目标目录存在
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("无法创建目标目录 '{}': {}", dst.display(), e))?;

    // 复制每个条目
    let entries =
        std::fs::read_dir(src).map_err(|e| format!("无法读取旧目录 '{}': {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件 '{}' 失败: {}", src_path.display(), e))?;
        }
    }

    // 删除旧目录
    std::fs::remove_dir_all(src)
        .map_err(|e| format!("无法删除旧目录 '{}': {}", src.display(), e))?;

    Ok(())
}

/// 递归复制目录
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("无法创建目录 '{}': {}", dst.display(), e))?;

    let entries =
        std::fs::read_dir(src).map_err(|e| format!("无法读取目录 '{}': {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("复制文件 '{}' 失败: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}
