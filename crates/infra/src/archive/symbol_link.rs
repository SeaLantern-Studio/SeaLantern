//! 符号链接的识别与载荷校验。
//!
//! 本模块只做两件事：从 Unix mode 位判定条目是否为符号链接，以及解析
//! 链接载荷为可移植的相对路径。它**不创建链接**——创建符号链接在各平台
//! 的权限与语义互不兼容，解压流程有意拒绝这类条目，实际策略由调用方决定。
//!
//! 两个函数都只在归档适配器内部使用：`tar_read` 与 `zip_read` 依赖它们
//! 在落盘前拦截链接条目。

use crate::fs::SafeRelativePath;

use super::ArchiveError;

const UNIX_FILE_TYPE_MASK: u32 = 0o170000;
const UNIX_SYMBOLIC_LINK: u32 = 0o120000;

/// 判断 Unix 权限位是否将条目标识为符号链接。
///
/// 适用于任何在 header 中携带 Unix mode 的归档格式（ZIP 的 external
/// attributes、tar 的 mode 字段）。
pub fn is_symbolic_link(unix_mode: Option<u32>) -> bool {
    unix_mode.is_some_and(|mode| mode & UNIX_FILE_TYPE_MASK == UNIX_SYMBOLIC_LINK)
}

/// 将归档中的符号链接载荷解析为可移植的相对路径。
///
/// 注意这里校验的是"可移植且不逃逸"这一性质，而非 POSIX 的"相对链接所在
/// 目录"语义：本函数的结果只用于给出拒绝理由，解压流程不会真正创建链接。
/// 若将来要支持创建链接，必须由调用方补充相对于链接目录的解析。
///
/// 符号链接载荷是原始字节，必须是 UTF-8 编码且仅包含普通斜杠分隔的组件。
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
