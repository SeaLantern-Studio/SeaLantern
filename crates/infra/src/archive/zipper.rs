use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::ArchiveError;

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
/// Empty directories are retained. Symbolic links and special file types are
/// rejected so the output has consistent behavior when consumed on another
/// supported platform.
pub fn create_zip(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ArchiveSummary, ArchiveError> {
    let source = source.as_ref();
    let destination = destination.as_ref();
    let result = create_zip_inner(source, destination);
    if let Err(error) = &result {
        crate::observability::archive_operation_failed("create ZIP", destination, error);
    }
    result
}

fn create_zip_inner(source: &Path, destination: &Path) -> Result<ArchiveSummary, ArchiveError> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| ArchiveError::io("read ZIP source metadata", source, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ArchiveError::InvalidSource {
            path: source.to_path_buf(),
            reason: "source must be a directory that is not a symbolic link",
        });
    }

    let mut entries = Vec::new();
    collect_entries(source, source, &mut entries)?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| ArchiveError::io("create ZIP destination parent", parent, error))?;
    }
    let file = File::create(destination)
        .map_err(|error| ArchiveError::io("create ZIP archive", destination, error))?;
    let mut writer = ZipWriter::new(file);
    let file_options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let directory_options =
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut summary = ArchiveSummary::default();

    for entry in entries {
        let name = portable_name(&entry.relative)?;
        if entry.is_directory {
            writer
                .add_directory(name, directory_options)
                .map_err(|error| {
                    ArchiveError::zip("write directory entry to", destination, error)
                })?;
            summary.directories += 1;
            continue;
        }

        writer
            .start_file(name, file_options)
            .map_err(|error| ArchiveError::zip("start file entry in", destination, error))?;
        let mut input = File::open(&entry.absolute)
            .map_err(|error| ArchiveError::io("open ZIP source file", &entry.absolute, error))?;
        let copied = io::copy(&mut input, &mut writer)
            .map_err(|error| ArchiveError::io("write ZIP file entry", &entry.absolute, error))?;
        summary.files += 1;
        summary.bytes += copied;
    }

    writer
        .finish()
        .map_err(|error| ArchiveError::zip("finalize", destination, error))?;
    Ok(summary)
}

#[derive(Debug)]
struct SourceEntry {
    absolute: PathBuf,
    relative: PathBuf,
    is_directory: bool,
}

fn collect_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<SourceEntry>,
) -> Result<(), ArchiveError> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| ArchiveError::io("read ZIP source directory", directory, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ArchiveError::io("iterate ZIP source directory", directory, error))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| ArchiveError::io("read ZIP source entry metadata", &path, error))?;
        let relative = path
            .strip_prefix(root)
            .expect("child was read beneath ZIP source")
            .to_path_buf();
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            return Err(ArchiveError::UnsupportedSourceEntry { path, kind: "symbolic link" });
        }
        if file_type.is_dir() {
            entries.push(SourceEntry {
                absolute: path.clone(),
                relative,
                is_directory: true,
            });
            collect_entries(root, &path, entries)?;
        } else if file_type.is_file() {
            entries.push(SourceEntry {
                absolute: path,
                relative,
                is_directory: false,
            });
        } else {
            return Err(ArchiveError::UnsupportedSourceEntry { path, kind: "special" });
        }
    }
    Ok(())
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
}
