//! 备份归档的创建与解压。
//!
//! 归档的读写、路径安全、资源上限等全部由 `sealantern_infra::archive` 承担。
//! 本模块只负责备份业务特有的两件事：把 [`BackupFormat`] 分派到对应的 infra
//! 入口，以及在解压前依据可用内存决定是否放行。

use std::fs;
use std::path::Path;

use sealantern_infra::archive::{
    ArchiveFormat, CompressionLevel as ArchiveCompressionLevel, ExtractionLimits,
    ExtractionSummary, create_tar_gz_with_level, create_zip_with_level, detect_archive_format,
    extract_tar_gz_with_limits, extract_zip_with_limits,
};
use sealantern_infra::platform::collect_resource_snapshot;
use tracing::{debug, warn};

use super::error::{BackupError, BackupResult};
use super::models::{BackupFormat, CompressionLevel};

const MAX_ARCHIVE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
const MAX_ENTRY_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_ENTRY_PATH_BYTES: usize = 4096;
const MAX_COMPRESSION_RATIO: u64 = 200;

const MIN_AVAILABLE_MEMORY_BYTES: u64 = 128 * 1024 * 1024;
const STREAMING_MEMORY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_INDEX_MEMORY_BYTES: u64 = 128 * 1024 * 1024;

const BACKUP_TARGET: &str = "sealantern.feature.backup";
const EVENT_MEMORY_PREFLIGHT: &str = "backup_restore_memory_preflight";

/// 归档解压后的统计信息。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ArchiveStats {
    pub archive_bytes: u64,
    pub entries: usize,
    pub unpacked_bytes: u64,
}

/// 按备份格式创建归档。
///
/// 创建后立即按恢复时的上限校验一遍，避免产出「备份成功但永远恢复不了」的
/// 归档，见 [`verify_restorable`]。
pub(crate) fn create_archive(
    source: &Path,
    destination: &Path,
    format: BackupFormat,
    compression_level: CompressionLevel,
) -> BackupResult<()> {
    let level = archive_compression_level(compression_level);
    let summary = match format {
        BackupFormat::Zip => create_zip_with_level(source, destination, level),
        BackupFormat::TarGz => create_tar_gz_with_level(source, destination, level),
    }
    .map_err(map_create_error)?;

    if let Err(error) = verify_restorable(destination, summary) {
        remove_unrestorable_archive(destination);
        return Err(error);
    }
    Ok(())
}

/// 校验刚创建的归档在恢复时不会撞上资源上限。
///
/// 备份写出后若无法恢复，故障只会在用户真正需要它时才暴露，因此这里提前拦下。
/// 校验只用创建过程已经得到的统计与归档文件大小，不重新读取归档内容——对 ZIP
/// 而言可以省下一次中央目录扫描，对 tar.gz 而言可以省下整整一遍解压。
///
/// 覆盖归档大小、条目数、总解压字节与整体压缩比。压缩比对 tar.gz 与恢复侧的
/// 判定完全一致（gzip 整体压缩）；对 ZIP 是整体近似，因为恢复侧按条目比较，
/// 逐条目的严格判定仍在恢复时执行。这一层的作用是零成本拦下明显不可恢复的
/// 归档，例如大段重复内容压出的极高压缩比。
fn verify_restorable(
    archive_path: &Path,
    summary: sealantern_infra::archive::ArchiveSummary,
) -> BackupResult<()> {
    let archive_bytes = archive_size(archive_path)?;
    let entries = summary.files.saturating_add(summary.directories);
    if entries > MAX_ENTRIES as u64 {
        return Err(unrestorable(archive_path, "条目数量超过恢复限制"));
    }
    if summary.bytes > MAX_TOTAL_BYTES {
        return Err(unrestorable(archive_path, "解压后总大小超过恢复限制"));
    }
    if summary.bytes > archive_bytes.saturating_mul(MAX_COMPRESSION_RATIO) {
        return Err(unrestorable(archive_path, "压缩比超过恢复限制"));
    }
    Ok(())
}

fn unrestorable(archive_path: &Path, reason: &str) -> BackupError {
    BackupError::Validation(format!("归档 {:?} 无法恢复: {reason}", archive_path))
}

/// 删除无法恢复的归档，使失败的备份不留下产物。
fn remove_unrestorable_archive(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        warn!(
            target: BACKUP_TARGET,
            event_name = "backup_archive_cleanup_failed",
            path = %path.display(),
            error = %error,
            "failed to remove an archive that cannot be restored"
        );
    }
}

/// 解压备份归档到一个尚不存在的目标目录。
///
/// `format` 只是元数据中记录的期望格式，实际使用的解析器由文件魔数决定：备份
/// 元数据可能与磁盘内容不符（历史上存在过用 ZIP 内容写出 `.tar.gz` 文件的缺陷），
/// 按内容判定才能正确读出这些归档。
pub(crate) fn extract_archive(
    archive_path: &Path,
    destination: &Path,
    format: BackupFormat,
) -> BackupResult<ArchiveStats> {
    let archive_bytes = archive_size(archive_path)?;
    let actual_format = detect_archive_format(archive_path)?;
    if expected_format(format) != actual_format {
        warn!(
            target: BACKUP_TARGET,
            event_name = "backup_archive_format_mismatch",
            archive = %archive_path.display(),
            recorded_format = %format,
            detected_format = %actual_format,
            "backup metadata format disagrees with archive contents; using detected format"
        );
    }
    check_memory_budget(archive_path, actual_format, archive_bytes)?;

    let limits = extraction_limits();
    let summary = match actual_format {
        ArchiveFormat::Zip => extract_zip_with_limits(archive_path, destination, limits)?,
        ArchiveFormat::TarGz => extract_tar_gz_with_limits(archive_path, destination, limits)?,
    };
    let stats = archive_stats(archive_bytes, summary);

    debug!(
        target: BACKUP_TARGET,
        event_name = "backup_archive_extracted",
        archive = %archive_path.display(),
        archive_bytes = stats.archive_bytes,
        entries = stats.entries,
        unpacked_bytes = stats.unpacked_bytes,
        "backup archive extracted"
    );
    Ok(stats)
}

/// 在解压之前检查可用内存是否足以容纳解析开销。
///
/// ZIP 的中央目录可以在解压前廉价读出，因此按实际条目数估算；tar.gz 没有
/// 中央目录，取得条目数需要完整解压一遍 gzip 流，代价是解压两次。因此 tar.gz
/// 不做条目数预检，仅保留固定门槛，条目路径集合的累积交由 infra 层已有的
/// max_entries / max_entry_path_bytes 流式限制在解压过程中约束。
fn check_memory_budget(
    archive_path: &Path,
    format: ArchiveFormat,
    archive_bytes: u64,
) -> BackupResult<()> {
    let entry_count = match format {
        // 预检时顺便读中央目录，与随后解压时的读取重复但成本低廉（只读目录，
        // 不解压条目），换取真实条目数。
        ArchiveFormat::Zip => count_zip_entries(archive_path)?,
        ArchiveFormat::TarGz => 0,
    };
    let required_memory = required_memory(archive_bytes, entry_count, format);
    let available_memory = collect_resource_snapshot().available_memory_bytes;

    if available_memory < required_memory {
        warn!(
            target: BACKUP_TARGET,
            event_name = EVENT_MEMORY_PREFLIGHT,
            status = "rejected",
            format = %format,
            archive = %archive_path.display(),
            archive_bytes = archive_bytes,
            entries = entry_count,
            available_memory_bytes = available_memory,
            required_memory_bytes = required_memory,
            "backup restore rejected before unpack because available memory is too low"
        );
        return Err(BackupError::InsufficientMemory {
            available: available_memory,
            required: required_memory,
        });
    }

    debug!(
        target: BACKUP_TARGET,
        event_name = EVENT_MEMORY_PREFLIGHT,
        status = "passed",
        format = %format,
        archive = %archive_path.display(),
        archive_bytes = archive_bytes,
        entries = entry_count,
        available_memory_bytes = available_memory,
        required_memory_bytes = required_memory,
        "backup restore memory preflight passed"
    );
    Ok(())
}

/// 读取 ZIP 中央目录得到条目数。
///
/// 只解析目录结构，不读取任何条目内容；中央目录通常在归档末尾，读取量与条目
/// 数成正比。条目数超限时这里不拒绝——统一的资源上限由 infra 的
/// [`ExtractionLimits`] 在解压时施加，这里只关心内存估算。
fn count_zip_entries(path: &Path) -> BackupResult<usize> {
    sealantern_infra::archive::zip_entry_count(path).map_err(BackupError::Archive)
}

/// 估算解压一个归档所需的常驻内存。
///
/// 各项对应解压过程中真实存在的持有量：条目路径去重集合按实际条目数计、ZIP 的
/// 中央目录需要先整体读入、流式缓冲固定开销。系数为经验值，用于把明显不可行的
/// 恢复挡在解压之前，而非精确预测。
///
/// tar.gz（`entry_count` 为 0）只保留固定门槛：条目路径集合的增长由 infra 的
/// 流式限制约束，不由预检预估。
fn required_memory(archive_bytes: u64, entry_count: usize, format: ArchiveFormat) -> u64 {
    let index_memory = (entry_count as u64)
        .saturating_mul(8 * 1024)
        .min(MAX_INDEX_MEMORY_BYTES);
    let path_memory = (entry_count as u64)
        .saturating_mul(MAX_ENTRY_PATH_BYTES as u64)
        .min(64 * 1024 * 1024);
    let archive_overhead = match format {
        // ZIP 解析前需要读入中央目录，其规模随归档增长。
        ArchiveFormat::Zip => (archive_bytes / 4).min(512 * 1024 * 1024),
        // tar.gz 全程流式，除固定窗口外不随归档大小增长。
        ArchiveFormat::TarGz => 0,
    };
    MIN_AVAILABLE_MEMORY_BYTES
        .saturating_add(STREAMING_MEMORY_BYTES)
        .saturating_add(index_memory)
        .saturating_add(path_memory)
        .saturating_add(archive_overhead)
}

/// 备份解压统一使用的资源上限。
fn extraction_limits() -> ExtractionLimits {
    ExtractionLimits {
        max_archive_bytes: MAX_ARCHIVE_BYTES,
        max_entries: MAX_ENTRIES,
        max_entry_bytes: MAX_ENTRY_BYTES,
        max_total_bytes: MAX_TOTAL_BYTES,
        max_entry_path_bytes: MAX_ENTRY_PATH_BYTES,
        max_compression_ratio: MAX_COMPRESSION_RATIO,
    }
}

fn archive_stats(archive_bytes: u64, summary: ExtractionSummary) -> ArchiveStats {
    ArchiveStats {
        archive_bytes,
        entries: summary.files.saturating_add(summary.directories) as usize,
        unpacked_bytes: summary.bytes,
    }
}

fn expected_format(format: BackupFormat) -> ArchiveFormat {
    match format {
        BackupFormat::Zip => ArchiveFormat::Zip,
        BackupFormat::TarGz => ArchiveFormat::TarGz,
    }
}

fn archive_compression_level(level: CompressionLevel) -> ArchiveCompressionLevel {
    match level {
        CompressionLevel::Low => ArchiveCompressionLevel::Low,
        CompressionLevel::Medium => ArchiveCompressionLevel::Medium,
        CompressionLevel::High => ArchiveCompressionLevel::High,
    }
}

/// 校验归档是一个普通文件并返回其大小。
///
/// 文件缺失、是符号链接或不是普通文件都归为备份损坏：备份文件由本模块写出，
/// 出现这些情况说明存储被外部改动过。
fn archive_size(path: &Path) -> BackupResult<u64> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(BackupError::CorruptedBackup(path.to_path_buf()));
        }
        Err(error) => return Err(BackupError::Io(error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(BackupError::CorruptedBackup(path.to_path_buf()));
    }
    if metadata.len() > MAX_ARCHIVE_BYTES {
        return Err(BackupError::Validation(format!("归档 {:?} 无效: 归档大小超过限制", path)));
    }
    Ok(metadata.len())
}

/// 把归档创建失败中的「目标已存在」映射到备份自己的错误变体。
///
/// 其余错误按 `#[from]` 转为 [`BackupError::Archive`]。
fn map_create_error(error: sealantern_infra::archive::ArchiveError) -> BackupError {
    match error {
        sealantern_infra::archive::ArchiveError::DestinationExists { path } => {
            BackupError::AlreadyExists(path)
        }
        error => BackupError::Archive(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_budget_reserves_parser_space_before_unpack() {
        let small = required_memory(1, 0, ArchiveFormat::TarGz);
        let large = required_memory(MAX_ARCHIVE_BYTES, MAX_ENTRIES, ArchiveFormat::Zip);

        assert!(small >= MIN_AVAILABLE_MEMORY_BYTES);
        // ZIP 需要额外容纳中央目录，因此同等条件下要求更高。
        assert!(large > small);
    }

    #[test]
    fn tar_gz_budget_does_not_grow_with_archive_size() {
        // tar.gz 全程流式，内存需求不应随归档增大。
        assert_eq!(
            required_memory(1, 0, ArchiveFormat::TarGz),
            required_memory(MAX_ARCHIVE_BYTES, 0, ArchiveFormat::TarGz)
        );
    }

    #[test]
    fn small_backup_requires_far_less_memory_than_a_large_one() {
        // 1 KiB 的小备份按实际条目数估算，显著低于按配置上限取悲观值的估算。
        let small = required_memory(1024, 2, ArchiveFormat::TarGz);
        let worst = required_memory(MAX_ARCHIVE_BYTES, MAX_ENTRIES, ArchiveFormat::Zip);

        assert!(small < worst);
        // 小备份的增量只有固定门槛，不随条目数膨胀。
        assert!(small < 2 * MIN_AVAILABLE_MEMORY_BYTES);
    }

    #[test]
    fn zip_entry_count_reads_the_central_directory() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("server.properties"), b"motd=Sea Lantern").unwrap();
        let archive = root.path().join("count.zip");
        sealantern_infra::archive::create_zip(&source, &archive).unwrap();

        assert_eq!(count_zip_entries(&archive).unwrap(), 1);
    }

    #[test]
    fn stats_count_files_and_directories_as_entries() {
        let summary = ExtractionSummary { files: 3, directories: 2, bytes: 128 };
        let stats = archive_stats(1024, summary);

        assert_eq!(stats.archive_bytes, 1024);
        assert_eq!(stats.entries, 5);
        assert_eq!(stats.unpacked_bytes, 128);
    }
}
