use super::{FsError, SafeRelativePath};

/// 解析可移植的归档条目路径。
///
/// ZIP 和 TAR 适配器必须将结果路径传递给基于目录句柄的提取器。
/// 在此处返回拼接后的路径会重新引入验证与文件创建之间的符号链接替换竞争。
pub fn archive_entry_path(entry_name: &str) -> Result<SafeRelativePath, FsError> {
    SafeRelativePath::parse(entry_name)
}

/// 解析归档中的符号链接目标。
///
/// 归档适配器必须拒绝指向提取根目录之外的目标。
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
