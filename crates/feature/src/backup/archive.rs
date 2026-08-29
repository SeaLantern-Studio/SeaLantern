use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use sealantern_infra::archive::{ExtractionLimits, extract_zip_with_limits, is_symbolic_link};
use sealantern_infra::fs::SafeRelativePath;
use sealantern_infra::platform::collect_resource_snapshot;
use tar::Builder as TarBuilder;
use tracing::{debug, warn};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::error::{BackupError, BackupResult};
use super::models::{BackupFormat, CompressionLevel};

const MAX_ARCHIVE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
const MAX_ENTRY_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_COMPRESSION_RATIO: u64 = 200;
const MAX_ENTRY_PATH_BYTES: usize = 4096;

const MIN_AVAILABLE_MEMORY_BYTES: u64 = 128 * 1024 * 1024;
const STREAMING_MEMORY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_INDEX_MEMORY_BYTES: u64 = 128 * 1024 * 1024;

const BACKUP_TARGET: &str = "sealantern.feature.backup";
const EVENT_MEMORY_PREFLIGHT: &str = "backup_restore_memory_preflight";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ArchiveStats {
    pub archive_bytes: u64,
    pub entries: usize,
    pub unpacked_bytes: u64,
}

pub(crate) fn create_archive(
    source: &Path,
    destination: &Path,
    format: BackupFormat,
    compression_level: CompressionLevel,
) -> BackupResult<()> {
    ensure_source_directory(source)?;
    reject_existing_destination(destination)?;

    let temporary = temporary_path(destination);
    let result = match format {
        BackupFormat::Zip => create_zip_archive(source, &temporary, compression_level),
        BackupFormat::TarGz => create_tar_archive(source, &temporary, compression_level),
    };

    match result {
        Ok(()) => {
            let validation = validate_created_archive(&temporary, format);
            if let Err(error) = validation {
                remove_temporary_file(&temporary);
                Err(error)
            } else {
                publish_archive(&temporary, destination)
            }
        }
        Err(error) => {
            remove_temporary_file(&temporary);
            Err(error)
        }
    }
}

pub(crate) fn extract_archive(
    archive_path: &Path,
    destination: &Path,
    format: BackupFormat,
) -> BackupResult<ArchiveStats> {
    let effective_format = resolve_archive_format(archive_path, format)?;
    let stats = preflight_archive(archive_path, effective_format)?;
    reject_existing_destination(destination)?;
    fs::create_dir_all(destination.parent().unwrap_or_else(|| Path::new(".")))?;

    match effective_format {
        BackupFormat::Zip => {
            extract_zip_with_limits(archive_path, destination, zip_extraction_limits())?;
        }
        BackupFormat::TarGz => {
            extract_tar_archive(archive_path, destination, stats.archive_bytes)?;
        }
    }

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

fn validate_created_archive(path: &Path, format: BackupFormat) -> BackupResult<()> {
    let archive_bytes = archive_size(path)?;
    match format {
        BackupFormat::Zip => {
            inspect_zip_archive(path, archive_bytes)?;
        }
        BackupFormat::TarGz => {
            inspect_tar_archive(path, archive_bytes)?;
        }
    }
    Ok(())
}

fn preflight_archive(archive_path: &Path, format: BackupFormat) -> BackupResult<ArchiveStats> {
    let archive_bytes = archive_size(archive_path)?;
    let initial_stats = ArchiveStats {
        archive_bytes,
        entries: MAX_ENTRIES,
        ..ArchiveStats::default()
    };
    check_memory_budget(archive_path, format, initial_stats)?;

    let stats = match format {
        BackupFormat::Zip => inspect_zip_archive(archive_path, archive_bytes)?,
        BackupFormat::TarGz => inspect_tar_archive(archive_path, archive_bytes)?,
    };
    check_memory_budget(archive_path, format, stats)?;
    Ok(stats)
}

fn check_memory_budget(
    archive_path: &Path,
    format: BackupFormat,
    stats: ArchiveStats,
) -> BackupResult<()> {
    let required_memory = required_memory(stats, format);
    let available_memory = collect_resource_snapshot().available_memory_bytes;

    if !has_memory_budget(available_memory, required_memory) {
        warn!(
            target: BACKUP_TARGET,
            event_name = EVENT_MEMORY_PREFLIGHT,
            status = "rejected",
            format = %format,
            archive = %archive_path.display(),
            archive_bytes = stats.archive_bytes,
            entries = stats.entries,
            estimated_unpacked_bytes = stats.unpacked_bytes,
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
        archive_bytes = stats.archive_bytes,
        entries = stats.entries,
        estimated_unpacked_bytes = stats.unpacked_bytes,
        available_memory_bytes = available_memory,
        required_memory_bytes = required_memory,
        "backup restore memory preflight passed"
    );
    Ok(())
}

fn has_memory_budget(available: u64, required: u64) -> bool {
    available >= required
}

fn required_memory(stats: ArchiveStats, format: BackupFormat) -> u64 {
    let index_memory = (stats.entries as u64)
        .saturating_mul(8 * 1024)
        .min(MAX_INDEX_MEMORY_BYTES);
    let path_memory = (stats.entries as u64)
        .saturating_mul(MAX_ENTRY_PATH_BYTES as u64)
        .min(64 * 1024 * 1024);
    let archive_overhead = match format {
        BackupFormat::Zip => (stats.archive_bytes / 4).min(512 * 1024 * 1024),
        BackupFormat::TarGz => 0,
    };
    let output_overhead = (stats.unpacked_bytes / 256).min(64 * 1024 * 1024);
    MIN_AVAILABLE_MEMORY_BYTES
        .saturating_add(STREAMING_MEMORY_BYTES)
        .saturating_add(index_memory)
        .saturating_add(path_memory)
        .saturating_add(archive_overhead)
        .saturating_add(output_overhead)
}

fn zip_extraction_limits() -> ExtractionLimits {
    ExtractionLimits {
        max_archive_bytes: MAX_ARCHIVE_BYTES,
        max_entries: MAX_ENTRIES,
        max_entry_bytes: MAX_ENTRY_BYTES,
        max_total_bytes: MAX_TOTAL_BYTES,
        max_entry_path_bytes: MAX_ENTRY_PATH_BYTES,
        max_compression_ratio: MAX_COMPRESSION_RATIO,
    }
}

fn create_zip_archive(
    source: &Path,
    destination: &Path,
    compression_level: CompressionLevel,
) -> BackupResult<()> {
    let file = create_temporary_archive(destination)?;
    let mut writer = ZipWriter::new(file);
    let file_options = zip_file_options(compression_level);
    let directory_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);
    let mut directories = vec![PathBuf::new()];

    while let Some(directory) = directories.pop() {
        let directory_path = source.join(&directory);
        for entry in fs::read_dir(&directory_path)? {
            let entry = entry?;
            let relative = directory.join(entry.file_name());
            let source_path = source.join(&relative);
            let metadata = validate_source_entry(&source_path)?;
            let name = portable_name(&relative)?;

            if metadata.is_dir() {
                writer.add_directory(name, directory_options)?;
                directories.push(relative);
            } else {
                writer.start_file(name, file_options)?;
                let mut input = File::open(&source_path)?;
                io::copy(&mut input, &mut writer)?;
            }
        }
    }

    writer.finish()?;
    Ok(())
}

fn create_tar_archive(
    source: &Path,
    destination: &Path,
    compression_level: CompressionLevel,
) -> BackupResult<()> {
    let file = create_temporary_archive(destination)?;
    let encoder = GzEncoder::new(file, flate2_compression(compression_level));
    let mut builder = TarBuilder::new(encoder);
    let mut directories = vec![PathBuf::new()];

    while let Some(directory) = directories.pop() {
        let directory_path = source.join(&directory);
        for entry in fs::read_dir(&directory_path)? {
            let entry = entry?;
            let relative = directory.join(entry.file_name());
            let source_path = source.join(&relative);
            let metadata = validate_source_entry(&source_path)?;
            let name = portable_name(&relative)?;
            ensure_tar_path(&name)?;

            if metadata.is_dir() {
                builder.append_dir(&name, &source_path)?;
                directories.push(relative);
            } else {
                let mut header = tar::Header::new_gnu();
                header.set_mode(0o644);
                header.set_size(metadata.len());
                let mut input = File::open(&source_path)?;
                builder.append_data(&mut header, &name, &mut input)?;
            }
        }
    }

    let encoder = builder.into_inner()?;
    encoder.finish()?;
    Ok(())
}

fn extract_tar_archive(
    archive_path: &Path,
    destination: &Path,
    archive_bytes: u64,
) -> BackupResult<()> {
    fs::create_dir(destination)?;
    let file = File::open(archive_path)?;
    let mut decoder = GzDecoder::new(file);
    let mut unpacked_bytes = 0_u64;

    while let Some(entry) = next_tar_entry(&mut decoder, archive_path)? {
        let output_path = destination.join(&entry.path);
        if entry.is_directory {
            ensure_parent_directory(&output_path)?;
            match fs::create_dir(&output_path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if !is_normal_directory(&output_path)? {
                        return Err(invalid_archive(archive_path, "目录路径与文件路径冲突"));
                    }
                }
                Err(error) => return Err(BackupError::Io(error)),
            }
            skip_tar_payload(&mut decoder, entry.size, archive_path)?;
        } else {
            check_entry_size(archive_path, entry.size)?;
            unpacked_bytes = checked_total_size(archive_path, unpacked_bytes, entry.size)?;
            ensure_parent_directory(&output_path)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output_path)?;
            copy_tar_payload(&mut decoder, &mut output, entry.size, archive_path)?;
        }
    }
    finish_tar_stream(&mut decoder, archive_path, unpacked_bytes, archive_bytes)?;
    Ok(())
}

fn inspect_zip_archive(archive_path: &Path, archive_bytes: u64) -> BackupResult<ArchiveStats> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    if archive.len() > MAX_ENTRIES {
        return Err(invalid_archive(archive_path, format!("条目数量超过限制 {}", MAX_ENTRIES)));
    }

    let mut paths = HashSet::with_capacity(archive.len());
    let mut files = HashSet::new();
    let mut unpacked_bytes = 0_u64;

    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let entry_name = entry.name();
        if entry_name.len() > MAX_ENTRY_PATH_BYTES {
            return Err(invalid_archive(archive_path, "条目路径过长"));
        }
        let relative = safe_entry_path(archive_path, entry_name)?;
        let is_directory = entry.is_dir();
        validate_entry_path_conflict(archive_path, &relative, is_directory, &paths, &files)?;
        if !paths.insert(relative.clone()) {
            return Err(invalid_archive(archive_path, "归档包含重复条目路径"));
        }
        if is_symbolic_link(entry.unix_mode()) {
            return Err(invalid_archive(archive_path, "归档包含符号链接"));
        }
        if is_directory {
            continue;
        }

        let entry_bytes = entry.size();
        check_entry_size(archive_path, entry_bytes)?;
        unpacked_bytes = checked_total_size(archive_path, unpacked_bytes, entry_bytes)?;
        let compressed_bytes = entry.compressed_size();
        if entry_bytes > 0
            && (compressed_bytes == 0
                || entry_bytes > compressed_bytes.saturating_mul(MAX_COMPRESSION_RATIO))
        {
            return Err(invalid_archive(archive_path, "压缩比超过限制"));
        }
        files.insert(relative);
    }

    Ok(ArchiveStats {
        archive_bytes,
        entries: archive.len(),
        unpacked_bytes,
    })
}

fn inspect_tar_archive(archive_path: &Path, archive_bytes: u64) -> BackupResult<ArchiveStats> {
    let file = File::open(archive_path)?;
    let mut decoder = GzDecoder::new(file);
    let mut entries = 0_usize;
    let mut paths = HashSet::new();
    let mut files = HashSet::new();
    let mut unpacked_bytes = 0_u64;

    while let Some(entry) = next_tar_entry(&mut decoder, archive_path)? {
        entries += 1;
        if entries > MAX_ENTRIES {
            return Err(invalid_archive(archive_path, format!("条目数量超过限制 {}", MAX_ENTRIES)));
        }

        validate_entry_path_conflict(
            archive_path,
            &entry.path,
            entry.is_directory,
            &paths,
            &files,
        )?;
        if !paths.insert(entry.path.clone()) {
            return Err(invalid_archive(archive_path, "归档包含重复条目路径"));
        }
        if entry.is_directory {
            skip_tar_payload(&mut decoder, entry.size, archive_path)?;
            continue;
        }

        check_entry_size(archive_path, entry.size)?;
        unpacked_bytes = checked_total_size(archive_path, unpacked_bytes, entry.size)?;
        files.insert(entry.path);
        skip_tar_payload(&mut decoder, entry.size, archive_path)?;
    }

    finish_tar_stream(&mut decoder, archive_path, unpacked_bytes, archive_bytes)?;

    Ok(ArchiveStats { archive_bytes, entries, unpacked_bytes })
}

struct TarEntryHeader {
    path: PathBuf,
    is_directory: bool,
    size: u64,
}

fn next_tar_entry<R: Read>(
    reader: &mut R,
    archive_path: &Path,
) -> BackupResult<Option<TarEntryHeader>> {
    let mut header = [0_u8; 512];
    read_tar_block(reader, &mut header, archive_path)?;
    if header.iter().all(|byte| *byte == 0) {
        let mut end_block = [0_u8; 512];
        read_tar_block(reader, &mut end_block, archive_path)?;
        if !end_block.iter().all(|byte| *byte == 0) {
            return Err(invalid_archive(archive_path, "归档结束标记不完整"));
        }
        return Ok(None);
    }

    validate_tar_checksum(&header, archive_path)?;
    let type_flag = header[156];
    let is_directory = type_flag == b'5';
    if type_flag != 0 && type_flag != b'0' && !is_directory {
        return Err(invalid_archive(archive_path, "归档包含不支持的特殊或扩展条目"));
    }

    let path = tar_entry_path(&header, archive_path)?;
    let size = parse_tar_number(&header[124..136], archive_path, "条目大小")?;
    if is_directory && size != 0 {
        return Err(invalid_archive(archive_path, "目录条目包含文件数据"));
    }
    if path.to_string_lossy().len() > MAX_ENTRY_PATH_BYTES {
        return Err(invalid_archive(archive_path, "条目路径过长"));
    }
    let path = safe_entry_path(archive_path, &path.to_string_lossy())?;

    Ok(Some(TarEntryHeader { path, is_directory, size }))
}

fn read_tar_block<R: Read>(
    reader: &mut R,
    block: &mut [u8; 512],
    archive_path: &Path,
) -> BackupResult<()> {
    reader.read_exact(block).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            invalid_archive(archive_path, "归档在完整结束前截断")
        } else {
            BackupError::Io(error)
        }
    })
}

fn validate_tar_checksum(header: &[u8; 512], archive_path: &Path) -> BackupResult<()> {
    let expected = parse_tar_number(&header[148..156], archive_path, "header校验和")?;
    let actual = header
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if (148..156).contains(&index) {
                b' ' as u64
            } else {
                *byte as u64
            }
        })
        .sum::<u64>();
    if actual != expected {
        return Err(invalid_archive(
            archive_path,
            format!("header校验和不匹配: 声明值 {expected}，计算值 {actual}"),
        ));
    }
    Ok(())
}

fn parse_tar_number(field: &[u8], archive_path: &Path, field_name: &str) -> BackupResult<u64> {
    if field.first().is_some_and(|byte| byte & 0x80 != 0) {
        let mut value = (field[0] & 0x7f) as u64;
        for byte in &field[1..] {
            value = value
                .checked_shl(8)
                .and_then(|value| value.checked_add(*byte as u64))
                .ok_or_else(|| invalid_archive(archive_path, format!("{field_name}溢出")))?;
        }
        return Ok(value);
    }

    let start = field
        .iter()
        .position(|byte| *byte != 0 && *byte != b' ')
        .unwrap_or(field.len());
    let end = start
        + field[start..]
            .iter()
            .position(|byte| *byte == 0 || *byte == b' ')
            .unwrap_or(field.len() - start);
    let digits = &field[start..end];
    if digits.is_empty() {
        return Ok(0);
    }
    let mut value = 0_u64;
    for digit in digits {
        if !(b'0'..=b'7').contains(digit) {
            return Err(invalid_archive(archive_path, format!("{field_name}不是八进制数字")));
        }
        value = value
            .checked_mul(8)
            .and_then(|value| value.checked_add((digit - b'0') as u64))
            .ok_or_else(|| invalid_archive(archive_path, format!("{field_name}溢出")))?;
    }
    Ok(value)
}

fn tar_entry_path(header: &[u8; 512], archive_path: &Path) -> BackupResult<PathBuf> {
    let name = tar_text_field(&header[..100], archive_path, "条目名称")?;
    let prefix = tar_text_field(&header[345..500], archive_path, "条目前缀")?;
    let path = if prefix.is_empty() {
        name
    } else if name.is_empty() {
        return Err(invalid_archive(archive_path, "条目名称为空"));
    } else {
        format!("{prefix}/{name}")
    };
    if path.is_empty() {
        return Err(invalid_archive(archive_path, "条目名称为空"));
    }
    Ok(PathBuf::from(path))
}

fn tar_text_field(field: &[u8], archive_path: &Path, field_name: &str) -> BackupResult<String> {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    std::str::from_utf8(&field[..end])
        .map(str::to_owned)
        .map_err(|_| invalid_archive(archive_path, format!("{field_name}不是UTF-8")))
}

fn skip_tar_payload<R: Read>(reader: &mut R, size: u64, archive_path: &Path) -> BackupResult<()> {
    let padded_size = size
        .checked_add(511)
        .map(|size| size / 512 * 512)
        .ok_or_else(|| invalid_archive(archive_path, "条目大小溢出"))?;
    skip_tar_bytes(reader, padded_size, archive_path)
}

fn skip_tar_bytes<R: Read>(reader: &mut R, bytes: u64, archive_path: &Path) -> BackupResult<()> {
    let mut remaining = bytes;
    let mut buffer = [0_u8; 64 * 1024];
    while remaining > 0 {
        let requested = remaining.min(buffer.len() as u64) as usize;
        let count = reader.read(&mut buffer[..requested])?;
        if count == 0 {
            return Err(invalid_archive(archive_path, "条目数据不完整"));
        }
        remaining -= count as u64;
    }
    Ok(())
}

fn copy_tar_payload<R: Read, W: Write>(
    reader: &mut R,
    output: &mut W,
    size: u64,
    archive_path: &Path,
) -> BackupResult<()> {
    let copied = {
        let mut limited = reader.take(size);
        io::copy(&mut limited, output)?
    };
    if copied != size {
        return Err(invalid_archive(archive_path, "条目数据不完整"));
    }
    let padding = (512 - size % 512) % 512;
    skip_tar_bytes(reader, padding, archive_path)
}

fn finish_tar_stream<R: Read>(
    reader: &mut R,
    archive_path: &Path,
    unpacked_bytes: u64,
    archive_bytes: u64,
) -> BackupResult<()> {
    let mut byte = [0_u8; 1];
    if reader.read(&mut byte)? != 0 {
        return Err(invalid_archive(archive_path, "归档结束标记后存在数据"));
    }
    if unpacked_bytes > 0
        && (archive_bytes == 0
            || unpacked_bytes > archive_bytes.saturating_mul(MAX_COMPRESSION_RATIO))
    {
        return Err(invalid_archive(archive_path, "总压缩比超过限制"));
    }
    Ok(())
}

fn ensure_parent_directory(path: &Path) -> BackupResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn is_normal_directory(path: &Path) -> BackupResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(!metadata.file_type().is_symlink() && metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(BackupError::Io(error)),
    }
}

fn validate_entry_path_conflict(
    archive_path: &Path,
    path: &Path,
    is_directory: bool,
    paths: &HashSet<PathBuf>,
    files: &HashSet<PathBuf>,
) -> BackupResult<()> {
    if path
        .ancestors()
        .skip(1)
        .any(|ancestor| files.contains(ancestor))
    {
        return Err(invalid_archive(archive_path, "文件条目阻断了目录路径"));
    }
    if !is_directory && paths.iter().any(|existing| existing.starts_with(path)) {
        return Err(invalid_archive(archive_path, "文件和目录路径发生冲突"));
    }
    Ok(())
}

fn safe_entry_path(archive_path: &Path, path: &str) -> BackupResult<PathBuf> {
    SafeRelativePath::parse(path)
        .map(|path| path.as_path().to_path_buf())
        .map_err(|error| invalid_archive(archive_path, error.to_string()))
}

fn check_entry_size(archive_path: &Path, size: u64) -> BackupResult<()> {
    if size > MAX_ENTRY_BYTES {
        return Err(invalid_archive(archive_path, "单个条目大小超过限制"));
    }
    Ok(())
}

fn checked_total_size(archive_path: &Path, current: u64, added: u64) -> BackupResult<u64> {
    let total = current
        .checked_add(added)
        .ok_or_else(|| invalid_archive(archive_path, "条目总大小溢出"))?;
    if total > MAX_TOTAL_BYTES {
        return Err(invalid_archive(archive_path, "条目总大小超过限制"));
    }
    Ok(total)
}

fn ensure_source_directory(path: &Path) -> BackupResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BackupError::Validation(format!("归档源不是普通目录: {:?}", path)));
    }
    Ok(())
}

fn validate_source_entry(path: &Path) -> BackupResult<std::fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(BackupError::Validation(format!("归档源包含符号链接: {:?}", path)));
    }
    if !metadata.is_dir() && !metadata.is_file() {
        return Err(BackupError::Validation(format!("归档源包含不支持的特殊文件: {:?}", path)));
    }
    Ok(metadata)
}

fn portable_name(path: &Path) -> BackupResult<String> {
    let mut name = String::new();
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(BackupError::Validation(format!("归档路径包含非法组件: {:?}", path)));
        }
        let component = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| BackupError::Validation("归档路径包含非UTF-8文件名".to_string()))?;
        if !name.is_empty() {
            name.push('/');
        }
        name.push_str(component);
    }
    SafeRelativePath::parse(&name)
        .map_err(|error| BackupError::Validation(format!("归档路径无效: {}", error)))?;
    Ok(name)
}

fn ensure_tar_path(path: &str) -> BackupResult<()> {
    let bytes = path.as_bytes();
    if bytes.len() <= 100 {
        return Ok(());
    }

    let Some(separator) = path.rfind('/') else {
        return Err(BackupError::Validation(format!(
            "TarGz条目路径无法用固定header表示: {}",
            path
        )));
    };
    let prefix_bytes = path[..separator].len();
    let name_bytes = path[separator + 1..].len();
    if prefix_bytes <= 155 && name_bytes <= 100 {
        Ok(())
    } else {
        Err(BackupError::Validation(format!("TarGz条目路径无法用固定header表示: {}", path)))
    }
}

fn resolve_archive_format(path: &Path, requested: BackupFormat) -> BackupResult<BackupFormat> {
    if requested == BackupFormat::TarGz && has_zip_magic(path)? {
        return Ok(BackupFormat::Zip);
    }
    Ok(requested)
}

fn has_zip_magic(path: &Path) -> BackupResult<bool> {
    let mut file = File::open(path)?;
    let mut magic = [0_u8; 4];
    if file.read(&mut magic)? != magic.len() {
        return Ok(false);
    }
    Ok(magic[0] == b'P'
        && magic[1] == b'K'
        && matches!((magic[2], magic[3]), (3, 4) | (5, 6) | (7, 8)))
}

fn archive_size(path: &Path) -> BackupResult<u64> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(BackupError::CorruptedBackup(path.to_path_buf()));
    }
    if metadata.len() > MAX_ARCHIVE_BYTES {
        return Err(invalid_archive(path, "归档大小超过限制"));
    }
    Ok(metadata.len())
}

fn reject_existing_destination(path: &Path) -> BackupResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(BackupError::AlreadyExists(path.to_path_buf())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(BackupError::Io(error)),
    }
}

fn create_temporary_archive(path: &Path) -> BackupResult<File> {
    fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))?;
    let temporary = path.to_path_buf();
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(BackupError::Io)
}

fn publish_archive(temporary: &Path, destination: &Path) -> BackupResult<()> {
    let result = fs::hard_link(temporary, destination);
    remove_temporary_file(temporary);
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(BackupError::AlreadyExists(destination.to_path_buf()))
        }
        Err(error) => Err(BackupError::Io(error)),
    }
}

fn remove_temporary_file(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != io::ErrorKind::NotFound
    {
        warn!(
            target: BACKUP_TARGET,
            event_name = "backup_archive_cleanup_failed",
            path = %path.display(),
            error = %error,
            "temporary backup archive cleanup failed"
        );
    }
}

fn temporary_path(destination: &Path) -> PathBuf {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("backup.archive");
    destination
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".{name}.{}.tmp", Uuid::new_v4()))
}

fn zip_file_options(compression_level: CompressionLevel) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(compression_level_value(compression_level)))
        .large_file(true)
}

fn compression_level_value(level: CompressionLevel) -> i64 {
    match level {
        CompressionLevel::Low => 1,
        CompressionLevel::Medium => 6,
        CompressionLevel::High => 9,
    }
}

fn flate2_compression(level: CompressionLevel) -> Compression {
    Compression::new(compression_level_value(level) as u32)
}

fn invalid_archive(path: &Path, reason: impl Into<String>) -> BackupError {
    BackupError::Validation(format!("归档 {:?} 无效: {}", path, reason.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_budget_reserves_parser_space_before_unpack() {
        let small_archive = ArchiveStats {
            archive_bytes: 1,
            entries: 1,
            unpacked_bytes: 1,
        };
        let worst_case = ArchiveStats {
            archive_bytes: MAX_ARCHIVE_BYTES,
            entries: MAX_ENTRIES,
            unpacked_bytes: MAX_TOTAL_BYTES,
        };
        let small_required = required_memory(small_archive, BackupFormat::TarGz);
        let worst_required = required_memory(worst_case, BackupFormat::Zip);

        assert!(small_required >= MIN_AVAILABLE_MEMORY_BYTES);
        assert!(worst_required > small_required);
        assert!(!has_memory_budget(worst_required - 1, worst_required));
        assert!(has_memory_budget(worst_required, worst_required));
    }

    #[test]
    fn archive_entry_paths_use_the_shared_safe_path_parser() {
        let archive_path = Path::new("backup.zip");
        let valid_prefixes = ["", "config/", "config/world/"];
        let valid_names = ["server.properties", "level.dat", "中文.dat"];
        for prefix in valid_prefixes {
            for name in valid_names {
                assert!(safe_entry_path(archive_path, &format!("{prefix}{name}")).is_ok());
            }
        }

        for invalid in [
            "",
            ".",
            "..",
            "../outside",
            "config/../../outside",
            r"config\server.properties",
            "/absolute/path",
        ] {
            assert!(safe_entry_path(archive_path, invalid).is_err(), "{invalid} should fail");
        }
    }
}
