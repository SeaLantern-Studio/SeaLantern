use std::path::{Path, PathBuf};

use super::{FsError, SafeRelativePath};

/// Resolves an archive entry under an existing extraction root.
///
/// ZIP and TAR adapters should validate every entry with this function before
/// creating a file, directory, or link. It rejects absolute paths, traversal,
/// and paths that cross an existing symbolic link.
pub fn archive_entry_destination(
    extraction_root: impl AsRef<Path>,
    entry_name: &str,
) -> Result<PathBuf, FsError> {
    SafeRelativePath::parse(entry_name)?.resolve_under(extraction_root)
}

/// Parses a symbolic-link payload from an archive.
///
/// Archive adapters must reject targets that escape the extraction root.
pub fn parse_symbolic_link_target(target: &str) -> Result<SafeRelativePath, FsError> {
    SafeRelativePath::parse(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_archive_traversal() {
        let root = crate::fs::test_dir("archive");
        assert!(archive_entry_destination(&root, "../outside").is_err());
        assert!(parse_symbolic_link_target("/absolute").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
