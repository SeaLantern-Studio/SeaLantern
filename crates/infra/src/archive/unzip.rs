use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use zip::ZipArchive;

use crate::fs::SafeRelativePath;

use super::{is_symbolic_link, parse_symbolic_link_target, ArchiveError};

const MAX_SYMBOLIC_LINK_TARGET_BYTES: u64 = 4 * 1024;

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

/// Extracts a ZIP archive into destination.
///
/// Every entry is parsed as a portable SafeRelativePath. ZIP entries that
/// represent symbolic links are rejected after their targets are validated,
/// because creating links requires an explicit platform-specific policy.
pub fn extract_zip(
    archive: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ExtractionSummary, ArchiveError> {
    let archive = archive.as_ref();
    let destination = destination.as_ref();
    let result = extract_zip_inner(archive, destination);
    if let Err(error) = &result {
        crate::observability::archive_operation_failed("extract ZIP", archive, error);
    }
    result
}

fn extract_zip_inner(
    archive_path: &Path,
    destination: &Path,
) -> Result<ExtractionSummary, ArchiveError> {
    fs::create_dir_all(destination)
        .map_err(|error| ArchiveError::io("create ZIP extraction directory", destination, error))?;
    let root = Dir::open_ambient_dir(destination, ambient_authority())
        .map_err(|error| ArchiveError::io("open ZIP extraction directory", destination, error))?;
    let file = File::open(archive_path)
        .map_err(|error| ArchiveError::io("open ZIP archive", archive_path, error))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| ArchiveError::zip("read", archive_path, error))?;
    let mut summary = ExtractionSummary::default();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| ArchiveError::zip("read entry from", archive_path, error))?;
        let entry_name = entry.name().to_owned();
        let relative =
            SafeRelativePath::parse(&entry_name).map_err(|error| ArchiveError::UnsafeEntry {
                archive: archive_path.to_path_buf(),
                entry: entry_name.clone(),
                reason: error.to_string(),
            })?;

        if is_symbolic_link(entry.unix_mode()) {
            validate_symbolic_link_target(&mut entry)?;
            return Err(ArchiveError::UnsupportedEntry {
                archive: archive_path.to_path_buf(),
                entry: entry_name,
                kind: "symbolic link",
            });
        }
        if entry.is_dir() {
            root.create_dir_all(relative.as_path()).map_err(|error| {
                ArchiveError::io(
                    "create ZIP entry directory",
                    destination.join(relative.as_path()),
                    error,
                )
            })?;
            summary.directories += 1;
            continue;
        }

        if let Some(parent) = relative.as_path().parent() {
            root.create_dir_all(parent).map_err(|error| {
                ArchiveError::io("create ZIP entry parent", destination.join(parent), error)
            })?;
        }
        let output_path = destination.join(relative.as_path());
        let mut output = root
            .create(relative.as_path())
            .map_err(|error| ArchiveError::io("create ZIP entry file", &output_path, error))?;
        let copied = io::copy(&mut entry, &mut output)
            .map_err(|error| ArchiveError::io("extract ZIP entry file", &output_path, error))?;
        summary.files += 1;
        summary.bytes += copied;
    }

    Ok(summary)
}

fn validate_symbolic_link_target(entry: &mut zip::read::ZipFile<'_>) -> Result<(), ArchiveError> {
    let mut target = Vec::new();
    entry
        .take(MAX_SYMBOLIC_LINK_TARGET_BYTES + 1)
        .read_to_end(&mut target)
        .map_err(|error| ArchiveError::InvalidSymbolicLinkTarget {
            reason: if error.kind() == io::ErrorKind::InvalidData {
                "target could not be decoded from ZIP"
            } else {
                "target could not be read from ZIP"
            },
        })?;
    if target.len() as u64 > MAX_SYMBOLIC_LINK_TARGET_BYTES {
        return Err(ArchiveError::InvalidSymbolicLinkTarget {
            reason: "target exceeds the 4096-byte limit",
        });
    }
    parse_symbolic_link_target(&target).map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use super::*;

    #[test]
    fn rejects_path_traversal_before_writing() {
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

        fs::remove_dir_all(root).unwrap();
    }
}
