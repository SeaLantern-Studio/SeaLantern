use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::limits::{ArchiveSummary, accumulate_bytes};
use super::{ArchiveError, open_existing_directory, parent_path};

/// 创建包含 source 目录内容的 ZIP 归档文件。
///
/// 目标文件必须不存在。临时归档在目标目录中完成，然后通过 rename 原子地移动到
/// 最终位置，因此源文件读取或写入失败不会用部分结果替换原有的归档文件。
///
/// 选择 rename 而非硬链接是因为前者不依赖文件系统的链接能力（FAT32/exFAT 等
/// 不支持硬链接），且成功后无需再清理临时文件。
pub fn create_zip(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ArchiveSummary, ArchiveError> {
    let source = source.as_ref();
    let destination = destination.as_ref();
    let result = create_zip_inner(source, destination);
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

fn create_zip_inner(source: &Path, destination: &Path) -> Result<ArchiveSummary, ArchiveError> {
    reject_existing_destination(destination)?;
    let source_root =
        open_existing_directory(source, "source must be a directory that is not a symbolic link")?;
    let temporary = temporary_path(destination);
    let result = write_archive(&source_root, source, &temporary);
    match result {
        Ok(summary) => {
            if let Err(error) = fs::rename(&temporary, destination) {
                let publish_error =
                    ArchiveError::io("publish completed archive", destination, error);
                remove_temporary_archive(&temporary);
                return Err(publish_error);
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
/// 仅在失败路径上调用：成功时临时文件已被 rename 移走，路径不再存在。
/// 清理失败只记录日志，不覆盖调用方要返回的原始错误。
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
) -> Result<ArchiveSummary, ArchiveError> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(|error| ArchiveError::io("create temporary archive", temporary, error))?;
    let mut writer = ZipWriter::new(file);
    let file_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
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
            let name = portable_name(&relative)?;
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
fn portable_name(path: &Path) -> Result<String, ArchiveError> {
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
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::extract_zip;

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
        // rename 发布后临时文件不应残留，输出目录里只有归档本身。
        let published: Vec<_> = fs::read_dir(root.join("output"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(published, vec![std::ffi::OsString::from("server.zip")]);

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
