use std::cell::Cell;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use cap_std::fs::{Dir, OpenOptions};
use flate2::read::GzDecoder;
use tar::{Entry, EntryType};

use super::limits::{ExtractionLimits, ExtractionSummary, accumulate_bytes, check_limit};
use super::{
    ArchiveError, EntryPathRegistry, check_entry_path_length, create_new_directory,
    ensure_directory, ensure_parent_dirs, is_symbolic_link, parent_path,
    parse_symbolic_link_target, publish_new, safe_entry_path,
};

const MAX_SYMBOLIC_LINK_TARGET_BYTES: usize = 4 * 1024;

/// 单个「读取段」允许的最大解压字节数。
///
/// tar 会在 `next_entry` 内部把 GNU longname / longlink 与 PAX 扩展头的全部
/// 内容读进 `Vec<u8>`，且不施加任何上限（`Entry::read_all` 的 `with_capacity`
/// 只限制初始容量，随后的 `read_to_end` 无界增长）。这意味着恶意归档只要把
/// 扩展头的 size 字段声明成数 GB，就能在任何条目交到本模块手上之前耗尽内存，
/// [`ExtractionLimits::max_entry_bytes`] 完全拦不住。
///
/// 因此解码器按「段」限流：调用方每确认消费一段数据就重置计数，两次重置之间
/// 的累计解压字节不得超过此上限。条目内容由 [`copy_entry_with_limits`] 逐块
/// 读取并逐块重置，大文件不受影响；扩展头的一次性读取全部落在同一段内，声明
/// 过大即被拒绝。
///
/// 上限取 1 MiB：合法扩展头承载的是路径与元数据，量级为几 KiB（tar 自身的
/// 路径上限即 4 KiB 级别），留出两个数量级余量已足够，同时把最坏一次性内存
/// 分配约束在可忽略的范围。
const MAX_SEGMENT_BYTES: u64 = 1024 * 1024;

/// tar 结束标记之后允许的最大填充字节数。
///
/// tar 以两个全零块表示归档结束，其后按实现惯例还会补齐到固定记录大小：
/// GNU tar 默认的 blocking factor 为 20，即补齐到 10240 字节的整数倍。因此
/// 结束标记之后出现成片零字节是合法的，只是不允许出现非零数据。
///
/// 上限取 64 KiB，覆盖各实现常用的记录大小，同时避免用无限零字节流拖住解压。
const MAX_TRAILING_PADDING_BYTES: u64 = 64 * 1024;

/// 被解码器拦下的一次限制越界。
///
/// 解码器只能返回 [`io::Error`]，无法直接携带 [`ArchiveError::LimitExceeded`]
/// 所需的归档路径与计数。这里记录足以重建该错误的最小信息，由调用方在拿到
/// I/O 错误后取出并还原为精确的限制错误。
#[derive(Clone, Copy, Debug)]
struct LimitBreach {
    limit: &'static str,
    observed: u64,
    maximum: u64,
}

/// gzip 解压流的字节计量与限制状态。
///
/// 由解码器与解压循环共享：解码器在每次读取后记账，解压循环在确认消费完一段
/// 数据后重置段计数。所有字段都是 `Cell`，因为 [`Read`] 只提供 `&mut self`
/// 而状态需要在被 `tar::Archive` 拿走所有权后仍可从外部访问。
struct StreamLimits {
    /// 本段已解压字节数，见 [`MAX_SEGMENT_BYTES`]。
    segment: Cell<u64>,
    /// 全流已解压字节数，含 tar header 与块填充。
    total: Cell<u64>,
    /// 全流解压字节上限。
    max_total: u64,
    /// 由归档文件字节数与最大压缩比推得的解压字节上限。
    ///
    /// 仅在解压总量超过 [`ExtractionLimits::min_ratio_enforcement_bytes`] 后
    /// 生效。
    max_ratio_bytes: u64,
    /// 开始施加压缩比上限的解压字节数。
    min_ratio_bytes: u64,
    /// 最近一次越界记录，供调用方还原精确错误。
    breach: Cell<Option<LimitBreach>>,
}

impl StreamLimits {
    fn new(archive_size: u64, limits: ExtractionLimits) -> Self {
        Self {
            segment: Cell::new(0),
            total: Cell::new(0),
            max_total: limits.max_total_bytes,
            max_ratio_bytes: archive_size.saturating_mul(limits.max_compression_ratio),
            min_ratio_bytes: limits.min_ratio_enforcement_bytes,
            breach: Cell::new(None),
        }
    }

    /// 记录本次解压出的字节数，超过任一上限时返回错误。
    fn account(&self, count: u64) -> io::Result<()> {
        let segment = self.segment.get().saturating_add(count);
        self.segment.set(segment);
        if segment > MAX_SEGMENT_BYTES {
            return Err(self.breach("uncompressed segment bytes", segment, MAX_SEGMENT_BYTES));
        }

        let Some(total) = self.total.get().checked_add(count) else {
            return Err(self.breach("total uncompressed bytes", u64::MAX, self.max_total));
        };
        self.total.set(total);
        if total > self.max_total {
            return Err(self.breach("total uncompressed bytes", total, self.max_total));
        }
        if total > self.max_ratio_bytes && total > self.min_ratio_bytes {
            return Err(self.breach("compression ratio", total, self.max_ratio_bytes));
        }
        Ok(())
    }

    fn breach(&self, limit: &'static str, observed: u64, maximum: u64) -> io::Error {
        self.breach
            .set(Some(LimitBreach { limit, observed, maximum }));
        io::Error::new(io::ErrorKind::InvalidData, "archive exceeds configured extraction limits")
    }

    /// 开始新的读取段。
    fn start_segment(&self) {
        self.segment.set(0);
    }

    fn take_breach(&self) -> Option<LimitBreach> {
        self.breach.take()
    }
}

/// 按 [`StreamLimits`] 记账的解压流包装器。
///
/// 位于 gzip 解码器与 tar 解析器之间，因此 tar 内部读取（header、块填充、
/// 扩展头内容）与条目内容读取一律计入。这是扩展头无界读取的唯一拦截点。
struct LimitedReader<R> {
    inner: R,
    limits: Rc<StreamLimits>,
}

impl<R: Read> Read for LimitedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.inner.read(buffer)?;
        self.limits.account(count as u64)?;
        Ok(count)
    }
}

/// 本模块使用的完整读取栈：文件 → 缓冲 → gzip 解压 → 限流。
type TarGzReader = LimitedReader<GzDecoder<BufReader<File>>>;

/// 把底层 I/O 错误还原为精确的归档错误。
///
/// 解码器越界时只能返回占位的 [`io::Error`]，真实原因记录在
/// [`StreamLimits::breach`] 中，优先取用。
fn stream_error(
    limits: &StreamLimits,
    operation: &'static str,
    archive_path: &Path,
    error: io::Error,
) -> ArchiveError {
    match limits.take_breach() {
        Some(breach) => ArchiveError::LimitExceeded {
            archive: archive_path.to_path_buf(),
            limit: breach.limit,
            observed: breach.observed,
            maximum: breach.maximum,
        },
        None => ArchiveError::tar(operation, archive_path, error),
    }
}

/// 使用默认限制将 tar.gz 归档解压到新的目标目录中。
///
/// 目标目录必须尚未存在。
pub fn extract_tar_gz(
    archive: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ExtractionSummary, ArchiveError> {
    extract_tar_gz_with_limits(archive, destination, ExtractionLimits::default())
}

/// 使用显式限制将 tar.gz 归档解压到新的目标目录中。
///
/// tar 是不可回退的流，没有 ZIP 那样的中央目录，无法在写入前完成全量预检。
/// 为了保持与 [`super::extract_zip`] 相同的「失败时目标目录从未出现」语义，
/// 解压先写入同一父目录下的临时目录，全部条目成功后经 [`publish_new`] 以
/// create-new 语义移动到目标位置；任何一步失败都会删除临时目录。
///
/// 因此所有条目名、重复路径、符号链接与字节上限都在流式读取过程中逐条校验，
/// 校验失败时已写入的部分随临时目录一起被丢弃。
pub fn extract_tar_gz_with_limits(
    archive: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    limits: ExtractionLimits,
) -> Result<ExtractionSummary, ArchiveError> {
    let archive = archive.as_ref();
    let destination = destination.as_ref();
    let result = extract_tar_gz_inner(archive, destination, limits);
    if let Err(error) = &result {
        crate::observability::archive_operation_failed_with_context(
            "extract tar.gz",
            archive,
            Some(destination),
            error.entry(),
            error,
        );
    }
    result
}

fn extract_tar_gz_inner(
    archive_path: &Path,
    destination: &Path,
    limits: ExtractionLimits,
) -> Result<ExtractionSummary, ArchiveError> {
    let archive_size = std::fs::metadata(archive_path)
        .map_err(|error| ArchiveError::io("read tar.gz archive metadata", archive_path, error))?
        .len();
    check_limit(archive_path, "compressed archive bytes", archive_size, limits.max_archive_bytes)?;
    reject_existing_destination(destination)?;

    let temporary = temporary_directory_path(destination);
    let root = create_new_directory(&temporary)?;
    let result = unpack_entries(&root, archive_path, destination, archive_size, limits);
    // Windows 上持有目录句柄会阻止移动，因此在发布之前释放。
    drop(root);
    match result {
        Ok(summary) => {
            if let Err(error) = publish_new(&temporary, destination) {
                remove_temporary_directory(&temporary);
                return Err(error);
            }
            Ok(summary)
        }
        Err(error) => {
            remove_temporary_directory(&temporary);
            Err(error)
        }
    }
}

/// 在做任何解压工作之前拒绝已存在的目标目录。
///
/// 临时目录发布时的 rename 也会因目标存在而失败，但提前检查能在浪费解压
/// 开销之前返回与 ZIP 一致的 [`ArchiveError::DestinationExists`]。
fn reject_existing_destination(destination: &Path) -> Result<(), ArchiveError> {
    match std::fs::symlink_metadata(destination) {
        Ok(_) => Err(ArchiveError::DestinationExists { path: destination.to_path_buf() }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(ArchiveError::io("read archive destination metadata", destination, error))
        }
    }
}

/// 与目标同父目录的临时解压目录路径，确保发布时的 rename 不跨分区。
fn temporary_directory_path(destination: &Path) -> PathBuf {
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("extracted");
    parent_path(destination).join(format!(".{filename}.{}.tmp", uuid::Uuid::new_v4()))
}

/// 清理未能发布的临时解压目录。
///
/// 仅在失败路径上调用；清理失败只记录日志，不覆盖调用方的原始错误。
fn remove_temporary_directory(path: &Path) {
    if let Err(error) = std::fs::remove_dir_all(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        crate::observability::archive_cleanup_failed(path, &error);
    }
}

/// 流式读取 tar.gz 条目并写入临时解压根目录。
///
/// `destination` 仅用于错误消息展示，实际写入始终通过 `root` 目录句柄完成。
///
/// gzip 解码器外层套 [`LimitedReader`]，使 tar 解析器的内部读取（header、块
/// 填充、GNU/PAX 扩展头内容）与条目内容一并受限，见 [`MAX_SEGMENT_BYTES`]。
fn unpack_entries(
    root: &Dir,
    archive_path: &Path,
    destination: &Path,
    archive_size: u64,
    limits: ExtractionLimits,
) -> Result<ExtractionSummary, ArchiveError> {
    let file = File::open(archive_path)
        .map_err(|error| ArchiveError::io("open tar.gz archive", archive_path, error))?;
    let stream_limits = Rc::new(StreamLimits::new(archive_size, limits));
    let reader: TarGzReader = LimitedReader {
        inner: GzDecoder::new(BufReader::new(file)),
        limits: Rc::clone(&stream_limits),
    };
    let mut archive = tar::Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|error| stream_error(&stream_limits, "read", archive_path, error))?;

    let mut summary = ExtractionSummary::default();
    let mut paths = EntryPathRegistry::default();
    let mut entry_count = 0_u64;

    for entry in entries {
        // 读取下一个条目头会连带消费上一个条目的剩余内容与块填充，因此在
        // 取到条目之后才重置段计数，避免把跳过的字节算进新的一段。
        let mut entry = entry.map_err(|error| {
            stream_error(&stream_limits, "read entry from", archive_path, error)
        })?;
        stream_limits.start_segment();
        entry_count += 1;
        check_limit(archive_path, "entry count", entry_count, limits.max_entries as u64)?;

        let entry_name = entry_display_name(&entry, archive_path, limits.max_entry_path_bytes)?;
        // 条目类型校验先于路径规范化：名为 `./` 的符号链接或设备节点也必须被
        // 拒绝，而不是因规范化后没有输出路径而被静默跳过。
        reject_unsupported_entry(&mut entry, archive_path, &entry_name)?;
        // 目录判定同时看 entry 类型与原始名称：声明为 Regular 但带尾部斜杠的
        // 条目（部分打包器对空目录的写法）仍按目录处理，与 ZIP 侧
        // `entry.is_dir()` 按尾斜杠判定的行为保持一致。
        let is_directory =
            entry.header().entry_type() == EntryType::Directory || entry_name.ends_with('/');
        let Some(normalized) = normalize_entry_name(&entry_name) else {
            // `./` 与 `/` 这类仅指代解压根目录的条目没有对应输出，跳过。
            continue;
        };
        let relative = safe_entry_path(archive_path, &normalized)?;
        paths.register(archive_path, &relative, &entry_name, is_directory)?;

        if is_directory {
            ensure_directory(root, &relative, destination)?;
            summary.directories += 1;
            continue;
        }

        ensure_parent_dirs(root, &relative, destination)?;
        let output_path = destination.join(&relative);
        let mut output = root
            .open_with(&relative, OpenOptions::new().write(true).create_new(true))
            .map_err(|error| ArchiveError::io("create tar.gz entry file", &output_path, error))?;
        copy_entry_with_limits(
            &mut entry,
            &mut output,
            &output_path,
            archive_path,
            &mut summary.bytes,
            &stream_limits,
            limits,
        )?;
        summary.files += 1;
    }

    // 条目迭代结束意味着 tar 读到了结束标记，但底层流可能还有数据。
    // `entries` 借用了 `archive`，for 循环已消费该迭代器，借用随之结束。
    reject_trailing_data(archive.into_inner(), &stream_limits, archive_path)?;
    Ok(summary)
}

/// 拒绝 tar 结束标记之后出现的非零数据。
///
/// tar 读到两个全零块即停止迭代，之后不再关心底层流。若此时仍有非零数据，说明
/// 归档被拼接过：解压方只会看到前一段内容，而其他工具可能读出完全不同的结果。
/// gzip 的 CRC 校验对此无效——追加的数据可以是另一个完整合法的 gzip 成员。
///
/// 成片零字节按记录对齐填充接受，见 [`MAX_TRAILING_PADDING_BYTES`]。
///
/// 接受完整的读取栈而非泛型 reader：`GzDecoder` 在单成员模式下读完一个成员即
/// 返回 EOF，因此这里读到的是同一个 gzip 成员内 tar 结束标记之后的字节。
fn reject_trailing_data(
    mut reader: TarGzReader,
    stream_limits: &StreamLimits,
    archive_path: &Path,
) -> Result<(), ArchiveError> {
    // 结束标记之后的读取不属于任何条目，单独作为一段计量。
    stream_limits.start_segment();
    let mut buffer = [0_u8; 8 * 1024];
    let mut padding = 0_u64;
    loop {
        let count = reader.read(&mut buffer).map_err(|error| {
            stream_error(stream_limits, "read trailing data from", archive_path, error)
        })?;
        if count == 0 {
            return Ok(());
        }
        if buffer[..count].iter().any(|byte| *byte != 0) {
            return Err(ArchiveError::UnsafeEntry {
                archive: archive_path.to_path_buf(),
                entry: String::new(),
                reason: "archive contains data after the end-of-archive marker".to_string(),
            });
        }
        padding = accumulate_bytes(
            padding,
            count as u64,
            archive_path,
            "trailing padding bytes",
            MAX_TRAILING_PADDING_BYTES,
        )?;
        check_limit(archive_path, "trailing padding bytes", padding, MAX_TRAILING_PADDING_BYTES)?;
        stream_limits.start_segment();
    }
}

/// 校验条目名长度并取出用于错误展示的名称。
///
/// 长度校验在转换为 `String` 之前完成：`from_utf8_lossy` 会为超长路径分配
/// 同等大小的内存，检查必须先于分配。名称本身按有损方式转换，因为展示用名称
/// 必须总能取到，真正的路径安全性由 [`safe_entry_path`] 判定。
fn entry_display_name<R: Read>(
    entry: &Entry<'_, R>,
    archive_path: &Path,
    max_entry_path_bytes: usize,
) -> Result<String, ArchiveError> {
    let raw = entry.path_bytes();
    check_entry_path_length(archive_path, &raw, max_entry_path_bytes)?;
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

/// 把 tar 条目名规范化为可交给 [`SafeRelativePath`] 校验的形式。
///
/// `tar czf x.tar.gz .` 会产出 `./`、`./config` 这类带前导当前目录标记的
/// 条目名，目录条目还会带尾部斜杠。两者对输出位置都没有影响，这里剥离后
/// 再校验；剥离结果为空表示条目就是解压根目录本身，返回 `None`。
///
/// 只处理前导 `./` 与尾部 `/`：路径中间出现的 `.` 或任何 `..` 仍会被
/// [`SafeRelativePath::parse`] 拒绝。
fn normalize_entry_name(entry_name: &str) -> Option<String> {
    let mut name = entry_name;
    while let Some(stripped) = name.strip_prefix("./") {
        name = stripped;
    }
    let name = name.trim_end_matches('/');
    if name.is_empty() || name == "." {
        return None;
    }
    Some(name.to_string())
}

/// 拒绝除普通文件与目录以外的所有条目类型。
///
/// 符号链接与硬链接会把写入重定向到解压根目录之外，FIFO 与设备节点在各平台
/// 上语义不一致，都不在本 API 的支持范围内。对符号链接额外读取并校验链接目标，
/// 以便在拒绝时给出与 ZIP 一致的精确原因。
fn reject_unsupported_entry<R: Read>(
    entry: &mut Entry<'_, R>,
    archive_path: &Path,
    entry_name: &str,
) -> Result<(), ArchiveError> {
    let header = entry.header();
    let entry_type = header.entry_type();
    // header mode 也可能把条目标记为符号链接，格式畸形时作为兜底检查。
    let claims_symbolic_link =
        entry_type == EntryType::Symlink || is_symbolic_link(header.mode().ok());
    if claims_symbolic_link {
        validate_symbolic_link_target(entry, archive_path, entry_name)?;
        return Err(ArchiveError::UnsupportedEntry {
            archive: archive_path.to_path_buf(),
            entry: entry_name.to_string(),
            kind: "symbolic link",
        });
    }
    if matches!(entry_type, EntryType::Regular | EntryType::Directory) {
        return Ok(());
    }
    Err(ArchiveError::UnsupportedEntry {
        archive: archive_path.to_path_buf(),
        entry: entry_name.to_string(),
        kind: entry_type_name(entry_type),
    })
}

/// 供错误消息使用的条目类型名称。
///
/// 不含 GNU longname/longlink 与 PAX 扩展头：tar 在 `next_entry` 内部就已消费
/// 并合并了这些头，它们永远不会作为独立条目出现在迭代结果中。
fn entry_type_name(entry_type: EntryType) -> &'static str {
    match entry_type {
        EntryType::Link => "hard link",
        EntryType::Symlink => "symbolic link",
        EntryType::Char => "character device",
        EntryType::Block => "block device",
        EntryType::Fifo => "fifo",
        EntryType::Continuous => "continuous file",
        EntryType::GNUSparse => "sparse file",
        _ => "unrecognized",
    }
}

/// 校验符号链接目标，用于在拒绝条目前给出精确原因。
///
/// tar 把链接目标放在 header 的 linkname 字段，无需读取条目内容。
fn validate_symbolic_link_target<R: Read>(
    entry: &Entry<'_, R>,
    archive_path: &Path,
    entry_name: &str,
) -> Result<(), ArchiveError> {
    let Some(target) = entry.link_name_bytes() else {
        return Err(ArchiveError::InvalidSymbolicLinkTargetEntry {
            archive: archive_path.to_path_buf(),
            entry: entry_name.to_string(),
            reason: "symbolic-link entry has no target",
        });
    };
    if target.len() > MAX_SYMBOLIC_LINK_TARGET_BYTES {
        return Err(ArchiveError::InvalidSymbolicLinkTargetEntry {
            archive: archive_path.to_path_buf(),
            entry: entry_name.to_string(),
            reason: "target exceeds the 4096-byte limit",
        });
    }
    match parse_symbolic_link_target(&target) {
        Ok(_) => Ok(()),
        Err(ArchiveError::InvalidSymbolicLinkTarget { reason }) => {
            Err(ArchiveError::InvalidSymbolicLinkTargetEntry {
                archive: archive_path.to_path_buf(),
                entry: entry_name.to_string(),
                reason,
            })
        }
        Err(error) => Err(error),
    }
}

/// 流式拷贝条目内容，按实际读取的字节数复核单条目上限。
///
/// header 声明的大小不可信，因此完全依据实际读取量累加。总字节与压缩比由
/// [`StreamLimits`] 在解码器层统一约束，这里只补充 tar 层面才有的单条目上限，
/// 并在每块之后重置段计数，使大文件不受 [`MAX_SEGMENT_BYTES`] 影响。
fn copy_entry_with_limits<R: Read>(
    entry: &mut Entry<'_, R>,
    output: &mut cap_std::fs::File,
    output_path: &Path,
    archive_path: &Path,
    total_bytes: &mut u64,
    stream_limits: &StreamLimits,
    limits: ExtractionLimits,
) -> Result<(), ArchiveError> {
    let mut buffer = [0_u8; 64 * 1024];
    let mut entry_bytes = 0_u64;
    loop {
        let count = entry
            .read(&mut buffer)
            .map_err(|error| stream_error(stream_limits, "read entry from", archive_path, error))?;
        if count == 0 {
            return Ok(());
        }
        stream_limits.start_segment();
        entry_bytes = accumulate_bytes(
            entry_bytes,
            count as u64,
            archive_path,
            "per-entry uncompressed bytes",
            limits.max_entry_bytes,
        )?;
        check_limit(
            archive_path,
            "per-entry uncompressed bytes",
            entry_bytes,
            limits.max_entry_bytes,
        )?;
        *total_bytes = accumulate_bytes(
            *total_bytes,
            count as u64,
            archive_path,
            "total uncompressed bytes",
            limits.max_total_bytes,
        )?;
        output
            .write_all(&buffer[..count])
            .map_err(|error| ArchiveError::io("write tar.gz entry file", output_path, error))?;
    }
}

#[cfg(test)]
mod tests {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::{Builder, Header};

    use super::*;

    /// 用给定条目构造一个 tar.gz 文件，条目为 `(名称, 类型, 内容)`。
    ///
    /// 条目名直接写入 GNU header 的 name 字段，而不是走 `Builder::append_data`：
    /// 后者会用 tar 自己的路径校验拒绝 `../` 这类名称，而测试恰恰需要构造
    /// 这种畸形归档来验证解压侧的拒绝逻辑。
    fn write_archive(path: &Path, entries: &[(&str, EntryType, &[u8])]) {
        let file = File::create(path).unwrap();
        let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
        for (name, entry_type, contents) in entries {
            let mut header = Header::new_gnu();
            header.set_entry_type(*entry_type);
            header.set_size(contents.len() as u64);
            header.set_mode(if *entry_type == EntryType::Directory {
                0o755
            } else {
                0o644
            });
            header.set_mtime(0);
            if *entry_type == EntryType::Symlink {
                header.set_size(0);
                header.set_link_name("../outside").unwrap();
            }
            let name_bytes = name.as_bytes();
            assert!(name_bytes.len() <= 100, "test entry name must fit the ustar name field");
            header.as_gnu_mut().unwrap().name[..name_bytes.len()].copy_from_slice(name_bytes);
            header.set_cksum();
            let payload = if *entry_type == EntryType::Symlink {
                &[][..]
            } else {
                contents
            };
            builder.append(&header, payload).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap();
    }

    /// 构造一个声明了超大 size 的 GNU 扩展头条目，用于验证扩展头限流。
    ///
    /// `tar` 在 `next_entry` 内部会把这类条目的内容整体读进内存，因此归档不会
    /// 走到本模块的条目循环——拦截必须发生在解码器层。
    fn write_oversized_extension_header(path: &Path, declared_size: u64) {
        let file = File::create(path).unwrap();
        let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::GNULongName);
        header.set_size(declared_size);
        header.set_mode(0o644);
        header.set_mtime(0);
        let name = b"././@LongLink";
        header.as_gnu_mut().unwrap().name[..name.len()].copy_from_slice(name);
        header.set_cksum();
        // 高度可压缩的载荷：归档文件本身很小，解压后却超过段上限。
        let payload = vec![b'a'; declared_size as usize];
        builder.append(&header, &payload[..]).unwrap();
        builder.into_inner().unwrap().finish().unwrap();
    }

    #[test]
    fn rejects_oversized_extension_header_before_reading_entries() {
        let root = crate::fs::test_dir("tar-extension-header");
        let archive = root.join("bomb.tar.gz");
        let destination = root.join("destination");
        write_oversized_extension_header(&archive, MAX_SEGMENT_BYTES * 2);

        // 放开压缩比与总字节上限，隔离出段上限这一条约束：本测试要证明的是
        // 扩展头的一次性读取被拦下，而不是恰好被其他上限挡住。
        let limits = ExtractionLimits {
            max_compression_ratio: u64::MAX,
            max_total_bytes: u64::MAX,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_tar_gz_with_limits(&archive, &destination, limits),
            Err(ArchiveError::LimitExceeded { limit: "uncompressed segment bytes", .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn extracts_paths_that_require_a_gnu_long_name_header() {
        let root = crate::fs::test_dir("tar-long-name");
        let archive = root.join("long.tar.gz");
        let destination = root.join("destination");
        // 超过 ustar name 字段的 100 字节，tar 会自动写出 GNU longname 扩展头。
        let long_name = format!(
            "datapacks/{}/data/{}/functions/tick.mcfunction",
            "a".repeat(60),
            "b".repeat(40)
        );
        assert!(long_name.len() > 100);
        let file = File::create(&archive).unwrap();
        let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::Regular);
        header.set_size(4);
        header.set_mode(0o644);
        header.set_mtime(0);
        builder
            .append_data(&mut header, &long_name, &b"say\n"[..])
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap();

        let summary = extract_tar_gz(&archive, &destination).unwrap();

        assert_eq!(summary.files, 1);
        assert_eq!(std::fs::read(destination.join(&long_name)).unwrap(), b"say\n");

        std::fs::remove_dir_all(root).unwrap();
    }

    /// 写出一个正常归档，并在 tar 结束标记之后追加原始字节。
    ///
    /// 追加内容位于同一个 gzip 成员内：`Builder::into_inner` 已写出结束块，
    /// 此时 `GzEncoder` 仍未 finish，继续写入即落在结束标记之后。
    fn write_archive_with_trailing_bytes(path: &Path, trailing: &[u8]) {
        let file = File::create(path).unwrap();
        let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::Regular);
        header.set_size(16);
        header.set_mode(0o644);
        header.set_mtime(0);
        builder
            .append_data(&mut header, "server.properties", &b"motd=Sea Lantern"[..])
            .unwrap();
        let mut encoder = builder.into_inner().unwrap();
        encoder.write_all(trailing).unwrap();
        encoder.finish().unwrap();
    }

    #[test]
    fn rejects_data_after_the_end_of_archive_marker() {
        let root = crate::fs::test_dir("tar-trailing-data");
        let archive = root.join("appended.tar.gz");
        let destination = root.join("destination");
        // 追加一个看似合法的 tar 块：gzip 的 CRC 覆盖它，因此校验和无法发现。
        let mut trailing = vec![0_u8; 512];
        trailing[0] = b'x';
        write_archive_with_trailing_bytes(&archive, &trailing);

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_a_symbolic_link_named_after_normalization() {
        let root = crate::fs::test_dir("tar-dot-symlink");
        let archive = root.join("link.tar.gz");
        let destination = root.join("destination");
        // 名称 `./` 规范化后没有输出路径，但类型是符号链接：必须被拒绝，
        // 而不是因无路径可写被静默跳过。具体错误取决于链接目标是否合法
        // （合法目标归 UnsupportedEntry，非法目标归 InvalidSymbolicLinkTargetEntry），
        // 关键是不能成功。
        write_archive(&archive, &[("./", EntryType::Symlink, b"")]);

        assert!(extract_tar_gz(&archive, &destination).is_err());
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn treats_a_trailing_slash_regular_entry_as_a_directory() {
        let root = crate::fs::test_dir("tar-trailing-slash");
        let archive = root.join("slash.tar.gz");
        let destination = root.join("destination");
        // 部分打包器把空目录写成 Regular 类型但带尾部斜杠：应与 ZIP 侧
        // `entry.is_dir()` 的行为一致，按目录处理而非落成同名文件。
        write_archive(
            &archive,
            &[
                ("config/", EntryType::Regular, b""),
                ("config/server.properties", EntryType::Regular, b"motd=Sea Lantern"),
            ],
        );

        let summary = extract_tar_gz(&archive, &destination).unwrap();

        assert_eq!(summary.directories, 1);
        assert_eq!(summary.files, 1);
        assert!(!destination.join("config").is_file());
        assert_eq!(
            std::fs::read(destination.join("config/server.properties")).unwrap(),
            b"motd=Sea Lantern"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_record_alignment_padding_after_the_marker() {
        let root = crate::fs::test_dir("tar-trailing-padding");
        let archive = root.join("padded.tar.gz");
        let destination = root.join("destination");
        // GNU tar 默认补齐到 10240 字节的记录边界，全零填充必须被接受。
        write_archive_with_trailing_bytes(&archive, &vec![0_u8; 10240]);

        let summary = extract_tar_gz(&archive, &destination).unwrap();

        assert_eq!(summary.files, 1);
        assert_eq!(
            std::fs::read(destination.join("server.properties")).unwrap(),
            b"motd=Sea Lantern"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_excessive_padding_after_the_marker() {
        let root = crate::fs::test_dir("tar-trailing-flood");
        let archive = root.join("flooded.tar.gz");
        let destination = root.join("destination");
        write_archive_with_trailing_bytes(
            &archive,
            &vec![0_u8; (MAX_TRAILING_PADDING_BYTES + 1) as usize],
        );

        // 放开压缩比：成片零字节压缩率极高，默认比率会先于填充上限触发，
        // 掩盖本测试要验证的约束。
        let limits = ExtractionLimits {
            max_compression_ratio: u64::MAX,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_tar_gz_with_limits(&archive, &destination, limits),
            Err(ArchiveError::LimitExceeded { limit: "trailing padding bytes", .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_a_file_entry_that_blocks_a_parent_directory() {
        let root = crate::fs::test_dir("tar-path-conflict");
        let archive = root.join("conflict.tar.gz");
        let destination = root.join("destination");
        // `config` 先作为普通文件出现，随后的条目却要用它当父目录。
        write_archive(
            &archive,
            &[
                ("config", EntryType::Regular, b"not a directory"),
                ("config/server.properties", EntryType::Regular, b"motd=Sea Lantern"),
            ],
        );

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_case_insensitive_path_collisions() {
        let root = crate::fs::test_dir("tar-case-collision");
        let archive = root.join("collision.tar.gz");
        let destination = root.join("destination");
        // Windows/macOS 上两个条目会落在同一文件。
        write_archive(
            &archive,
            &[
                ("Server.properties", EntryType::Regular, b"first"),
                ("server.properties", EntryType::Regular, b"second"),
            ],
        );

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_entry_paths_above_the_length_limit() {
        let root = crate::fs::test_dir("tar-path-length");
        let archive = root.join("long-name.tar.gz");
        let destination = root.join("destination");
        // 超过 ustar name 字段，tar 会写出 GNU longname 扩展头；长度校验必须
        // 认这个合并后的完整路径，而不是 header 里被截断的那 100 字节。
        let long_name = format!("config/{}.properties", "a".repeat(200));
        let file = File::create(&archive).unwrap();
        let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::Regular);
        header.set_size(7);
        header.set_mode(0o644);
        header.set_mtime(0);
        builder
            .append_data(&mut header, &long_name, &b"payload"[..])
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap();

        let limits = ExtractionLimits {
            max_entry_path_bytes: 64,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_tar_gz_with_limits(&archive, &destination, limits),
            Err(ArchiveError::LimitExceeded {
                limit: "entry path bytes",
                maximum: 64,
                ..
            })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn extracts_files_and_directories() {
        let root = crate::fs::test_dir("tar-extract");
        let archive = root.join("server.tar.gz");
        let destination = root.join("destination");
        write_archive(
            &archive,
            &[
                ("config/", EntryType::Directory, b""),
                ("config/server.properties", EntryType::Regular, b"motd=Sea Lantern"),
            ],
        );

        let summary = extract_tar_gz(&archive, &destination).unwrap();

        assert_eq!(summary.files, 1);
        assert_eq!(summary.directories, 1);
        assert_eq!(summary.bytes, 16);
        assert_eq!(
            std::fs::read(destination.join("config/server.properties")).unwrap(),
            b"motd=Sea Lantern"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_current_directory_prefixed_entry_names() {
        let root = crate::fs::test_dir("tar-dot-prefix");
        let archive = root.join("dotted.tar.gz");
        let destination = root.join("destination");
        write_archive(
            &archive,
            &[
                ("./", EntryType::Directory, b""),
                ("./config/", EntryType::Directory, b""),
                ("./config/server.properties", EntryType::Regular, b"motd=Sea Lantern"),
            ],
        );

        let summary = extract_tar_gz(&archive, &destination).unwrap();

        // `./` 指代解压根目录本身，不计入统计。
        assert_eq!(summary.directories, 1);
        assert_eq!(summary.files, 1);
        assert!(destination.join("config/server.properties").is_file());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_path_traversal_without_leaving_a_destination() {
        let root = crate::fs::test_dir("tar-traversal");
        let archive = root.join("unsafe.tar.gz");
        let destination = root.join("destination");
        write_archive(&archive, &[("../outside.txt", EntryType::Regular, b"unsafe")]);

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(!root.join("outside.txt").exists());
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_symbolic_link_entries_without_leaving_a_destination() {
        let root = crate::fs::test_dir("tar-symlink");
        let archive = root.join("link.tar.gz");
        let destination = root.join("destination");
        write_archive(&archive, &[("config", EntryType::Symlink, b"")]);

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::InvalidSymbolicLinkTargetEntry { .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_duplicate_entries() {
        let root = crate::fs::test_dir("tar-duplicate");
        let archive = root.join("duplicate.tar.gz");
        let destination = root.join("destination");
        write_archive(
            &archive,
            &[
                ("server.properties", EntryType::Regular, b"first"),
                ("server.properties", EntryType::Regular, b"second"),
            ],
        );

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enforces_per_entry_limit_without_leaving_a_destination() {
        let root = crate::fs::test_dir("tar-limits");
        let archive = root.join("large.tar.gz");
        let destination = root.join("destination");
        write_archive(&archive, &[("payload.bin", EntryType::Regular, &[0; 32])]);

        let limits = ExtractionLimits {
            max_entry_bytes: 16,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_tar_gz_with_limits(&archive, &destination, limits),
            Err(ArchiveError::LimitExceeded {
                limit: "per-entry uncompressed bytes",
                ..
            })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enforces_overall_compression_ratio() {
        let root = crate::fs::test_dir("tar-ratio");
        let archive = root.join("bomb.tar.gz");
        let destination = root.join("destination");
        // 负载须超过 min_ratio_enforcement_bytes，比率判定才会生效；零字节的
        // 压缩率极高，因此实际归档文件很小，整体压缩比远超上限。
        let threshold = ExtractionLimits::default().min_ratio_enforcement_bytes;
        let payload = vec![0_u8; (threshold + 512 * 1024) as usize];
        write_archive(&archive, &[("payload.bin", EntryType::Regular, &payload)]);

        let limits = ExtractionLimits {
            max_compression_ratio: 1,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_tar_gz_with_limits(&archive, &destination, limits),
            Err(ArchiveError::LimitExceeded { limit: "compression ratio", .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enforces_the_ratio_just_above_the_minimum_threshold() {
        let root = crate::fs::test_dir("tar-ratio-threshold");
        let archive = root.join("bomb.tar.gz");
        let destination = root.join("destination");
        // 负载恰好略高于阈值，压缩比上限必须生效。
        let threshold = ExtractionLimits::default().min_ratio_enforcement_bytes;
        let payload = vec![0_u8; (threshold + 1) as usize];
        write_archive(&archive, &[("payload.bin", EntryType::Regular, &payload)]);

        let limits = ExtractionLimits {
            max_compression_ratio: 1,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_tar_gz_with_limits(&archive, &destination, limits),
            Err(ArchiveError::LimitExceeded { limit: "compression ratio", .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn does_not_enforce_the_ratio_on_small_archives() {
        let root = crate::fs::test_dir("tar-ratio-small");
        let archive = root.join("small.tar.gz");
        let destination = root.join("destination");
        // 小归档的固定开销（结束标记与记录对齐填充）本身就是高压缩比的零字节，
        // 不应因此被拒。
        write_archive(&archive, &[("payload.bin", EntryType::Regular, &[0; 4096])]);

        let limits = ExtractionLimits {
            max_compression_ratio: 1,
            ..ExtractionLimits::default()
        };
        let summary = extract_tar_gz_with_limits(&archive, &destination, limits).unwrap();
        assert_eq!(summary.files, 1);
        assert_eq!(summary.bytes, 4096);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_overwrite_existing_destination() {
        let root = crate::fs::test_dir("tar-existing-destination");
        let archive = root.join("archive.tar.gz");
        let destination = root.join("destination");
        write_archive(&archive, &[("server.properties", EntryType::Regular, b"from archive")]);
        std::fs::create_dir_all(&destination).unwrap();
        std::fs::write(destination.join("server.properties"), b"existing").unwrap();

        assert!(matches!(
            extract_tar_gz(&archive, &destination),
            Err(ArchiveError::DestinationExists { .. })
        ));
        assert_eq!(std::fs::read(destination.join("server.properties")).unwrap(), b"existing");

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn leaves_no_temporary_directory_behind() {
        let root = crate::fs::test_dir("tar-temporary");
        let archive = root.join("server.tar.gz");
        let destination = root.join("destination");
        write_archive(&archive, &[("server.properties", EntryType::Regular, b"motd=Sea Lantern")]);

        extract_tar_gz(&archive, &destination).unwrap();

        let leftovers: Vec<_> = std::fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "unexpected leftovers: {leftovers:?}");

        std::fs::remove_dir_all(root).unwrap();
    }
}
