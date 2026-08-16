//! 配置/数据目录迁移逻辑。
//!
//! 负责在启动时检测旧版 `data_dir.json` 定位器文件，
//! 将数据一次性回迁到默认目录后清理定位器。
//! 后续版本不再支持自定义路径定位，仅保留环境变量控制。
//!
//! 迁移安全策略：
//! - 迁移前校验源/目标目录无包含关系，防止递归复制或误删数据树；
//! - 先将旧目录重命名为临时迁移源（同文件系统原子操作），
//!   复制失败时回滚重命名，保证用户数据始终可恢复；
//! - 跳过符号链接条目，避免跟随链接递归或复制链接指向的目录。

use std::path::{Path, PathBuf};

use crate::observability;
use sealantern_infra::platform::get_app_data_dir;

const APP_DATA_LOCATOR_FILE: &str = "data_dir.json";

/// 迁移过程的条目统计。
#[derive(Debug, Default, Clone, Copy)]
struct MigrationStats {
    /// 复制的文件数。
    files_copied: usize,
    /// 复制的目录数。
    dirs_copied: usize,
    /// 目标已存在同名条目而覆盖的冲突数。
    conflicts: usize,
}

/// 运行启动迁移：检测旧版定位器，回迁到默认目录。
///
/// 如果默认数据目录下存在 `data_dir.json`，说明用户曾使用 v1.3.0 的
/// 定位器功能指定了自定义数据目录。此函数将该目录下的内容搬回默认目录，
/// 然后删除定位器文件。此后启动均走默认路径。
///
/// 环境变量 `SEALANTERN_DATA_DIR` 不受此迁移影响（优先级更高）。
///
/// 迁移使用异步文件 I/O，避免大数据目录阻塞启动线程。
pub async fn run_startup_migration() {
    let default_dir = get_app_data_dir();
    let locator_path = default_dir.join(APP_DATA_LOCATOR_FILE);

    if !locator_path.exists() {
        return;
    }

    // 读取定位器中的旧数据目录
    let old_dir = match read_locator(&locator_path).await {
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
    match migrate_data_dir(&old_dir, &default_dir).await {
        Ok(stats) => {
            observability::config_migration_summary(stats.files_copied, stats.dirs_copied);

            // 删除定位器文件
            if let Err(e) = std::fs::remove_file(&locator_path) {
                observability::config_locator_cleanup_failed(&locator_path, &e);
            } else {
                observability::config_migration_completed();
            }
        }
        Err(e) => {
            observability::config_migration_failed(&e);
        }
    }
}

/// 从定位器文件读取自定义路径
async fn read_locator(path: &Path) -> Option<PathBuf> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    let dir = parsed.get("data_dir")?.as_str()?;
    let trimmed = dir.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

/// 搬迁数据目录内容（重命名源目录 + 复制 + 清理迁移源）。
///
/// 步骤：
/// 1. 恢复上次中断迁移留下的残留目录（若存在）；
/// 2. 校验源/目标目录无包含关系（S1）；
/// 3. 将旧目录重命名为 `{name}.migrating-{时间戳}`（同文件系统原子操作），
///    作为迁移源——此后任何失败都不会影响原始数据；
/// 4. 从迁移源复制到目标目录；
/// 5. 复制成功则清理迁移源（清理失败仅告警，数据已完整迁移）；
///    复制失败则回滚重命名，保留旧目录原样（回滚失败必须告警）。
async fn migrate_data_dir(src: &Path, dst: &Path) -> Result<MigrationStats, String> {
    let mut stats = MigrationStats::default();

    // 崩溃残留恢复：上次迁移中断时 src 可能已被重命名走，先恢复原名再继续，
    // 否则上层会误判"无需迁移"并删除定位器，导致旧数据永久失联
    recover_interrupted_migration(src).await?;

    if !src.exists() {
        return Ok(stats); // 旧目录不存在，无需搬迁
    }

    // S1: 源/目标不能存在包含关系，否则递归复制会膨胀、删除会误删数据树
    if src.starts_with(dst) || dst.starts_with(src) {
        observability::config_migration_invalid_path(src, dst);
        return Err(format!(
            "源目录 '{}' 与目标目录 '{}' 存在包含关系，已拒绝迁移",
            src.display(),
            dst.display()
        ));
    }

    // 确保目标目录存在
    tokio::fs::create_dir_all(dst)
        .await
        .map_err(|e| format!("无法创建目标目录 '{}': {}", dst.display(), e))?;

    // S2: 将旧目录重命名为临时迁移源（同文件系统内的原子操作）
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let src_name = src.file_name().and_then(|n| n.to_str()).unwrap_or("data");
    let staged = src.with_file_name(format!("{src_name}.migrating-{timestamp}"));
    tokio::fs::rename(src, &staged)
        .await
        .map_err(|e| format!("无法重命名旧目录 '{}': {}", src.display(), e))?;

    // 从迁移源复制到目标
    match copy_dir_recursive(staged.clone(), dst.to_path_buf()).await {
        Ok(sub) => {
            stats = sub;

            // 复制成功，清理迁移源；清理失败不影响迁移结果（数据已在目标），
            // 仅告警，残留目录留待下次启动时由残留恢复逻辑处理
            if let Err(e) = tokio::fs::remove_dir_all(&staged).await {
                observability::config_migration_cleanup_failed(&staged, &e);
            }

            Ok(stats)
        }
        Err(e) => {
            // 复制失败：回滚重命名，保留旧目录原样；回滚失败必须告警，
            // 否则用户数据会一直困在迁移源目录
            if let Err(rollback_err) = tokio::fs::rename(&staged, src).await {
                observability::config_migration_rollback_failed(&staged, src, &rollback_err);
            }
            Err(e)
        }
    }
}

/// 检测并恢复上次迁移中断留下的迁移源残留。
///
/// "重命名 → 复制"策略下，若进程在复制完成前崩溃或断电，会留下
/// `{name}.migrating-{时间戳}` 目录而原目录已消失。下次启动必须先把
/// 残留恢复为原目录名再继续迁移，否则旧数据会永久失联。
async fn recover_interrupted_migration(src: &Path) -> Result<(), String> {
    if src.exists() {
        return Ok(());
    }

    let src_name = src.file_name().and_then(|n| n.to_str()).unwrap_or("data");
    let prefix = format!("{src_name}.migrating-");
    // 无父目录（如相对根路径）时无法扫描同级目录，跳过恢复
    let parent = match src.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => return Ok(()),
    };

    let mut entries = tokio::fs::read_dir(&parent)
        .await
        .map_err(|e| format!("无法扫描迁移残留目录 '{}': {}", parent.display(), e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("扫描迁移残留失败: {}", e))?
    {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&prefix) {
            let staged = entry.path();
            tokio::fs::rename(&staged, src)
                .await
                .map_err(|e| format!("无法恢复中断的迁移残留 '{}': {}", staged.display(), e))?;
            observability::config_migration_resumed(&staged, src);
            return Ok(());
        }
    }

    Ok(())
}

/// 递归复制目录。
///
/// 递归的 `async fn` 会产生无限大小的 future，因此用 `Box::pin` 引入间接层。
fn copy_dir_recursive(
    src: PathBuf,
    dst: PathBuf,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MigrationStats, String>> + Send>> {
    Box::pin(async move {
        let mut stats = MigrationStats::default();

        tokio::fs::create_dir_all(&dst)
            .await
            .map_err(|e| format!("无法创建目录 '{}': {}", dst.display(), e))?;

        let mut entries = tokio::fs::read_dir(&src)
            .await
            .map_err(|e| format!("无法读取目录 '{}': {}", src.display(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("读取目录项失败: {}", e))?
        {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            // A7: 用 symlink_metadata 判断条目类型，跳过符号链接，
            // 避免跟随链接递归或把链接指向的目录整体复制进来
            let metadata = match tokio::fs::symlink_metadata(&src_path).await {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("读取条目元数据 '{}' 失败: {}", src_path.display(), e));
                }
            };
            if metadata.file_type().is_symlink() {
                observability::config_migration_symlink_skipped(&src_path);
                continue;
            }

            if metadata.is_dir() {
                let sub = copy_dir_recursive(src_path, dst_path).await?;
                stats.files_copied += sub.files_copied;
                stats.dirs_copied += sub.dirs_copied + 1;
                stats.conflicts += sub.conflicts;
            } else {
                if dst_path.exists() {
                    stats.conflicts += 1;
                    observability::config_migration_conflict(&dst_path);
                }
                tokio::fs::copy(&src_path, &dst_path)
                    .await
                    .map_err(|e| format!("复制文件 '{}' 失败: {}", src_path.display(), e))?;
                stats.files_copied += 1;
            }
        }

        Ok(stats)
    })
}
