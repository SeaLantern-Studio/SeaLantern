//! 资源文件的命名与启用状态规则（纯逻辑，零 IO）。
//!
//! Minecraft 生态沿用"文件名后缀"表达启用状态：禁用时在文件名末尾追加
//! `.disabled`（例如 `foo.jar` → `foo.jar.disabled`），文件内容保持不变。

/// 禁用状态使用的文件名后缀。
pub const DISABLED_SUFFIX: &str = ".disabled";

/// 可管理的资源文件扩展名。
const RESOURCE_EXTENSIONS: &[&str] = &["jar", "zip"];

/// 判断文件名是否处于禁用状态（后缀比较不区分大小写，零分配）。
pub fn is_disabled(file_name: &str) -> bool {
    let bytes = file_name.as_bytes();
    let suffix = DISABLED_SUFFIX.as_bytes();

    bytes.len() >= suffix.len() && bytes[bytes.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

/// 返回启用状态下的文件名（移除禁用后缀；已启用时原样返回）。
pub fn enabled_name(file_name: &str) -> &str {
    if is_disabled(file_name) {
        &file_name[..file_name.len() - DISABLED_SUFFIX.len()]
    } else {
        file_name
    }
}

/// 返回禁用状态下的文件名（追加禁用后缀，对已禁用名称幂等）。
pub fn disabled_name(file_name: &str) -> String {
    if is_disabled(file_name) {
        file_name.to_string()
    } else {
        format!("{file_name}{DISABLED_SUFFIX}")
    }
}

/// 判断文件名是否是可管理的资源文件（`*.jar` / `*.zip`，含禁用状态）。
pub fn is_resource_file(file_name: &str) -> bool {
    let base = enabled_name(file_name);
    base.rsplit_once('.').is_some_and(|(_, extension)| {
        RESOURCE_EXTENSIONS
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_resource_files() {
        assert!(is_resource_file("foo.jar"));
        assert!(is_resource_file("foo.JAR"));
        assert!(is_resource_file("foo.jar.disabled"));
        assert!(is_resource_file("data.zip"));
        assert!(!is_resource_file("foo.txt"));
        assert!(!is_resource_file("foo"));
        assert!(!is_resource_file("foo.jar.bak"));
    }

    #[test]
    fn toggles_enabled_and_disabled_names() {
        assert_eq!(disabled_name("foo.jar"), "foo.jar.disabled");
        assert_eq!(disabled_name("foo.jar.disabled"), "foo.jar.disabled");
        assert_eq!(enabled_name("foo.jar.disabled"), "foo.jar");
        assert_eq!(enabled_name("foo.jar"), "foo.jar");
    }

    #[test]
    fn disabled_detection_is_case_insensitive() {
        assert!(is_disabled("foo.jar.DISABLED"));
        assert_eq!(enabled_name("foo.jar.Disabled"), "foo.jar");
    }
}
