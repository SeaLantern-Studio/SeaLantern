use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use cap_std::fs::{Dir, OpenOptions};
use zip::ZipArchive;

use crate::fs::SafeRelativePath;

use super::{create_new_directory, is_symbolic_link, parse_symbolic_link_target, ArchiveError};

const MAX_SYMBOLIC_LINK_TARGET_BYTES: u64 = 4 * 1024;

/// Resource limits applied before and during ZIP extraction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtractionLimits {
    /// Maximum compressed ZIP file size accepted from disk.
    pub max_archive_bytes: u64,
    /// Maximum number of entries in the ZIP central directory.
    pub max_entries: usize,
    /// Maximum uncompressed bytes for one regular file.
    pub max_entry_bytes: u64,
    /// Maximum total uncompressed bytes written across all entries.
    pub max_total_bytes: u64,
    /// Maximum accepted ratio of uncompressed to compressed bytes.
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

/// Counts entries and uncompressed bytes processed during ZIP extraction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExtractionSummary {
    /// Number of regular files extracted.
    pub files: u64,
    /// Number of directory entries extracted.
    pub directories: u64,
    /// Total uncompressed file bytes extracted.
    pub bytes: u64,
}

/// Extracts a ZIP archive into a new destination directory with default limits.
///
/// The destination must not already exist. This avoids overwriting a previous
/// extraction if an archive is invalid or a later I/O operation fails.
pub fn extract_zip(
    archive: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ExtractionSummary, ArchiveError> {
    extract_zip_with_limits(archive, destination, ExtractionLimits::default())
}

/// Extracts a ZIP archive into a new destination directory with explicit limits.
///
/// All entry names, duplicate paths, symbolic links, and metadata limits are
/// validated before the destination directory is created. The destination is
/// opened without following its final path component, and every output file is
/// created exclusively.
pub fn extract_zip_with_limits(
    archive: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    limits: ExtractionLimits,
) -> Result<ExtractionSummary, ArchiveError> {
    let archive = archive.as_ref();
    let destination = destination.as_ref();
    let result = extract_zip_inner(archive, destination, limits);
    if let Err(error) = &result {
        crate::observability::archive_operation_failed_with_context(
            "extract ZIP",
            archive,
            Some(destination),
            error.entry(),
            error,
        );
    }
    result
}

fn extract_zip_inner(
    archive_path: &Path,
    destination: &Path,
    limits: ExtractionLimits,
) -> Result<ExtractionSummary, ArchiveError> {
    let archive_size = std::fs::metadata(archive_path)
        .map_err(|error| ArchiveError::io("read ZIP archive metadata", archive_path, error))?
        .len();
    check_limit(archive_path, "compressed archive bytes", archive_size, limits.max_archive_bytes)?;

    let file = File::open(archive_path)
        .map_err(|error| ArchiveError::io("open ZIP archive", archive_path, error))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| ArchiveError::zip("read", archive_path, error))?;
    validate_archive(&mut archive, archive_path, limits)?;

    let root = create_new_directory(destination)?;
    let mut summary = ExtractionSummary::default();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| ArchiveError::zip("read entry from", archive_path, error))?;
        let entry_name = entry.name().to_owned();
        let relative = safe_entry_path(archive_path, &entry_name)?;

        if entry.is_dir() {
            ensure_directory(&root, relative.as_path(), destination)?;
            summary.directories += 1;
            continue;
        }

        ensure_parent_dirs(&root, relative.as_path(), destination)?;
        let output_path = destination.join(relative.as_path());
        let mut output = root
            .open_with(relative.as_path(), OpenOptions::new().write(true).create_new(true))
            .map_err(|error| ArchiveError::io("create ZIP entry file", &output_path, error))?;
        let copied = copy_entry_with_limits(
            &mut entry,
            &mut output,
            archive_path,
            &entry_name,
            &mut summary.bytes,
            limits,
        )?;
        summary.files += 1;
        debug_assert_eq!(copied, entry.size());
    }

    Ok(summary)
}

fn validate_archive(
    archive: &mut ZipArchive<File>,
    archive_path: &Path,
    limits: ExtractionLimits,
) -> Result<(), ArchiveError> {
    check_limit(archive_path, "entry count", archive.len() as u64, limits.max_entries as u64)?;

    let mut paths = HashSet::with_capacity(archive.len());
    let mut total_bytes = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| ArchiveError::zip("read entry from", archive_path, error))?;
        let entry_name = entry.name().to_owned();
        let relative = safe_entry_path(archive_path, &entry_name)?;
        if !paths.insert(relative.clone()) {
            return Err(ArchiveError::UnsafeEntry {
                archive: archive_path.to_path_buf(),
                entry: entry_name,
                reason: "archive contains duplicate output paths".to_string(),
            });
        }
        if is_symbolic_link(entry.unix_mode()) {
            validate_symbolic_link_target(&mut entry, archive_path, &entry_name)?;
            return Err(ArchiveError::UnsupportedEntry {
                archive: archive_path.to_path_buf(),
                entry: entry_name,
                kind: "symbolic link",
            });
        }
        if entry.is_dir() {
            continue;
        }

        let entry_size = entry.size();
        check_limit(
            archive_path,
            "per-entry uncompressed bytes",
            entry_size,
            limits.max_entry_bytes,
        )?;
        total_bytes =
            total_bytes
                .checked_add(entry_size)
                .ok_or_else(|| ArchiveError::LimitExceeded {
                    archive: archive_path.to_path_buf(),
                    limit: "total uncompressed bytes",
                    observed: u64::MAX,
                    maximum: limits.max_total_bytes,
                })?;
        check_limit(archive_path, "total uncompressed bytes", total_bytes, limits.max_total_bytes)?;
        let compressed_size = entry.compressed_size();
        if entry_size > 0
            && (compressed_size == 0
                || entry_size > compressed_size.saturating_mul(limits.max_compression_ratio))
        {
            return Err(ArchiveError::LimitExceeded {
                archive: archive_path.to_path_buf(),
                limit: "compression ratio",
                observed: entry_size,
                maximum: compressed_size.saturating_mul(limits.max_compression_ratio),
            });
        }
    }
    Ok(())
}

fn safe_entry_path(
    archive_path: &Path,
    entry_name: &str,
) -> Result<SafeRelativePath, ArchiveError> {
    SafeRelativePath::parse(entry_name).map_err(|error| ArchiveError::UnsafeEntry {
        archive: archive_path.to_path_buf(),
        entry: entry_name.to_string(),
        reason: error.to_string(),
    })
}

fn ensure_parent_dirs(root: &Dir, path: &Path, destination: &Path) -> Result<(), ArchiveError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let mut current = PathBuf::new();
    for component in parent.components() {
        current.push(component);
        match root.open_dir(&current) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                root.create_dir(&current).map_err(|error| {
                    ArchiveError::io(
                        "create ZIP entry parent directory",
                        destination.join(&current),
                        error,
                    )
                })?;
                root.open_dir(&current).map_err(|error| {
                    ArchiveError::io(
                        "open ZIP entry parent directory",
                        destination.join(&current),
                        error,
                    )
                })?;
            }
            Err(error) => {
                return Err(ArchiveError::io(
                    "open ZIP entry parent directory",
                    destination.join(&current),
                    error,
                ));
            }
        }
    }
    Ok(())
}

fn ensure_directory(root: &Dir, path: &Path, destination: &Path) -> Result<(), ArchiveError> {
    ensure_parent_dirs(root, path, destination)?;
    match root.create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            root.open_dir(path).map(|_| ()).map_err(|error| {
                ArchiveError::io("open ZIP entry directory", destination.join(path), error)
            })
        }
        Err(error) => {
            Err(ArchiveError::io("create ZIP entry directory", destination.join(path), error))
        }
    }
}

fn copy_entry_with_limits(
    entry: &mut zip::read::ZipFile<'_>,
    output: &mut cap_std::fs::File,
    archive_path: &Path,
    entry_name: &str,
    total_bytes: &mut u64,
    limits: ExtractionLimits,
) -> Result<u64, ArchiveError> {
    let mut buffer = [0_u8; 64 * 1024];
    let mut entry_bytes = 0_u64;
    loop {
        let count = entry.read(&mut buffer).map_err(|error| ArchiveError::Io {
            operation: "read ZIP entry",
            path: archive_path.to_path_buf(),
            source: error,
        })?;
        if count == 0 {
            return Ok(entry_bytes);
        }
        entry_bytes =
            entry_bytes
                .checked_add(count as u64)
                .ok_or_else(|| ArchiveError::LimitExceeded {
                    archive: archive_path.to_path_buf(),
                    limit: "per-entry uncompressed bytes",
                    observed: u64::MAX,
                    maximum: limits.max_entry_bytes,
                })?;
        check_limit(
            archive_path,
            "per-entry uncompressed bytes",
            entry_bytes,
            limits.max_entry_bytes,
        )?;
        *total_bytes =
            total_bytes
                .checked_add(count as u64)
                .ok_or_else(|| ArchiveError::LimitExceeded {
                    archive: archive_path.to_path_buf(),
                    limit: "total uncompressed bytes",
                    observed: u64::MAX,
                    maximum: limits.max_total_bytes,
                })?;
        check_limit(
            archive_path,
            "total uncompressed bytes",
            *total_bytes,
            limits.max_total_bytes,
        )?;
        output
            .write_all(&buffer[..count])
            .map_err(|error| ArchiveError::Io {
                operation: "write ZIP entry file",
                path: PathBuf::from(entry_name),
                source: error,
            })?;
    }
}

fn validate_symbolic_link_target(
    entry: &mut zip::read::ZipFile<'_>,
    archive_path: &Path,
    entry_name: &str,
) -> Result<(), ArchiveError> {
    let mut target = Vec::new();
    entry
        .take(MAX_SYMBOLIC_LINK_TARGET_BYTES + 1)
        .read_to_end(&mut target)
        .map_err(|source| ArchiveError::SymbolicLinkTargetRead {
            archive: archive_path.to_path_buf(),
            entry: entry_name.to_string(),
            source,
        })?;
    if target.len() as u64 > MAX_SYMBOLIC_LINK_TARGET_BYTES {
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

fn check_limit(
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use super::*;

    #[test]
    fn rejects_path_traversal_before_creating_destination() {
        let root = crate::fs::test_dir("unzip");
        let archive_path = root.join("unsafe.zip");
        let destination = root.join("destination");
        let file = File::create(&archive_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("../outside.txt", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"unsafe").unwrap();
        writer.finish().unwrap();

        assert!(matches!(
            extract_zip(&archive_path, &destination),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(!root.join("outside.txt").exists());
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_to_overwrite_existing_destination() {
        let root = crate::fs::test_dir("existing-destination");
        let archive_path = root.join("archive.zip");
        let destination = root.join("destination");
        let file = File::create(&archive_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("server.properties", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"from archive").unwrap();
        writer.finish().unwrap();
        std::fs::create_dir_all(&destination).unwrap();
        std::fs::write(destination.join("server.properties"), b"existing").unwrap();

        assert!(matches!(
            extract_zip(&archive_path, &destination),
            Err(ArchiveError::DestinationExists { .. })
        ));
        assert_eq!(std::fs::read(destination.join("server.properties")).unwrap(), b"existing");

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enforces_declared_entry_limit_before_writing() {
        let root = crate::fs::test_dir("limits");
        let archive_path = root.join("large.zip");
        let destination = root.join("destination");
        let file = File::create(&archive_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("payload.bin", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&[0; 32]).unwrap();
        writer.finish().unwrap();

        let limits = ExtractionLimits {
            max_entry_bytes: 16,
            ..ExtractionLimits::default()
        };
        assert!(matches!(
            extract_zip_with_limits(&archive_path, &destination, limits),
            Err(ArchiveError::LimitExceeded {
                limit: "per-entry uncompressed bytes",
                ..
            })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_explicit_directory_after_an_implicit_parent() {
        let root = crate::fs::test_dir("directory-order");
        let archive_path = root.join("ordered.zip");
        let destination = root.join("destination");
        let file = File::create(&archive_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("config/server.properties", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"motd=Sea Lantern").unwrap();
        writer
            .add_directory("config", SimpleFileOptions::default())
            .unwrap();
        writer.finish().unwrap();

        let summary = extract_zip(&archive_path, &destination).unwrap();
        assert_eq!(summary.files, 1);
        assert_eq!(summary.directories, 1);
        assert_eq!(
            std::fs::read(destination.join("config/server.properties")).unwrap(),
            b"motd=Sea Lantern"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_symbolic_link_entries_before_creating_destination() {
        let root = crate::fs::test_dir("symbolic-link");
        let archive_path = root.join("link.zip");
        let destination = root.join("destination");
        let file = File::create(&archive_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .add_symlink("config", "../outside", SimpleFileOptions::default())
            .unwrap();
        writer.finish().unwrap();

        assert!(matches!(
            extract_zip(&archive_path, &destination),
            Err(ArchiveError::InvalidSymbolicLinkTargetEntry { .. })
        ));
        assert!(!destination.exists());

        std::fs::remove_dir_all(root).unwrap();
    }
}
