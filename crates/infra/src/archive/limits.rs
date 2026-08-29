//! 归档解压与创建过程中的资源限制与统计。
//!
//! 限制与统计本身与具体归档格式无关：ZIP 与 tar.gz 共用同一套上限定义
//! 和字节计数语义，仅在「何时能够校验」上有所差异。

use std::path::Path;

use super::ArchiveError;

/// 归档解压前后应用的资源限制。
///
/// ZIP 拥有中央目录，可在写入任何文件之前完成全量预检；tar.gz 是不可
/// 回退的流，只能在流式写入过程中持续累加校验。两者共用本结构，语义
/// 差异见各字段说明。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtractionLimits {
    /// 磁盘上可接受的压缩归档文件最大大小。
    pub max_archive_bytes: u64,
    /// 归档中的最大条目数。
    pub max_entries: usize,
    /// 单个常规文件的最大未压缩字节数。
    pub max_entry_bytes: u64,
    /// 所有条目写入的最大未压缩字节总数。
    pub max_total_bytes: u64,
    /// 可接受的未压缩与压缩字节的最大比率。
    ///
    /// ZIP 按条目比较（每个条目独立压缩，有各自的压缩后大小）；
    /// tar.gz 为整体流压缩，单条目无压缩后大小，只能比较累计解压
    /// 字节与归档文件总字节的比率，且该比较在解压流上持续进行。
    pub max_compression_ratio: u64,
}

impl Default for ExtractionLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 4 * 1024 * 1024 * 1024,
            max_entries: 10_000,
            max_entry_bytes: 4 * 1024 * 1024 * 1024,
            max_total_bytes: 16 * 1024 * 1024 * 1024,
            max_compression_ratio: 200,
        }
    }
}

/// 统计归档解压过程中处理的条目数和未压缩字节数。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExtractionSummary {
    /// 解压的常规文件数。
    pub files: u64,
    /// 解压的目录条目数。
    pub directories: u64,
    /// 解压的文件未压缩字节总数。
    pub bytes: u64,
}

/// 统计归档创建过程中处理的条目数和非压缩字节数。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArchiveSummary {
    /// 已写入的普通文件数。
    pub files: u64,
    /// 已写入的目录条目数。
    pub directories: u64,
    /// 已写入的文件总非压缩字节数。
    pub bytes: u64,
}

/// 校验观测值未超过上限，超限时返回 [`ArchiveError::LimitExceeded`]。
pub(super) fn check_limit(
    archive: &Path,
    limit: &'static str,
    observed: u64,
    maximum: u64,
) -> Result<(), ArchiveError> {
    if observed > maximum {
        return Err(ArchiveError::LimitExceeded {
            archive: archive.to_path_buf(),
            limit,
            observed,
            maximum,
        });
    }
    Ok(())
}

/// 累加字节计数，溢出时按超限处理。
///
/// 归档声明的条目大小不可信，累加可能溢出 `u64`。溢出等价于超出任何
/// 可配置上限，因此直接以 `u64::MAX` 作为观测值返回超限错误。
pub(super) fn accumulate_bytes(
    total: u64,
    additional: u64,
    archive: &Path,
    limit: &'static str,
    maximum: u64,
) -> Result<u64, ArchiveError> {
    total
        .checked_add(additional)
        .ok_or_else(|| ArchiveError::LimitExceeded {
            archive: archive.to_path_buf(),
            limit,
            observed: u64::MAX,
            maximum,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_observed_at_the_limit() {
        assert!(check_limit(Path::new("a.zip"), "bytes", 16, 16).is_ok());
    }

    #[test]
    fn rejects_observed_above_the_limit() {
        assert!(matches!(
            check_limit(Path::new("a.zip"), "bytes", 17, 16),
            Err(ArchiveError::LimitExceeded { observed: 17, maximum: 16, .. })
        ));
    }

    #[test]
    fn reports_overflow_as_limit_exceeded() {
        assert!(matches!(
            accumulate_bytes(u64::MAX, 1, Path::new("a.zip"), "bytes", 16),
            Err(ArchiveError::LimitExceeded { observed: u64::MAX, .. })
        ));
    }
}
