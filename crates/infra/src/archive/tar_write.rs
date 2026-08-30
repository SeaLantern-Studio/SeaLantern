use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::{Builder, EntryType, Header};

use super::limits::{ArchiveSummary, accumulate_bytes};
use super::{ArchiveError, CompressionLevel, open_existing_directory, parent_path, publish_new};

/// 归档内普通文件条目的固定权限位。
///
/// 备份与分发场景不保留源文件的实际权限：Windows 上没有对应概念，而在类 Unix
/// 平台上原样保留会把本机的 umask 与所有者信息带进归档，使同一份内容在不同机器
/// 上产出不同的字节。固定为常见的 0644/0755 让归档可复现。
const FILE_MODE: u32 = 0o644;

/// 归档内目录条目的固定权限位，理由同 [`FILE_MODE`]。
const DIRECTORY_MODE: u32 = 0o755;

/// 使用默认压缩级别创建包含 source 目录内容的 tar.gz 归档文件。
///
/// 语义与 [`create_tar_gz_with_level`] 相同，压缩级别取
/// [`CompressionLevel::Medium`]。
pub fn create_tar_gz(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ArchiveSummary, ArchiveError> {
    create_tar_gz_with_level(source, destination, CompressionLevel::default())
}

/// 使用显式压缩级别创建包含 source 目录内容的 tar.gz 归档文件。
///
/// 目标文件必须不存在。临时归档在目标同一父目录中完成，然后经 [`publish_new`]
/// 以 create-new 语义移动到最终位置，因此源文件读取或写入失败不会用部分结果
/// 替换原有的归档文件，目标在写入期间才出现也不会被覆盖。
///
/// 条目元数据被规范化：权限固定为 [`FILE_MODE`] / [`DIRECTORY_MODE`]，mtime 与
/// uid/gid 归零，子目录按名称排序遍历。同一份源目录内容与同一压缩级别因此产出
/// 逐字节一致的归档，便于校验与去重。
///
/// 符号链接、设备节点等特殊条目一律拒绝而非跟随，与 [`super::extract_tar_gz`]
/// 的接受范围对称。
pub fn create_tar_gz_with_level(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    level: CompressionLevel,
) -> Result<ArchiveSummary, ArchiveError> {
    let source = source.as_ref();
    let destination = destination.as_ref();
    let result = create_tar_gz_inner(source, destination, level);
    if let Err(error) = &result {
        crate::observability::archive_operation_failed_with_context(
            "create tar.gz",
            destination,
            Some(source),
            error.entry(),
            error,
        );
    }
    result
}

fn create_tar_gz_inner(
    source: &Path,
    destination: &Path,
    level: CompressionLevel,
) -> Result<ArchiveSummary, ArchiveError> {
    reject_existing_destination(destination)?;
    let source_root =
        open_existing_directory(source, "source must be a directory that is not a symbolic link")?;
    let temporary = temporary_path(destination);
    match write_archive(&source_root, source, &temporary, level) {
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

/// 广度遍历源目录并写入 tar 条目，整体经 gzip 压缩。
///
/// 子目录按名称排序后入栈，保证同一份源目录产出稳定的条目顺序。
/// 所有元数据读取与文件打开都通过 `source_root` 目录句柄完成，校验与使用作用于
/// 同一句柄，不存在中途被符号链接替换的窗口。
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
    let encoder = GzEncoder::new(file, Compression::new(level.deflate_level()));
    let mut builder = Builder::new(encoder);
    let mut summary = ArchiveSummary::default();
    let mut directories = vec![PathBuf::new()];

    while let Some(directory) = directories.pop() {
        let children = read_sorted_children(source_root, source_path, &directory)?;
        let mut child_directories = Vec::new();

        for child in children {
            let relative = directory.join(&child);
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
                // 目录条目名按惯例带尾部斜杠，便于其他工具区分空目录。
                append_directory(&mut builder, &format!("{name}/"), temporary)?;
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

            let declared = metadata.len();
            let input = source_root.open(&relative).map_err(|error| {
                ArchiveError::io("open archive source file", &display_path, error)
            })?;
            append_file(&mut builder, &name, declared, input, temporary, &display_path)?;
            summary.files += 1;
            summary.bytes = accumulate_bytes(
                summary.bytes,
                declared,
                temporary,
                "archive source bytes",
                u64::MAX - 1,
            )?;
        }
        directories.extend(child_directories.into_iter().rev());
    }

    let encoder = builder
        .into_inner()
        .map_err(|error| ArchiveError::tar("finalize tar stream of", temporary, error))?;
    encoder
        .finish()
        .map_err(|error| ArchiveError::tar("finalize gzip stream of", temporary, error))?;
    Ok(summary)
}

/// 读取目录下的子项名并按字节序排序。
///
/// 排序保证同一份源目录的遍历顺序稳定，而 `read_dir` 的返回顺序依赖文件系统。
fn read_sorted_children(
    source_root: &Dir,
    source_path: &Path,
    directory: &Path,
) -> Result<Vec<std::ffi::OsString>, ArchiveError> {
    let target = if directory.as_os_str().is_empty() {
        Path::new(".")
    } else {
        directory
    };
    let entries = source_root
        .read_dir(target)
        .map_err(|error| {
            ArchiveError::io("read archive source directory", source_path.join(directory), error)
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            ArchiveError::io("iterate archive source directory", source_path.join(directory), error)
        })?;
    let mut names: Vec<_> = entries.into_iter().map(|entry| entry.file_name()).collect();
    names.sort();
    Ok(names)
}

/// 追加一个目录条目。
fn append_directory<W: io::Write>(
    builder: &mut Builder<W>,
    name: &str,
    temporary: &Path,
) -> Result<(), ArchiveError> {
    let mut header = normalized_header(EntryType::Directory, DIRECTORY_MODE, 0);
    builder
        .append_data(&mut header, name, io::empty())
        .map_err(|error| ArchiveError::tar("write directory entry to", temporary, error))
}

/// 追加一个普通文件条目。
///
/// tar 先写 header 声明大小、再写内容，两者必须一致，否则产出的归档结构错乱且
/// tar crate 不会察觉。备份运行中的目录时源文件随时可能变长或变短，因此内容经
/// [`ExactSizeReader`] 约束为恰好 `declared` 字节：变长部分被截断，变短则报错。
fn append_file<W: io::Write, R: Read>(
    builder: &mut Builder<W>,
    name: &str,
    declared: u64,
    input: R,
    temporary: &Path,
    display_path: &Path,
) -> Result<(), ArchiveError> {
    let mut header = normalized_header(EntryType::Regular, FILE_MODE, declared);
    let mut reader = ExactSizeReader {
        inner: input.take(declared),
        remaining: declared,
    };
    builder
        .append_data(&mut header, name, &mut reader)
        .map_err(|error| ArchiveError::tar("write file entry to", temporary, error))?;
    if reader.remaining > 0 {
        return Err(ArchiveError::InvalidSource {
            path: display_path.to_path_buf(),
            reason: "source file shrank while being archived",
        });
    }
    Ok(())
}

/// 构造元数据已规范化的 GNU header。
///
/// 显式归零 uid/gid/mtime 而不依赖 `Header::new_gnu` 的零初始化，使「归档可
/// 复现」这一约定在代码中可见。条目名由 `append_data` 填写，超过 ustar 字段长度
/// 时由 tar 自动追加 GNU longname 扩展头。
fn normalized_header(entry_type: EntryType, mode: u32, size: u64) -> Header {
    let mut header = Header::new_gnu();
    header.set_entry_type(entry_type);
    header.set_mode(mode);
    header.set_size(size);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header
}

/// 恰好产出预期字节数的读取器。
///
/// `take` 负责上限，`remaining` 负责下限：读到 EOF 后若 `remaining` 非零，说明
/// 源文件比 header 声明的更短。
struct ExactSizeReader<R> {
    inner: io::Take<R>,
    remaining: u64,
}

impl<R: Read> Read for ExactSizeReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.inner.read(buffer)?;
        self.remaining = self.remaining.saturating_sub(count as u64);
        Ok(count)
    }
}

fn temporary_path(destination: &Path) -> PathBuf {
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("archive.tar.gz");
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
    use crate::archive::extract_tar_gz;

    #[test]
    fn archives_and_extracts_directory_contents() {
        let root = crate::fs::test_dir("tar-write");
        let source = root.join("source");
        let archive = root.join("output").join("server.tar.gz");
        let extracted = root.join("extracted");
        fs::create_dir_all(source.join("nested/empty")).unwrap();
        fs::write(source.join("nested/server.properties"), b"motd=Sea Lantern").unwrap();

        let created = create_tar_gz(&source, &archive).unwrap();
        let extracted_summary = extract_tar_gz(&archive, &extracted).unwrap();

        assert_eq!(created.files, 1);
        assert_eq!(created.directories, 2);
        assert_eq!(created.bytes, 16);
        assert_eq!(created.files, extracted_summary.files);
        assert_eq!(created.directories, extracted_summary.directories);
        assert_eq!(
            fs::read(extracted.join("nested/server.properties")).unwrap(),
            b"motd=Sea Lantern"
        );
        // 空目录必须往返保留，这依赖显式写出的目录条目。
        assert!(extracted.join("nested/empty").is_dir());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn produces_byte_identical_archives_for_identical_content() {
        let root = crate::fs::test_dir("tar-reproducible");
        let source = root.join("source");
        fs::create_dir_all(source.join("config")).unwrap();
        fs::write(source.join("config/a.txt"), b"alpha").unwrap();
        fs::write(source.join("config/b.txt"), b"beta").unwrap();

        create_tar_gz(&source, root.join("first.tar.gz")).unwrap();
        create_tar_gz(&source, root.join("second.tar.gz")).unwrap();

        // 固定 mode/mtime/uid/gid 与排序遍历共同保证归档可复现。
        assert_eq!(
            fs::read(root.join("first.tar.gz")).unwrap(),
            fs::read(root.join("second.tar.gz")).unwrap()
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archives_paths_longer_than_the_ustar_name_field() {
        let root = crate::fs::test_dir("tar-write-long-name");
        let source = root.join("source");
        let archive = root.join("long.tar.gz");
        let extracted = root.join("extracted");
        // Minecraft 数据包路径很容易超过 ustar 的 100 字节 name 字段，
        // 此时应由 tar 自动写出 GNU longname 扩展头而非报错。
        let nested = source
            .join("datapacks")
            .join("a".repeat(80))
            .join("data")
            .join("b".repeat(60))
            .join("functions");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("tick.mcfunction"), b"say hi\n").unwrap();

        create_tar_gz(&source, &archive).unwrap();
        let summary = extract_tar_gz(&archive, &extracted).unwrap();

        assert_eq!(summary.files, 1);
        let relative = nested.strip_prefix(&source).unwrap();
        assert_eq!(
            fs::read(extracted.join(relative).join("tick.mcfunction")).unwrap(),
            b"say hi\n"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn honours_the_requested_compression_level() {
        let root = crate::fs::test_dir("tar-level");
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        // 高度可压缩且足够大的负载，使级别差异体现在输出体积上。
        fs::write(source.join("payload.bin"), "sea lantern ".repeat(8192)).unwrap();

        let low = root.join("low.tar.gz");
        let high = root.join("high.tar.gz");
        create_tar_gz_with_level(&source, &low, CompressionLevel::Low).unwrap();
        create_tar_gz_with_level(&source, &high, CompressionLevel::High).unwrap();

        assert!(
            fs::metadata(&high).unwrap().len() < fs::metadata(&low).unwrap().len(),
            "high compression should produce a smaller archive"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_replace_existing_archive() {
        let root = crate::fs::test_dir("tar-existing-output");
        let source = root.join("source");
        let archive = root.join("server.tar.gz");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("server.properties"), b"motd=Sea Lantern").unwrap();
        fs::write(&archive, b"existing archive").unwrap();

        assert!(matches!(
            create_tar_gz(&source, &archive),
            Err(ArchiveError::DestinationExists { .. })
        ));
        assert_eq!(fs::read(&archive).unwrap(), b"existing archive");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn leaves_no_temporary_file_behind() {
        let root = crate::fs::test_dir("tar-write-temporary");
        let source = root.join("source");
        let output = root.join("output");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("server.properties"), b"motd=Sea Lantern").unwrap();

        create_tar_gz(&source, output.join("server.tar.gz")).unwrap();

        let published: Vec<_> = fs::read_dir(&output)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(published, vec![std::ffi::OsString::from("server.tar.gz")]);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_a_source_file_that_shrank_while_being_archived() {
        // 直接验证 ExactSizeReader 的下限约束：声明 16 字节但只提供 5 字节。
        let root = crate::fs::test_dir("tar-shrink");
        let temporary = root.join("archive.tmp");
        fs::create_dir_all(&root).unwrap();
        let file = fs::File::create(&temporary).unwrap();
        let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));

        let result = append_file(
            &mut builder,
            "server.properties",
            16,
            &b"short"[..],
            &temporary,
            Path::new("source/server.properties"),
        );

        assert!(matches!(result, Err(ArchiveError::InvalidSource { .. })));

        fs::remove_dir_all(root).unwrap();
    }
}
