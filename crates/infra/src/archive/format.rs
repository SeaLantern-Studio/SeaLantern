//! 归档格式识别与压缩级别。
//!
//! 归档格式由文件内容的魔数判定，而非扩展名：扩展名可能缺失、错误，或因历史
//! 缺陷与实际内容不符（例如曾经用 ZIP 内容写出 `.tar.gz` 文件）。

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::ArchiveError;

/// ZIP 本地文件头魔数，出现在普通非空归档的开头。
const ZIP_LOCAL_HEADER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

/// ZIP 中央目录结束记录魔数，空归档的开头即为此。
const ZIP_END_OF_CENTRAL_DIRECTORY: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];

/// ZIP 跨卷归档标记魔数。
const ZIP_SPANNED: [u8; 4] = [0x50, 0x4B, 0x07, 0x08];

/// gzip 成员头魔数。
const GZIP_MEMBER: [u8; 2] = [0x1F, 0x8B];

/// 本模块支持的归档格式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveFormat {
    /// ZIP 归档，由 [`super::extract_zip`] 与 [`super::create_zip`] 处理。
    Zip,
    /// gzip 压缩的 tar 归档，由 [`super::extract_tar_gz`] 与
    /// [`super::create_tar_gz`] 处理。
    TarGz,
}

impl ArchiveFormat {
    /// 该格式惯用的文件扩展名。
    pub fn extension(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::TarGz => "tar.gz",
        }
    }
}

impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.extension())
    }
}

/// 按文件内容的魔数判定归档格式。
///
/// 不参考扩展名：识别结果必须反映实际内容，否则调用方会用错误的解析器读取文件。
/// 无法识别时返回 [`ArchiveError::InvalidSource`]，调用方据此拒绝而不是猜测。
///
/// ZIP 认三种魔数：普通归档的本地文件头、空归档直接以中央目录结束记录开头、
/// 跨卷归档标记。gzip 只有单一的成员头魔数。
pub fn detect_archive_format(path: impl AsRef<Path>) -> Result<ArchiveFormat, ArchiveError> {
    let path = path.as_ref();
    let mut file =
        File::open(path).map_err(|error| ArchiveError::io("open archive", path, error))?;
    let mut magic = [0_u8; 4];
    let mut filled = 0;
    // 单次 read 不保证填满缓冲，循环直到读满或到达文件末尾。
    while filled < magic.len() {
        let count = file
            .read(&mut magic[filled..])
            .map_err(|error| ArchiveError::io("read archive magic bytes", path, error))?;
        if count == 0 {
            break;
        }
        filled += count;
    }
    detect_format_from_magic(path, &magic[..filled])
}

/// 从已读取的前导字节判定格式。
fn detect_format_from_magic(path: &Path, magic: &[u8]) -> Result<ArchiveFormat, ArchiveError> {
    if magic.starts_with(&GZIP_MEMBER) {
        return Ok(ArchiveFormat::TarGz);
    }
    if magic.starts_with(&ZIP_LOCAL_HEADER)
        || magic.starts_with(&ZIP_END_OF_CENTRAL_DIRECTORY)
        || magic.starts_with(&ZIP_SPANNED)
    {
        return Ok(ArchiveFormat::Zip);
    }
    Err(ArchiveError::InvalidSource {
        path: path.to_path_buf(),
        reason: "file is not a recognized ZIP or tar.gz archive",
    })
}

/// 归档创建时使用的压缩强度。
///
/// 三档而非直接暴露底层数值，因为 ZIP 的 deflate 与 gzip 各有自己的取值范围，
/// 由本枚举统一映射。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CompressionLevel {
    /// 最快，压缩率最低。
    Low,
    /// 速度与压缩率的折中。
    #[default]
    Medium,
    /// 最高压缩率，最慢。
    High,
}

impl CompressionLevel {
    /// 对应的 deflate 级别，ZIP 与 gzip 通用的 1-9 取值。
    pub(super) fn deflate_level(self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 6,
            Self::High => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_zip_magic_variants() {
        let path = Path::new("archive.bin");
        for magic in [ZIP_LOCAL_HEADER, ZIP_END_OF_CENTRAL_DIRECTORY, ZIP_SPANNED] {
            assert_eq!(detect_format_from_magic(path, &magic).unwrap(), ArchiveFormat::Zip);
        }
    }

    #[test]
    fn recognizes_gzip_magic() {
        // gzip 头只有 2 字节魔数，后续字节是压缩方法与标志位，不参与判定。
        let magic = [0x1F, 0x8B, 0x08, 0x00];
        assert_eq!(
            detect_format_from_magic(Path::new("archive.bin"), &magic).unwrap(),
            ArchiveFormat::TarGz
        );
    }

    #[test]
    fn rejects_unrecognized_and_truncated_input() {
        let path = Path::new("archive.bin");
        for magic in [&b""[..], &b"\x1f"[..], &b"PK"[..], &b"not an archive"[..]] {
            assert!(matches!(
                detect_format_from_magic(path, magic),
                Err(ArchiveError::InvalidSource { .. })
            ));
        }
    }

    #[test]
    fn detects_format_from_files_regardless_of_extension() {
        let root = crate::fs::test_dir("format-detection");
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("server.properties"), b"motd=Sea Lantern").unwrap();

        // 扩展名与内容故意互换，判定必须依据内容。
        let zip_named_tar = root.join("actually-zip.tar.gz");
        let tar_named_zip = root.join("actually-tar.zip");
        super::super::create_zip(&source, &zip_named_tar).unwrap();
        super::super::create_tar_gz(&source, &tar_named_zip).unwrap();

        assert_eq!(detect_archive_format(&zip_named_tar).unwrap(), ArchiveFormat::Zip);
        assert_eq!(detect_archive_format(&tar_named_zip).unwrap(), ArchiveFormat::TarGz);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn maps_compression_levels_to_deflate_values() {
        assert_eq!(CompressionLevel::Low.deflate_level(), 1);
        assert_eq!(CompressionLevel::default().deflate_level(), 6);
        assert_eq!(CompressionLevel::High.deflate_level(), 9);
    }
}
