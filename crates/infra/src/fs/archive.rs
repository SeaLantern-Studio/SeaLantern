use super::{FsError, SafeRelativePath};

/// Parses a portable archive entry path.
///
/// ZIP and TAR adapters must pass the resulting path to a directory-handle
/// based extractor. Returning a joined path here would reintroduce a symlink
/// replacement race between validation and file creation.
pub fn archive_entry_path(entry_name: &str) -> Result<SafeRelativePath, FsError> {
    SafeRelativePath::parse(entry_name)
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
        assert!(archive_entry_path("../outside").is_err());
        assert!(parse_symbolic_link_target("/absolute").is_err());
    }
}
