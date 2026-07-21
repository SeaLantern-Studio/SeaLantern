use crate::fs::SafeRelativePath;

use super::ArchiveError;

const UNIX_FILE_TYPE_MASK: u32 = 0o170000;
const UNIX_SYMBOLIC_LINK: u32 = 0o120000;

/// Returns whether ZIP Unix attributes identify an entry as a symbolic link.
pub fn is_symbolic_link(unix_mode: Option<u32>) -> bool {
    unix_mode.is_some_and(|mode| mode & UNIX_FILE_TYPE_MASK == UNIX_SYMBOLIC_LINK)
}

/// Parses a ZIP symbolic-link payload as a portable, root-relative target.
///
/// ZIP symlink payloads are raw bytes. They must be UTF-8 and use only normal,
/// forward-slash-separated components before a caller can safely apply a
/// platform-specific link creation policy.
pub fn parse_symbolic_link_target(target: &[u8]) -> Result<SafeRelativePath, ArchiveError> {
    let target = std::str::from_utf8(target).map_err(|_| {
        ArchiveError::InvalidSymbolicLinkTarget { reason: "target is not valid UTF-8" }
    })?;
    SafeRelativePath::parse(target).map_err(|_| ArchiveError::InvalidSymbolicLinkTarget {
        reason: "target must be portable and relative",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_unix_symbolic_link_mode() {
        assert!(is_symbolic_link(Some(0o120777)));
        assert!(!is_symbolic_link(Some(0o100644)));
        assert!(!is_symbolic_link(None));
    }

    #[test]
    fn accepts_only_safe_portable_targets() {
        assert_eq!(
            parse_symbolic_link_target(b"assets/server.jar")
                .unwrap()
                .as_path(),
            std::path::Path::new("assets/server.jar")
        );
        assert!(parse_symbolic_link_target(b"../outside").is_err());
        assert!(parse_symbolic_link_target(b"C:\\outside").is_err());
        assert!(parse_symbolic_link_target(&[0xff]).is_err());
    }
}
