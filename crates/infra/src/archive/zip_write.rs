use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::limits::{ArchiveSummary, accumulate_bytes};
use super::{ArchiveError, CompressionLevel, open_existing_directory, parent_path, publish_new};

/// 使用默认压缩级别创建包含 source 目录内容的 ZIP 归档文件。
///
/// 语义与 [`create_zip_with_level`] 相同，压缩级别取
/// [`CompressionLevel::Medium`]。
pub fn create_zip(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ArchiveSummary, ArchiveError> {
    create_zip_with_level(source, destination, CompressionLevel::default())
}

/// 使用显式压缩级别创建包含 source 目录内容的 ZIP 归档文件。
///
/// 目标文件必须不存在。临时归档在目标同一目录中完成，然后经 [`publish_new`]
/// 以 create-new 语义移动到最终位置，因此源文件读取或写入失败不会用部分结果
/// 替换原有的归档文件，目标在写入期间才出现也不会被覆盖。
pub fn create_zip_with_level(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    level: CompressionLevel,
) -> Result<ArchiveSummary, ArchiveError> {
    let source = source.as_ref();
    let destination = destination.as_ref();
    let result = create_zip_inner(source, destination, level);
    if let Err(error) = &result {
        crate::observability::archive_operation_failed_with_context(
            "create ZIP",
            destination,
            Some(source),
            error.entry(),
            error,
        );
    }
    result
}

fn create_zip_inner(
    source: &Path,
    destination: &Path,
    level: CompressionLevel,
) -> Result<ArchiveSummary, ArchiveError> {
    reject_existing_destination(destination)?;
    let source_root =
        open_existing_directory(source, "source must be a directory that is not a symbolic link")?;
    let temporary = temporary_path(destination);
    let result = write_archive(&source_root, source, &temporary, level);
    match result {
        Ok(summary) => {
            if let Err(error) = publish_new(&temporary, destination) {
                remove_temporary_archive(&temporary);
                return Err(error);
            }
            Ok(summary)
        }
        Err(error) => {
            remove_temporary_archive(&temporary);
            Err(error)
        }
    }
}

/// 清理未能发布的临时归档。
///
/// 仅在失败路径上调用：成功时临时文件已被 [`super::publish_new`] 移走，路径
/// 不再存在。清理失败只记录日志，不覆盖调用方要返回的原始错误。
fn remove_temporary_archive(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != io::ErrorKind::NotFound
    {
        crate::observability::archive_cleanup_failed(path, &error);
    }
}

/// 拒绝已存在的目标路径，并确保其父目录存在。
///
/// 归档创建从不覆盖既有文件：目标存在即返回 [`ArchiveError::DestinationExists`]，
/// 避免用部分写入的结果替换原有归档。
fn reject_existing_destination(destination: &Path) -> Result<(), ArchiveError> {
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            return Err(ArchiveError::DestinationExists { path: destination.to_path_buf() });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ArchiveError::io("read archive destination metadata", destination, error));
        }
    }
    let parent = parent_path(destination);
    fs::create_dir_all(parent)
        .map_err(|error| ArchiveError::io("create archive destination parent", parent, error))?;
    Ok(())
}

/// 广度遍历源目录并写入 ZIP 条目。
///
/// 子目录按名称排序后入栈，保证同一份源目录产出稳定的条目顺序。
/// 符号链接与设备等特殊条目一律拒绝，不做跟随。
fn write_archive(
    source_root: &Dir,
    source_path: &Path,
    temporary: &Path,
    level: CompressionLevel,
) -> Result<ArchiveSummary, ArchiveError> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(|error| ArchiveError::io("create temporary archive", temporary, error))?;
    let mut writer = ZipWriter::new(file);
    let file_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(level.deflate_level() as i64))
        .large_file(true);
    let directory_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);
    let mut summary = ArchiveSummary::default();
    let mut directories = vec![PathBuf::new()];

    while let Some(directory) = directories.pop() {
        let mut children = source_root
            .read_dir(if directory.as_os_str().is_empty() {
                Path::new(".")
            } else {
                &directory
            })
            .map_err(|error| {
                ArchiveError::io(
                    "read archive source directory",
                    source_path.join(&directory),
                    error,
                )
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                ArchiveError::io(
                    "iterate archive source directory",
                    source_path.join(&directory),
                    error,
                )
            })?;
        children.sort_by_key(|entry| entry.file_name());
        let mut child_directories = Vec::new();

        for child in children {
            let relative = directory.join(child.file_name());
            let display_path = source_path.join(&relative);
            let metadata = source_root.symlink_metadata(&relative).map_err(|error| {
                ArchiveError::io("read archive source entry metadata", &display_path, error)
            })?;
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                return Err(ArchiveError::UnsupportedSourceEntry {
                    path: display_path,
                    kind: "symbolic link",
                });
            }
            let name = portable_name(&relative, &display_path)?;
            if file_type.is_dir() {
                writer
                    .add_directory(name, directory_options)
                    .map_err(|error| {
                        ArchiveError::zip("write directory entry to", temporary, error)
                    })?;
                summary.directories += 1;
                child_directories.push(relative);
                continue;
            }
            if !file_type.is_file() {
                return Err(ArchiveError::UnsupportedSourceEntry {
                    path: display_path,
                    kind: "special",
                });
            }

            writer
                .start_file(name, file_options)
                .map_err(|error| ArchiveError::zip("start file entry in", temporary, error))?;
            let mut input = source_root.open(&relative).map_err(|error| {
                ArchiveError::io("open archive source file", &display_path, error)
            })?;
            let copied = io::copy(&mut input, &mut writer).map_err(|error| {
                ArchiveError::io("write archive file entry", &display_path, error)
            })?;
            summary.files += 1;
            summary.bytes = accumulate_bytes(
                summary.bytes,
                copied,
                temporary,
                "archive source bytes",
                u64::MAX - 1,
            )?;
        }
        directories.extend(child_directories.into_iter().rev());
    }

    writer
        .finish()
        .map_err(|error| ArchiveError::zip("finalize", temporary, error))?;
    Ok(summary)
}

fn temporary_path(destination: &Path) -> PathBuf {
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("archive.zip");
    parent_path(destination).join(format!(".{filename}.{}.tmp", uuid::Uuid::new_v4()))
}

/// 将相对路径转换为归档内使用的正斜杠条目名。
///
/// 转换结果与解压侧共用同一套路径可移植性判定（[`crate::fs::SafeRelativePath`]）：
/// 写入侧在产出归档时就拒绝 Windows 保留名、尾随点/空格等不可移植名称，而不是
/// 等到恢复时才由 `safe_entry_path` 拒绝——否则会产出「备份成功但永远恢复不了」
/// 的归档。`display_path` 仅用于错误消息展示源文件位置。
fn portable_name(path: &Path, display_path: &Path) -> Result<String, ArchiveError> {
    let mut name = String::new();
    for component in path.components() {
        let component =
            component
                .as_os_str()
                .to_str()
                .ok_or_else(|| ArchiveError::InvalidSource {
                    path: path.to_path_buf(),
                    reason: "path contains non-Unicode components",
                })?;
        if !name.is_empty() {
            name.push('/');
        }
        name.push_str(component);
    }
    crate::fs::SafeRelativePath::parse(&name).map_err(|_| {
        ArchiveError::UnsupportedSourceEntry {
            path: display_path.to_path_buf(),
            kind: "non-portable name",
        }
    })?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::extract_zip;

    /// 源目录内出现 Windows 保留名等不可移植名称时，创建应失败而非产出
    /// 「备份成功但永远恢复不了」的归档。仅在 Unix 上测试：Windows 无法创建
    /// 名为 `NUL` 的文件（保留设备名）。
    #[cfg(unix)]
    #[test]
    fn rejects_non_portable_source_entry_names() {
        let root = crate::fs::test_dir("zip-non-portable");
        let source = root.join("source");
        let archive = root.join("output").join("server.zip");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("NUL"), b"reserved name").unwrap();
        fs::write(source.join("normal.properties"), b"motd=Sea Lantern").unwrap();

        assert!(matches!(
            create_zip(&source, &archive),
            Err(ArchiveError::UnsupportedSourceEntry { .. })
        ));
        assert!(!archive.exists());

        fs::remove_dir_all(root).unwrap();
    }

    /// 带尾随点的名称同样不可移植，写入侧应拒绝。
    #[cfg(unix)]
    #[test]
    fn rejects_trailing_dot_source_entry_names() {
        let root = crate::fs::test_dir("zip-trailing-dot");
        let source = root.join("source");
        let archive = root.join("output").join("server.zip");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("evil."), b"trailing dot").unwrap();

        assert!(matches!(
            create_zip(&source, &archive),
            Err(ArchiveError::UnsupportedSourceEntry { .. })
        ));
        assert!(!archive.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archives_and_extracts_directory_contents() {
        let root = crate::fs::test_dir("zipper");
        let source = root.join("source");
        let archive = root.join("output").join("server.zip");
        let extracted = root.join("extracted");
        fs::create_dir_all(source.join("nested/empty")).unwrap();
        fs::write(source.join("nested/server.properties"), b"motd=Sea Lantern").unwrap();

        let created = create_zip(&source, &archive).unwrap();
        let extracted_summary = extract_zip(&archive, &extracted).unwrap();

        assert_eq!(created.files, 1);
        assert_eq!(created.directories, 2);
        assert_eq!(created.bytes, 16);
        assert_eq!(created.files, extracted_summary.files);
        assert_eq!(created.directories, extracted_summary.directories);
        assert_eq!(
            fs::read(extracted.join("nested/server.properties")).unwrap(),
            b"motd=Sea Lantern"
        );
        assert!(extracted.join("nested/empty").is_dir());
        // 发布后临时文件不应残留，输出目录里只有归档本身。
        let published: Vec<_> = fs::read_dir(root.join("output"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(published, vec![std::ffi::OsString::from("server.zip")]);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn honours_the_requested_compression_level() {
        let root = crate::fs::test_dir("zip-level");
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        // 高度可压缩且足够大的负载，使级别差异体现在输出体积上。
        fs::write(source.join("payload.bin"), "sea lantern ".repeat(8192)).unwrap();

        let low = root.join("low.zip");
        let high = root.join("high.zip");
        create_zip_with_level(&source, &low, CompressionLevel::Low).unwrap();
        create_zip_with_level(&source, &high, CompressionLevel::High).unwrap();

        assert!(
            fs::metadata(&high).unwrap().len() < fs::metadata(&low).unwrap().len(),
            "high compression should produce a smaller archive"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_replace_existing_archive() {
        let root = crate::fs::test_dir("existing-output");
        let source = root.join("source");
        let archive = root.join("server.zip");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("server.properties"), b"motd=Sea Lantern").unwrap();
        fs::write(&archive, b"existing archive").unwrap();

        assert!(matches!(
            create_zip(&source, &archive),
            Err(ArchiveError::DestinationExists { .. })
        ));
        assert_eq!(fs::read(&archive).unwrap(), b"existing archive");

        fs::remove_dir_all(root).unwrap();
    }
}
