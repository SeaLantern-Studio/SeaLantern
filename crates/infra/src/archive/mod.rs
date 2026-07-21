//! Portable ZIP archive infrastructure.
//!
//! Archives are written from directory contents and extracted through validated
//! relative paths. Symbolic links are deliberately rejected during extraction:
//! creating them has incompatible permissions and semantics across supported
//! platforms, and callers must opt into a dedicated policy before doing so.

mod error;
mod symbol_link;
mod unzip;
mod zipper;

use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

pub use error::ArchiveError;
pub use symbol_link::{is_symbolic_link, parse_symbolic_link_target};
pub use unzip::{extract_zip, extract_zip_with_limits, ExtractionLimits, ExtractionSummary};
pub use zipper::{create_zip, ArchiveSummary};

fn open_existing_directory(path: &Path, role: &'static str) -> Result<Dir, ArchiveError> {
    let parent_path = parent_path(path);
    let name = path
        .file_name()
        .ok_or_else(|| ArchiveError::InvalidSource {
            path: path.to_path_buf(),
            reason: "source directory must have a final path component",
        })?;
    let parent = Dir::open_ambient_dir(parent_path, ambient_authority())
        .map_err(|error| ArchiveError::io("open ZIP source parent", parent_path, error))?;
    let metadata = parent
        .symlink_metadata(name)
        .map_err(|error| ArchiveError::io("read ZIP source metadata", path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ArchiveError::InvalidSource { path: path.to_path_buf(), reason: role });
    }
    parent
        .open_dir(name)
        .map_err(|error| ArchiveError::io("open ZIP source directory", path, error))
}

fn create_new_directory(path: &Path) -> Result<Dir, ArchiveError> {
    let parent_path = parent_path(path);
    std::fs::create_dir_all(parent_path)
        .map_err(|error| ArchiveError::io("create ZIP destination parent", parent_path, error))?;
    let name = path
        .file_name()
        .ok_or_else(|| ArchiveError::InvalidDestination {
            path: path.to_path_buf(),
            reason: "destination directory must have a final path component",
        })?;
    let parent = Dir::open_ambient_dir(parent_path, ambient_authority())
        .map_err(|error| ArchiveError::io("open ZIP destination parent", parent_path, error))?;
    match parent.create_dir(name) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ArchiveError::DestinationExists { path: path.to_path_buf() });
        }
        Err(error) => return Err(ArchiveError::io("create ZIP extraction directory", path, error)),
    }
    parent
        .open_dir(name)
        .map_err(|error| ArchiveError::io("open ZIP extraction directory", path, error))
}

fn parent_path(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}
