use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::{open_existing_directory, parent_path, ArchiveError};

/// Counts entries and uncompressed bytes processed during ZIP creation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArchiveSummary {
    /// Number of regular files written.
    pub files: u64,
    /// Number of directory entries written.
    pub directories: u64,
    /// Total uncompressed file bytes written.
    pub bytes: u64,
}

/// Creates a ZIP archive containing the contents of source.
///
/// The destination must not already exist. A temporary archive is completed in
/// the destination directory and then hard-linked into place, so source-read
/// or write failures never replace a previous archive with a partial result.
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
            if let Err(error) = fs::hard_link(&temporary, destination) {
                let publish_error =
                    ArchiveError::io("publish completed ZIP archive", destination, error);
                remove_temporary_archive(&temporary);
                return Err(publish_error);
            }
            remove_temporary_archive(&temporary);
            Ok(summary)
        }
        Err(error) => {
            remove_temporary_archive(&temporary);
            Err(error)
        }
    }
}

fn remove_temporary_archive(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != io::ErrorKind::NotFound {
            crate::observability::archive_cleanup_failed(path, &error);
        }
    }
}

fn reject_existing_destination(destination: &Path) -> Result<(), ArchiveError> {
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            return Err(ArchiveError::DestinationExists { path: destination.to_path_buf() });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ArchiveError::io("read ZIP destination metadata", destination, error));
        }
    }
    let parent = parent_path(destination);
    fs::create_dir_all(parent)
        .map_err(|error| ArchiveError::io("create ZIP destination parent", parent, error))?;
    Ok(())
}

fn write_archive(
    source_root: &Dir,
    source_path: &Path,
    temporary: &Path,
) -> Result<ArchiveSummary, ArchiveError> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(|error| ArchiveError::io("create temporary ZIP archive", temporary, error))?;
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
                directory.as_path()
            })
            .map_err(|error| {
                ArchiveError::io("read ZIP source directory", source_path.join(&directory), error)
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                ArchiveError::io(
                    "iterate ZIP source directory",
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
                ArchiveError::io("read ZIP source entry metadata", &display_path, error)
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
            let mut input = source_root
                .open(&relative)
                .map_err(|error| ArchiveError::io("open ZIP source file", &display_path, error))?;
            let copied = io::copy(&mut input, &mut writer)
                .map_err(|error| ArchiveError::io("write ZIP file entry", &display_path, error))?;
            summary.files += 1;
            summary.bytes =
                summary
                    .bytes
                    .checked_add(copied)
                    .ok_or_else(|| ArchiveError::LimitExceeded {
                        archive: temporary.to_path_buf(),
                        limit: "archive source bytes",
                        observed: u64::MAX,
                        maximum: u64::MAX - 1,
                    })?;
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
